use super::*;

struct AnnouncementUserFixture {
    url: String,
    token: String,
    device: String,
    client: reqwest::Client,
    upstream_hits: Arc<Mutex<usize>>,
    gateway: tokio::task::JoinHandle<()>,
    upstream: tokio::task::JoinHandle<()>,
}

impl AnnouncementUserFixture {
    async fn start(
        user_id: &str,
        role: &str,
        repository: Arc<InMemoryAnnouncementReadRepository>,
    ) -> Self {
        let now = Utc::now();
        let mut user = sample_auth_user(now);
        user.id = user_id.into();
        user.role = role.into();
        let session = format!("{user_id}-session");
        let device = format!("{user_id}-device");
        let token = build_test_auth_token(
            "access",
            serde_json::Map::from_iter([
                ("user_id".into(), json!(user.id)),
                ("role".into(), json!(user.role)),
                (
                    "created_at".into(),
                    json!(user.created_at.map(|value| value.to_rfc3339())),
                ),
                ("session_id".into(), json!(session)),
            ]),
            now + chrono::Duration::hours(1),
        );
        let (url, upstream_hits, gateway, upstream) = start_auth_announcement_gateway_with_state(
            user,
            sample_auth_wallet(user_id, now),
            [sample_auth_session(
                user_id,
                &session,
                &device,
                "refresh-placeholder",
                now,
            )],
            repository,
        )
        .await;
        Self {
            url,
            token,
            device,
            client: reqwest::Client::new(),
            upstream_hits,
            gateway,
            upstream,
        }
    }

    fn auth(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        request
            .bearer_auth(&self.token)
            .header("x-client-device-id", &self.device)
            .header("user-agent", "AetherTest/1.0")
    }

    async fn list(&self, suffix: &str) -> serde_json::Value {
        let response = self
            .auth(
                self.client
                    .get(format!("{}/api/announcements/users/me{suffix}", self.url)),
            )
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "{suffix}");
        response.json().await.unwrap()
    }

    async fn mark_read(&self, id: &str) {
        let response = self
            .auth(
                self.client
                    .patch(format!("{}/api/announcements/{id}/read-status", self.url)),
            )
            .json(&json!({ "is_read": true }))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    async fn read_all(&self) {
        let response = self
            .auth(
                self.client
                    .post(format!("{}/api/announcements/read-all", self.url)),
            )
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}

impl Drop for AnnouncementUserFixture {
    fn drop(&mut self) {
        self.gateway.abort();
        self.upstream.abort();
    }
}

fn announcement(id: &str, pinned: bool, priority: i32, created_at: i64) -> StoredAnnouncement {
    StoredAnnouncement::new(
        id.into(),
        format!("Title {id}"),
        format!("Content {id}"),
        "info".into(),
        priority,
        true,
        pinned,
        false,
        Some("author-1".into()),
        Some("Author".into()),
        None,
        None,
        created_at,
        created_at,
    )
    .unwrap()
}

fn ids(payload: &serde_json::Value) -> Vec<&str> {
    payload["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap())
        .collect()
}

#[tokio::test]
async fn announcement_user_list_keeps_visible_order_and_global_unread_counts_across_pages() {
    let now = Utc::now().timestamp();
    let mut draft = announcement("draft", true, 999, now);
    draft.is_active = false;
    let mut future = announcement("future", true, 999, now);
    future.start_time_unix_secs = Some((now + 3600) as u64);
    let mut expired = announcement("expired", true, 999, now);
    expired.end_time_unix_secs = Some((now - 3600) as u64);
    let repository = Arc::new(InMemoryAnnouncementReadRepository::seed_with_reads(
        vec![
            announcement("active-e", false, 100, now - 20),
            announcement("active-b", true, 10, now - 100),
            draft,
            announcement("active-d", false, 100, now - 10),
            future,
            announcement("active-c", true, 5, now - 50),
            expired,
            announcement("active-a", true, 10, now - 100),
        ],
        [("list-user".into(), "active-b".into())],
    ));
    let fixture = AnnouncementUserFixture::start("list-user", "user", repository).await;
    let all = fixture.list("").await;
    assert_eq!(
        ids(&all),
        ["active-a", "active-b", "active-c", "active-d", "active-e"]
    );
    assert_eq!(all["total"], 5);
    assert_eq!(all["unread_count"], 4);
    assert_eq!(all["limit"], 20);
    assert_eq!(all["offset"], 0);
    assert_eq!(all["items"][0]["is_read"], false);
    assert_eq!(all["items"][1]["is_read"], true);
    assert_eq!(all["items"][0]["content"], "Content active-a");
    assert_eq!(all["items"][0]["author"]["username"], "Author");

    let page = fixture.list("?limit=2&offset=1&unread_only=false").await;
    assert_eq!(ids(&page), ["active-b", "active-c"]);
    assert_eq!(page["total"], 5);
    assert_eq!(page["unread_count"], 4);
    assert_eq!(page["limit"], 2);
    assert_eq!(page["offset"], 1);
    let unread = fixture.list("?limit=2&offset=1&unread_only=true").await;
    assert_eq!(ids(&unread), ["active-c", "active-d"]);
    assert_eq!(unread["total"], 4);
    assert_eq!(unread["unread_count"], 4);
    for (query, total) in [
        ("?limit=2&offset=999", 5),
        ("?limit=2&offset=999&unread_only=true", 4),
    ] {
        let outside = fixture.list(query).await;
        assert!(ids(&outside).is_empty());
        assert_eq!(outside["total"], total);
        assert_eq!(outside["unread_count"], 4);
    }

    fixture.mark_read("active-d").await;
    let unread = fixture.list("?unread_only=true").await;
    assert_eq!(ids(&unread), ["active-a", "active-c", "active-e"]);
    assert_eq!(unread["total"], 3);
    assert_eq!(unread["unread_count"], 3);
    let all = fixture.list("").await;
    assert_eq!(all["items"][3]["is_read"], true);
    let badge = fixture
        .auth(fixture.client.get(format!(
            "{}/api/announcements/users/me/unread-count",
            fixture.url
        )))
        .send()
        .await
        .unwrap();
    assert_eq!(badge.status(), StatusCode::OK);
    assert_eq!(
        badge.json::<serde_json::Value>().await.unwrap()["unread_count"],
        3
    );
    fixture.read_all().await;
    let all = fixture.list("").await;
    assert_eq!(all["total"], 5);
    assert_eq!(all["unread_count"], 0);
    assert!(all["items"]
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item["is_read"] == true));
    let unread = fixture.list("?unread_only=true").await;
    assert!(ids(&unread).is_empty());
    assert_eq!(unread["total"], 0);
    assert_eq!(*fixture.upstream_hits.lock().unwrap(), 0);
}

#[tokio::test]
async fn announcement_user_list_is_personal_for_user_admin_and_audit_admin() {
    let now = Utc::now().timestamp();
    let repository = Arc::new(InMemoryAnnouncementReadRepository::seed(vec![
        announcement("shared-notice", false, 1, now),
        announcement("second-notice", false, 0, now),
    ]));
    for role in ["user", "admin", "audit_admin"] {
        let fixture = AnnouncementUserFixture::start(
            &format!("announcement-{role}"),
            role,
            Arc::clone(&repository),
        )
        .await;
        let initial = fixture.list("").await;
        assert_eq!(initial["total"], 2, "{role}");
        assert_eq!(
            initial["unread_count"], 2,
            "another user's reads must not affect {role}"
        );
        assert!(initial["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["is_read"] == false));
        fixture.mark_read("shared-notice").await;
        let changed = fixture.list("").await;
        assert_eq!(changed["unread_count"], 1, "{role}");
        assert_eq!(changed["items"][0]["is_read"], true);
        fixture.read_all().await;
        assert_eq!(fixture.list("").await["unread_count"], 0, "{role}");
        assert_eq!(*fixture.upstream_hits.lock().unwrap(), 0);
    }
}

#[tokio::test]
async fn announcement_user_list_requires_auth_and_rejects_invalid_filters() {
    let fixture = AnnouncementUserFixture::start(
        "announcement-validation",
        "user",
        Arc::new(InMemoryAnnouncementReadRepository::seed(Vec::new())),
    )
    .await;
    let url = format!("{}/api/announcements/users/me", fixture.url);
    assert_eq!(
        fixture.client.get(&url).send().await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );
    for query in [
        "limit=0",
        "limit=101",
        "limit=-1",
        "limit=1.5",
        "limit=",
        "offset=-1",
        "offset=1.5",
        "offset=9223372036854775808",
        "offset=18446744073709551616",
        "unread_only=invalid",
        "unread_only=",
        "limit=20&limit=20",
        "offset=0&offset=0",
        "unread_only=false&unread_only=false",
        "active_only=false",
        "user_id=another-user",
        "now=4102444800",
    ] {
        let response = fixture
            .auth(fixture.client.get(format!("{url}?{query}")))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{query}");
    }
    let empty = fixture.list("/?limit=100&offset=9223372036854775807").await;
    assert!(ids(&empty).is_empty());
    assert_eq!(empty["total"], 0);
    assert_eq!(empty["unread_count"], 0);
    assert_eq!(empty["limit"], 100);
    assert_eq!(empty["offset"], i64::MAX);
    assert_eq!(*fixture.upstream_hits.lock().unwrap(), 0);
}
