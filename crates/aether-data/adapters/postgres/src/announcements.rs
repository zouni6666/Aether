use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row};

use aether_data_contracts::repository::announcements::*;
use aether_data_contracts::DataLayerError;
use aether_data_query::{push_eq, push_limit, push_limit_offset, WhereClause};

use crate::error::SqlxResultExt;

const ANNOUNCEMENT_SELECT: &str = r#"
SELECT
  a.id,
  a.title,
  a.content,
  a.type,
  a.priority,
  a.is_active,
  a.is_pinned,
  a.requires_ack,
  a.author_id,
  u.username AS author_username,
  EXTRACT(EPOCH FROM a.start_time)::bigint AS start_time_unix_secs,
  EXTRACT(EPOCH FROM a.end_time)::bigint AS end_time_unix_secs,
  EXTRACT(EPOCH FROM a.created_at)::bigint AS created_at_unix_ms,
  EXTRACT(EPOCH FROM a.updated_at)::bigint AS updated_at_unix_secs
FROM announcements a
LEFT JOIN users u ON u.id = a.author_id
"#;

const LIST_REQUIRED_UNREAD_ACTIVE_ANNOUNCEMENTS_SQL: &str = r#"
SELECT
  a.id,
  a.title,
  a.content,
  a.type,
  a.priority,
  a.is_active,
  a.is_pinned,
  a.requires_ack,
  a.author_id,
  u.username AS author_username,
  EXTRACT(EPOCH FROM a.start_time)::bigint AS start_time_unix_secs,
  EXTRACT(EPOCH FROM a.end_time)::bigint AS end_time_unix_secs,
  EXTRACT(EPOCH FROM a.created_at)::bigint AS created_at_unix_ms,
  EXTRACT(EPOCH FROM a.updated_at)::bigint AS updated_at_unix_secs
FROM announcements a
LEFT JOIN users u ON u.id = a.author_id
WHERE a.is_active = TRUE
  AND a.requires_ack = TRUE
  AND (a.start_time IS NULL OR a.start_time <= TO_TIMESTAMP($2::double precision))
  AND (a.end_time IS NULL OR a.end_time >= TO_TIMESTAMP($2::double precision))
  AND NOT EXISTS (
    SELECT 1
    FROM announcement_reads r
    WHERE r.user_id = $1
      AND r.announcement_id = a.id
  )
ORDER BY a.is_pinned DESC, a.priority DESC, a.created_at DESC, a.id ASC
LIMIT $3
"#;

const LIST_USER_ANNOUNCEMENTS_SQL: &str = r#"
WITH visible AS MATERIALIZED (
  SELECT a.*, EXISTS (
    SELECT 1 FROM announcement_reads r WHERE r.user_id = $1 AND r.announcement_id = a.id
  ) AS is_read
  FROM announcements a
  WHERE a.is_active = TRUE
    AND (a.start_time IS NULL OR a.start_time <= TO_TIMESTAMP($2::double precision))
    AND (a.end_time IS NULL OR a.end_time >= TO_TIMESTAMP($2::double precision))
), counts AS (
  SELECT count(*) FILTER (WHERE NOT $3 OR NOT is_read)::bigint AS total,
         count(*) FILTER (WHERE NOT is_read)::bigint AS unread_count
  FROM visible
)
SELECT
  counts.total,
  counts.unread_count,
  a.id,
  a.title,
  a.content,
  a.type,
  a.priority,
  a.is_active,
  a.is_pinned,
  a.requires_ack,
  a.is_read,
  a.author_id,
  u.username AS author_username,
  EXTRACT(EPOCH FROM a.start_time)::bigint AS start_time_unix_secs,
  EXTRACT(EPOCH FROM a.end_time)::bigint AS end_time_unix_secs,
  EXTRACT(EPOCH FROM a.created_at)::bigint AS created_at_unix_ms,
  EXTRACT(EPOCH FROM a.updated_at)::bigint AS updated_at_unix_secs
FROM counts
LEFT JOIN LATERAL (
  SELECT * FROM visible WHERE NOT $3 OR NOT is_read
  ORDER BY is_pinned DESC, priority DESC, created_at DESC, id ASC
  LIMIT $4 OFFSET $5
) a ON TRUE
LEFT JOIN users u ON u.id = a.author_id
ORDER BY a.is_pinned DESC, a.priority DESC, a.created_at DESC, a.id ASC
"#;

const CREATE_ANNOUNCEMENT_SQL: &str = r#"
INSERT INTO announcements (
  id,
  title,
  content,
  type,
  priority,
  author_id,
  is_active,
  is_pinned,
  requires_ack,
  start_time,
  end_time,
  created_at,
  updated_at
)
VALUES (
  $1,
  $2,
  $3,
  $4,
  $5,
  $6,
  TRUE,
  $7,
  $8,
  $9,
  $10,
  NOW(),
  NOW()
)
RETURNING
  id,
  title,
  content,
  type,
  priority,
  is_active,
  is_pinned,
  requires_ack,
  author_id,
  (SELECT username FROM users WHERE id = announcements.author_id) AS author_username,
  EXTRACT(EPOCH FROM start_time)::bigint AS start_time_unix_secs,
  EXTRACT(EPOCH FROM end_time)::bigint AS end_time_unix_secs,
  EXTRACT(EPOCH FROM created_at)::bigint AS created_at_unix_ms,
  EXTRACT(EPOCH FROM updated_at)::bigint AS updated_at_unix_secs
"#;

const UPDATE_ANNOUNCEMENT_SQL: &str = r#"
UPDATE announcements
SET
  title = COALESCE($2, title),
  content = COALESCE($3, content),
  type = COALESCE($4, type),
  priority = COALESCE($5, priority),
  is_active = COALESCE($6, is_active),
  is_pinned = COALESCE($7, is_pinned),
  requires_ack = COALESCE($8, requires_ack),
  start_time = COALESCE($9, start_time),
  end_time = COALESCE($10, end_time),
  updated_at = NOW()
WHERE id = $1
RETURNING
  id,
  title,
  content,
  type,
  priority,
  is_active,
  is_pinned,
  requires_ack,
  author_id,
  (SELECT username FROM users WHERE id = announcements.author_id) AS author_username,
  EXTRACT(EPOCH FROM start_time)::bigint AS start_time_unix_secs,
  EXTRACT(EPOCH FROM end_time)::bigint AS end_time_unix_secs,
  EXTRACT(EPOCH FROM created_at)::bigint AS created_at_unix_ms,
  EXTRACT(EPOCH FROM updated_at)::bigint AS updated_at_unix_secs
"#;

const DELETE_ANNOUNCEMENT_SQL: &str = r#"
DELETE FROM announcements
WHERE id = $1
"#;
const DELETE_ANNOUNCEMENT_READS_SQL: &str = r#"
DELETE FROM announcement_reads
WHERE announcement_id = $1
"#;

const MARK_ANNOUNCEMENT_AS_READ_SQL: &str = r#"
INSERT INTO announcement_reads (
  id,
  user_id,
  announcement_id,
  read_at
)
VALUES (
  $1,
  $2,
  $3,
  TO_TIMESTAMP($4::double precision)
)
ON CONFLICT (user_id, announcement_id) DO NOTHING
"#;

#[derive(Debug, Clone)]
pub struct SqlxAnnouncementReadRepository {
    pool: PgPool,
}

impl SqlxAnnouncementReadRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn apply_active_filter(
        builder: &mut QueryBuilder<'_, Postgres>,
        where_clause: &mut WhereClause,
        active_only: bool,
        now_unix_secs: u64,
    ) {
        if !active_only {
            return;
        }

        where_clause.push_next(builder);
        builder
            .push("a.is_active = TRUE AND (a.start_time IS NULL OR a.start_time <= TO_TIMESTAMP(")
            .push_bind(now_unix_secs as f64)
            .push("::double precision)) AND (a.end_time IS NULL OR a.end_time >= TO_TIMESTAMP(")
            .push_bind(now_unix_secs as f64)
            .push("::double precision))");
    }
}

#[async_trait]
impl AnnouncementReadRepository for SqlxAnnouncementReadRepository {
    async fn find_by_id(
        &self,
        announcement_id: &str,
    ) -> Result<Option<StoredAnnouncement>, DataLayerError> {
        let mut builder = QueryBuilder::<Postgres>::new(ANNOUNCEMENT_SELECT);
        let mut where_clause = WhereClause::new();
        push_eq(
            &mut builder,
            &mut where_clause,
            "a.id",
            announcement_id.to_string(),
        );
        push_limit(&mut builder, 1);
        let row = builder
            .build()
            .fetch_optional(&self.pool)
            .await
            .map_postgres_err()?;
        row.as_ref().map(map_announcement_row).transpose()
    }

    async fn list_announcements(
        &self,
        query: &AnnouncementListQuery,
    ) -> Result<StoredAnnouncementPage, DataLayerError> {
        let now_unix_secs = query.now_unix_secs.unwrap_or_else(current_unix_secs);
        let mut count_builder =
            QueryBuilder::<Postgres>::new("SELECT COUNT(a.id) AS total FROM announcements a");
        let mut count_where = WhereClause::new();
        Self::apply_active_filter(
            &mut count_builder,
            &mut count_where,
            query.active_only,
            now_unix_secs,
        );
        let total = count_builder
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_postgres_err()?
            .max(0) as u64;

        let mut list_builder = QueryBuilder::<Postgres>::new(ANNOUNCEMENT_SELECT);
        let mut list_where = WhereClause::new();
        Self::apply_active_filter(
            &mut list_builder,
            &mut list_where,
            query.active_only,
            now_unix_secs,
        );
        list_builder
            .push(" ORDER BY a.is_pinned DESC, a.priority DESC, a.created_at DESC, a.id ASC");
        push_limit_offset(&mut list_builder, query.limit as i64, query.offset as i64);
        let rows = list_builder
            .build()
            .fetch_all(&self.pool)
            .await
            .map_postgres_err()?;
        let items = rows
            .iter()
            .map(map_announcement_row)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(StoredAnnouncementPage { items, total })
    }

    async fn list_user_announcements(
        &self,
        user_id: &str,
        query: &UserAnnouncementListQuery,
    ) -> Result<StoredUserAnnouncementPage, DataLayerError> {
        query.validate()?;
        // A single statement preserves counts even when the selected page is empty.
        let rows = sqlx::query(LIST_USER_ANNOUNCEMENTS_SQL)
            .bind(user_id)
            .bind(query.now_unix_secs as f64)
            .bind(query.unread_only)
            .bind(query.limit as i64)
            .bind(query.offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_postgres_err()?;
        let mut page = StoredUserAnnouncementPage::default();
        for row in rows {
            page.total = row.try_get::<i64, _>("total").map_postgres_err()? as u64;
            page.unread_count = row.try_get::<i64, _>("unread_count").map_postgres_err()? as u64;
            if row
                .try_get::<Option<String>, _>("id")
                .map_postgres_err()?
                .is_some()
            {
                page.items.push(StoredUserAnnouncement {
                    announcement: map_announcement_row(&row)?,
                    is_read: row.try_get("is_read").map_postgres_err()?,
                });
            }
        }
        Ok(page)
    }

    async fn count_unread_active_announcements(
        &self,
        user_id: &str,
        now_unix_secs: u64,
    ) -> Result<u64, DataLayerError> {
        let mut builder =
            QueryBuilder::<Postgres>::new("SELECT COUNT(a.id) AS total FROM announcements a");
        let mut where_clause = WhereClause::new();
        Self::apply_active_filter(&mut builder, &mut where_clause, true, now_unix_secs);
        where_clause.push_next(&mut builder);
        builder
            .push("NOT EXISTS (SELECT 1 FROM announcement_reads r WHERE r.user_id = ")
            .push_bind(user_id.to_string())
            .push(" AND r.announcement_id = a.id)");
        let total = builder
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_postgres_err()?
            .max(0) as u64;
        Ok(total)
    }

    async fn list_required_unread_active_announcements(
        &self,
        user_id: &str,
        now_unix_secs: u64,
        limit: usize,
    ) -> Result<Vec<StoredAnnouncement>, DataLayerError> {
        let rows = sqlx::query(LIST_REQUIRED_UNREAD_ACTIVE_ANNOUNCEMENTS_SQL)
            .bind(user_id)
            .bind(now_unix_secs as f64)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_postgres_err()?;
        rows.iter().map(map_announcement_row).collect()
    }
}

#[async_trait]
impl AnnouncementWriteRepository for SqlxAnnouncementReadRepository {
    async fn create_announcement(
        &self,
        record: CreateAnnouncementRecord,
    ) -> Result<StoredAnnouncement, DataLayerError> {
        record.validate()?;
        let row = sqlx::query(CREATE_ANNOUNCEMENT_SQL)
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(record.title)
            .bind(record.content)
            .bind(record.kind)
            .bind(record.priority)
            .bind(record.author_id)
            .bind(record.is_pinned)
            .bind(record.requires_ack)
            .bind(optional_datetime(record.start_time_unix_secs))
            .bind(optional_datetime(record.end_time_unix_secs))
            .fetch_one(&self.pool)
            .await
            .map_postgres_err()?;
        map_announcement_row(&row)
    }

    async fn update_announcement(
        &self,
        record: UpdateAnnouncementRecord,
    ) -> Result<Option<StoredAnnouncement>, DataLayerError> {
        record.validate()?;
        let row = sqlx::query(UPDATE_ANNOUNCEMENT_SQL)
            .bind(record.announcement_id)
            .bind(record.title)
            .bind(record.content)
            .bind(record.kind)
            .bind(record.priority)
            .bind(record.is_active)
            .bind(record.is_pinned)
            .bind(record.requires_ack)
            .bind(optional_datetime(record.start_time_unix_secs))
            .bind(optional_datetime(record.end_time_unix_secs))
            .fetch_optional(&self.pool)
            .await
            .map_postgres_err()?;
        row.as_ref().map(map_announcement_row).transpose()
    }

    async fn delete_announcement(&self, announcement_id: &str) -> Result<bool, DataLayerError> {
        let mut tx = self.pool.begin().await.map_postgres_err()?;
        sqlx::query(DELETE_ANNOUNCEMENT_READS_SQL)
            .bind(announcement_id)
            .execute(&mut *tx)
            .await
            .map_postgres_err()?;
        let result = sqlx::query(DELETE_ANNOUNCEMENT_SQL)
            .bind(announcement_id)
            .execute(&mut *tx)
            .await
            .map_postgres_err()?;
        tx.commit().await.map_postgres_err()?;
        Ok(result.rows_affected() > 0)
    }

    async fn mark_announcement_as_read(
        &self,
        user_id: &str,
        announcement_id: &str,
        read_at_unix_secs: u64,
    ) -> Result<bool, DataLayerError> {
        let result = sqlx::query(MARK_ANNOUNCEMENT_AS_READ_SQL)
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(user_id)
            .bind(announcement_id)
            .bind(read_at_unix_secs as f64)
            .execute(&self.pool)
            .await
            .map_postgres_err()?;
        Ok(result.rows_affected() > 0)
    }
}

fn optional_datetime(unix_secs: Option<u64>) -> Option<chrono::DateTime<Utc>> {
    unix_secs.and_then(|value| {
        i64::try_from(value)
            .ok()
            .and_then(|value| Utc.timestamp_opt(value, 0).single())
    })
}

fn current_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn map_announcement_row(row: &PgRow) -> Result<StoredAnnouncement, DataLayerError> {
    StoredAnnouncement::new(
        row.try_get("id").map_postgres_err()?,
        row.try_get("title").map_postgres_err()?,
        row.try_get("content").map_postgres_err()?,
        row.try_get("type").map_postgres_err()?,
        row.try_get("priority").map_postgres_err()?,
        row.try_get("is_active").map_postgres_err()?,
        row.try_get("is_pinned").map_postgres_err()?,
        row.try_get("requires_ack").map_postgres_err()?,
        row.try_get("author_id").map_postgres_err()?,
        row.try_get("author_username").map_postgres_err()?,
        row.try_get("start_time_unix_secs").map_postgres_err()?,
        row.try_get("end_time_unix_secs").map_postgres_err()?,
        row.try_get("created_at_unix_ms").map_postgres_err()?,
        row.try_get("updated_at_unix_secs").map_postgres_err()?,
    )
}

#[cfg(test)]
mod tests {
    use super::SqlxAnnouncementReadRepository;
    use crate::{PostgresPoolConfig, PostgresPoolFactory};

    #[tokio::test]
    #[ignore = "requires AETHER_TEST_DATABASE_URL; uses connection-local temporary tables"]
    async fn live_personal_announcement_page_preserves_global_unread_and_visibility() {
        use aether_data_contracts::repository::announcements::{
            AnnouncementReadRepository, AnnouncementWriteRepository, UserAnnouncementListQuery,
        };
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&std::env::var("AETHER_TEST_DATABASE_URL").unwrap())
            .await
            .unwrap();
        sqlx::raw_sql(
            "CREATE TEMP TABLE announcements (LIKE public.announcements INCLUDING ALL);
          CREATE TEMP TABLE announcement_reads (LIKE public.announcement_reads INCLUDING ALL);
          CREATE TEMP TABLE users (id varchar(36) PRIMARY KEY, username varchar(255));",
        )
        .execute(&pool)
        .await
        .unwrap();
        let repository = SqlxAnnouncementReadRepository::new(pool.clone());
        let now = 1_800_000_000_i64;
        for (id, active, pinned, priority, start, end) in [
            ("pinned", true, true, 0, None, None),
            ("normal-a", true, false, 20, Some(now), Some(now)),
            ("normal-b", true, false, 20, None, None),
            ("draft", false, false, 99, None, None),
            ("future", true, false, 99, Some(now + 1), None),
            ("expired", true, false, 99, None, Some(now - 1)),
        ] {
            sqlx::query("INSERT INTO announcements(id,title,content,type,priority,is_active,is_pinned,created_at,updated_at,start_time,end_time) VALUES($1,$1,'content','info',$2,$3,$4,TO_TIMESTAMP($5::double precision),TO_TIMESTAMP($5::double precision),TO_TIMESTAMP($6::double precision),TO_TIMESTAMP($7::double precision))")
                .bind(id).bind(priority).bind(active).bind(pinned).bind(now as f64)
                .bind(start.map(|value| value as f64)).bind(end.map(|value| value as f64))
                .execute(&pool).await.unwrap();
        }
        repository
            .mark_announcement_as_read("reader", "pinned", now as u64)
            .await
            .unwrap();
        let mut query = UserAnnouncementListQuery {
            unread_only: false,
            offset: 0,
            limit: 1,
            now_unix_secs: now as u64,
        };
        let first = repository
            .list_user_announcements("reader", &query)
            .await
            .unwrap();
        assert_eq!((first.total, first.unread_count), (3, 2));
        assert_eq!(first.items[0].announcement.id, "pinned");
        assert!(first.items[0].is_read);
        query.unread_only = true;
        query.offset = 1;
        let second = repository
            .list_user_announcements("reader", &query)
            .await
            .unwrap();
        assert_eq!((second.total, second.unread_count), (2, 2));
        assert_eq!(second.items[0].announcement.id, "normal-b");
        assert!(!second.items[0].is_read);
        query.offset = i64::MAX as usize;
        let empty = repository
            .list_user_announcements("reader", &query)
            .await
            .unwrap();
        assert!(empty.items.is_empty());
        assert_eq!((empty.total, empty.unread_count), (2, 2));
        let other = repository
            .list_user_announcements("other", &query)
            .await
            .unwrap();
        assert_eq!((other.total, other.unread_count), (3, 3));
        repository
            .mark_announcement_as_read("reader", "normal-a", now as u64)
            .await
            .unwrap();
        query.offset = 0;
        let after_read = repository
            .list_user_announcements("reader", &query)
            .await
            .unwrap();
        assert_eq!((after_read.total, after_read.unread_count), (1, 1));
        assert_eq!(
            repository
                .count_unread_active_announcements("reader", now as u64)
                .await
                .unwrap(),
            1
        );
        query.limit = 101;
        assert!(repository
            .list_user_announcements("reader", &query)
            .await
            .is_err());
        pool.close().await;
    }

    #[tokio::test]
    async fn repository_constructs_from_lazy_pool() {
        let factory = PostgresPoolFactory::new(PostgresPoolConfig {
            database_url: "postgres://localhost/aether".to_string(),
            min_connections: 1,
            max_connections: 4,
            acquire_timeout_ms: 1_000,
            idle_timeout_ms: 5_000,
            max_lifetime_ms: 30_000,
            statement_cache_capacity: 64,
            require_ssl: false,
        })
        .expect("factory should build");

        let pool = factory.connect_lazy().expect("pool should build");
        let _repository = SqlxAnnouncementReadRepository::new(pool);
    }
}
