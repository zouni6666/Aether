use super::*;
use aether_data_contracts::repository::{
    provider_catalog::{ProviderCatalogWriteRepository, StoredProviderCatalogProvider},
    routing_profiles::{
        CreateRoutingGroupRecord, RoutingGroupReadRepository, RoutingGroupWriteRepository,
        UpdateRoutingGroupRecord,
    },
};
use serde_json::json;

#[tokio::test]
async fn postgres_scoped_provider_creation_rolls_back_and_serializes_group_saves() {
    let Some(server) = ManagedPostgresServer::try_start()
        .await
        .expect("local postgres should start or skip")
    else {
        return;
    };
    let pool = PgPool::connect(server.database_url()).await.unwrap();
    prepare_and_apply_clean_postgres_database(&pool).await;
    let groups =
        crate::repository::routing_profiles::PostgresRoutingGroupRepository::new(pool.clone());
    let providers =
        crate::repository::provider_catalog::SqlxProviderCatalogReadRepository::new(pool.clone());
    for id in ["selected", "other"] {
        groups
            .create_routing_group(CreateRoutingGroupRecord {
                id: id.into(),
                name: id.into(),
                description: None,
                enabled: true,
                is_system_default: id == "selected",
                sort_order: 0,
                config_json: json!({"disabled_providers": ["existing-disabled"]}),
                version: 1,
                created_at: 1,
                updated_at: 1,
                published_at: None,
            })
            .await
            .unwrap();
    }
    let provider =
        StoredProviderCatalogProvider::new("new".into(), "new".into(), None, "custom".into())
            .unwrap();
    assert!(providers
        .create_provider_in_routing_group(&provider, None, "missing")
        .await
        .is_err());
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM providers")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
    assert!(groups
        .list_routing_groups()
        .await
        .unwrap()
        .iter()
        .all(|group| group.version == 1));

    providers
        .create_provider_in_routing_group(&provider, None, "selected")
        .await
        .unwrap();
    let before = groups.list_routing_groups().await.unwrap();
    for group in &before {
        assert_eq!(group.version, if group.id == "selected" { 1 } else { 2 });
        assert_eq!(
            group.config_json["disabled_providers"],
            if group.id == "selected" {
                json!(["existing-disabled"])
            } else {
                json!(["existing-disabled", "new"])
            }
        );
    }
    // The INSERT fails after group updates execute. Its transaction must undo
    // every exclusion and version change along with any priority shifts.
    assert!(providers
        .create_provider_in_routing_group(&provider, Some(0), "other")
        .await
        .is_err());
    assert_eq!(groups.list_routing_groups().await.unwrap(), before);
    assert!(groups
        .update_routing_group(
            "other",
            UpdateRoutingGroupRecord {
                expected_version: Some(1),
                config_json: Some(json!({"disabled_providers": []})),
                ..Default::default()
            }
        )
        .await
        .is_err());
    assert_eq!(groups.list_routing_groups().await.unwrap(), before);

    let concurrent_provider = StoredProviderCatalogProvider::new(
        "concurrent".into(),
        "concurrent".into(),
        None,
        "custom".into(),
    )
    .unwrap();
    let (created, edited) = tokio::join!(
        providers.create_provider_in_routing_group(&concurrent_provider, None, "selected"),
        groups.update_routing_group(
            "other",
            UpdateRoutingGroupRecord {
                expected_version: Some(2),
                config_json: Some(json!({"disabled_providers": ["new"]})),
                ..Default::default()
            }
        )
    );
    created.unwrap();
    let other = groups
        .list_routing_groups()
        .await
        .unwrap()
        .into_iter()
        .find(|group| group.id == "other")
        .unwrap();
    assert!(other.config_json["disabled_providers"]
        .as_array()
        .unwrap()
        .contains(&json!("concurrent")));
    assert_eq!(other.version, if edited.is_ok() { 4 } else { 3 });
}
