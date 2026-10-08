use aether_data_contracts::repository::routing_profiles::{
    RoutingGroupBindingQuery, RoutingGroupBindingSubject, RoutingGroupLookupKey,
    RoutingGroupReadRepository, StoredRoutingGroup,
};
use thiserror::Error;

pub(crate) const ROUTING_GROUP_HEADER: &str = "x-aether-scheduler-group";

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub(crate) enum GatewayRoutingSelectionError {
    #[error("no enabled routing strategy is configured for this request")]
    NoDefault,
    #[error("routing group was explicitly requested but was not found: {0}")]
    NotFound(String),
    #[error("routing group was explicitly requested but is not enabled: {0}")]
    Disabled(String),
    #[error("routing group was explicitly requested but is not allowed for this principal: {0}")]
    Forbidden(String),
    #[error("routing group repository lookup failed: {0}")]
    Repository(String),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct GatewayRoutingSelectionInput<'a> {
    pub explicit_group: Option<&'a str>,
    /// A public group selected on the user's API key. Explicit request headers
    /// take precedence, while an unavailable saved choice must fail closed.
    pub preferred_group: Option<&'a str>,
    pub user_id: Option<&'a str>,
    pub api_key_id: Option<&'a str>,
    pub user_group_ids: &'a [String],
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct GatewayRoutingGroupSelection {
    pub group: Option<StoredRoutingGroup>,
    pub source: String,
}

pub(crate) async fn select_gateway_routing_group(
    repository: &(impl RoutingGroupReadRepository + ?Sized),
    input: GatewayRoutingSelectionInput<'_>,
) -> Result<GatewayRoutingGroupSelection, GatewayRoutingSelectionError> {
    if let Some(explicit) = input
        .explicit_group
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let group = repository
            .find_routing_group(RoutingGroupLookupKey::Id(explicit))
            .await
            .map_err(repository_selection_error)?;
        let group = match group {
            Some(group) => Some(group),
            None => repository
                .find_routing_group(RoutingGroupLookupKey::Name(explicit))
                .await
                .map_err(repository_selection_error)?,
        };
        let Some(group) = group else {
            return Err(GatewayRoutingSelectionError::NotFound(explicit.to_string()));
        };
        if !group.enabled {
            return Err(GatewayRoutingSelectionError::Disabled(group.id));
        }
        if !explicit_group_allowed(repository, &group, &input).await? {
            return Err(GatewayRoutingSelectionError::Forbidden(group.id));
        }
        return Ok(GatewayRoutingGroupSelection {
            group: Some(group),
            source: "explicit_header".to_string(),
        });
    }

    if let Some(preferred) = input
        .preferred_group
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let group = repository
            .find_routing_group(RoutingGroupLookupKey::Id(preferred))
            .await
            .map_err(repository_selection_error)?
            .ok_or_else(|| GatewayRoutingSelectionError::NotFound(preferred.to_string()))?;
        if !group.enabled {
            return Err(GatewayRoutingSelectionError::Disabled(group.id));
        }
        if !has_authenticated_principal(&input) || !routing_group_is_user_visible(&group) {
            return Err(GatewayRoutingSelectionError::Forbidden(group.id));
        }
        return Ok(GatewayRoutingGroupSelection {
            group: Some(group),
            source: "api_key_selection".to_string(),
        });
    }

    // When there are no bindings at all, no principal-specific lookup can
    // produce a group. The data-state repository answers this with a cached
    // existence query, so the common "routing configured but unused" case
    // does not materialize the binding table per API key/user.
    let has_bindings = repository
        .has_any_routing_group_binding()
        .await
        .map_err(repository_selection_error)?;
    if !has_bindings {
        let system_default = repository
            .find_routing_group(RoutingGroupLookupKey::SystemDefault)
            .await
            .map_err(repository_selection_error)?
            .filter(|group| group.enabled);
        return Ok(GatewayRoutingGroupSelection {
            group: system_default,
            source: "system_default".to_string(),
        });
    }

    for (subject_type, subject_id, source) in default_binding_candidates(&input) {
        let bindings = repository
            .list_routing_group_bindings(&RoutingGroupBindingQuery {
                group_id: None,
                subject_type: Some(subject_type),
                subject_id: Some(subject_id.to_string()),
            })
            .await
            .map_err(repository_selection_error)?;
        for binding in bindings.into_iter().filter(|binding| binding.is_default) {
            let group = repository
                .find_routing_group(RoutingGroupLookupKey::Id(&binding.group_id))
                .await
                .map_err(repository_selection_error)?;
            if let Some(group) = group.filter(|group| group.enabled) {
                return Ok(GatewayRoutingGroupSelection {
                    group: Some(group),
                    source: source.to_string(),
                });
            }
        }
    }

    let system_default = repository
        .find_routing_group(RoutingGroupLookupKey::SystemDefault)
        .await
        .map_err(repository_selection_error)?
        .filter(|group| group.enabled);
    Ok(GatewayRoutingGroupSelection {
        group: system_default,
        source: "system_default".to_string(),
    })
}

async fn explicit_group_allowed(
    repository: &(impl RoutingGroupReadRepository + ?Sized),
    group: &StoredRoutingGroup,
    input: &GatewayRoutingSelectionInput<'_>,
) -> Result<bool, GatewayRoutingSelectionError> {
    // Public selection is opt-in. Turning it off does not revoke existing
    // administrator-granted bindings or the system-default compatibility path.
    if has_authenticated_principal(input) && routing_group_is_user_visible(group) {
        return Ok(true);
    }
    if group.is_system_default {
        return Ok(true);
    }
    for (subject_type, subject_id, _) in default_binding_candidates(input) {
        let bindings = repository
            .list_routing_group_bindings(&RoutingGroupBindingQuery {
                group_id: Some(group.id.clone()),
                subject_type: Some(subject_type),
                subject_id: Some(subject_id.to_string()),
            })
            .await
            .map_err(repository_selection_error)?;
        if bindings.iter().any(|binding| binding.allow_explicit_select) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(crate) fn routing_group_is_user_visible(group: &StoredRoutingGroup) -> bool {
    group
        .config_json
        .get("user_visible")
        .and_then(serde_json::Value::as_bool)
        == Some(true)
}

fn has_authenticated_principal(input: &GatewayRoutingSelectionInput<'_>) -> bool {
    input
        .user_id
        .into_iter()
        .chain(input.api_key_id)
        .any(|id| !id.trim().is_empty())
}

fn repository_selection_error(error: impl std::fmt::Display) -> GatewayRoutingSelectionError {
    GatewayRoutingSelectionError::Repository(error.to_string())
}

fn default_binding_candidates<'a>(
    input: &'a GatewayRoutingSelectionInput<'a>,
) -> Vec<(RoutingGroupBindingSubject, &'a str, &'static str)> {
    let mut candidates = Vec::new();
    if let Some(api_key_id) = input.api_key_id {
        candidates.push((
            RoutingGroupBindingSubject::ApiKey,
            api_key_id,
            "api_key_default",
        ));
    }
    if let Some(user_id) = input.user_id {
        candidates.push((RoutingGroupBindingSubject::User, user_id, "user_default"));
    }
    for group_id in input.user_group_ids {
        candidates.push((
            RoutingGroupBindingSubject::UserGroup,
            group_id.as_str(),
            "user_group_default",
        ));
    }
    candidates
}

#[cfg(test)]
mod tests {
    use aether_data::repository::routing_profiles::InMemoryRoutingGroupRepository;
    use aether_data_contracts::repository::routing_profiles::{
        CreateRoutingGroupBindingRecord, CreateRoutingGroupRecord, RoutingGroupWriteRepository,
        StoredRoutingGroupBinding, StoredRoutingGroupVersion,
    };
    use aether_data_contracts::DataLayerError;
    use async_trait::async_trait;
    use serde_json::json;

    use super::*;

    struct FailingRoutingGroupRepository {
        id_lookup_is_missing: bool,
    }

    impl FailingRoutingGroupRepository {
        fn failure<T>() -> Result<T, DataLayerError> {
            Err(DataLayerError::Sql(
                "routing repository unavailable".to_string(),
            ))
        }
    }

    #[async_trait]
    impl RoutingGroupReadRepository for FailingRoutingGroupRepository {
        async fn list_routing_groups(&self) -> Result<Vec<StoredRoutingGroup>, DataLayerError> {
            Self::failure()
        }

        async fn find_routing_group(
            &self,
            lookup: RoutingGroupLookupKey<'_>,
        ) -> Result<Option<StoredRoutingGroup>, DataLayerError> {
            if self.id_lookup_is_missing && matches!(lookup, RoutingGroupLookupKey::Id(_)) {
                return Ok(None);
            }
            Self::failure()
        }

        async fn list_routing_group_bindings(
            &self,
            _query: &RoutingGroupBindingQuery,
        ) -> Result<Vec<StoredRoutingGroupBinding>, DataLayerError> {
            Self::failure()
        }

        async fn list_routing_group_versions(
            &self,
            _group_id: &str,
        ) -> Result<Vec<StoredRoutingGroupVersion>, DataLayerError> {
            Self::failure()
        }
    }

    #[tokio::test]
    async fn public_groups_allow_authenticated_selection_but_visibility_is_opt_in() {
        let repository = InMemoryRoutingGroupRepository::default();
        for (id, config, enabled) in [
            ("public", json!({ "user_visible": true }), true),
            ("private", json!({ "user_visible": false }), true),
            ("legacy", json!({}), true),
            ("malformed", json!({ "user_visible": "true" }), true),
            ("disabled", json!({ "user_visible": true }), false),
        ] {
            repository
                .create_routing_group(CreateRoutingGroupRecord {
                    id: id.into(),
                    name: format!("{id}-name"),
                    description: None,
                    enabled,
                    is_system_default: false,
                    sort_order: 0,
                    config_json: config,
                    version: 1,
                    created_at: 1,
                    updated_at: 1,
                    published_at: None,
                })
                .await
                .unwrap();
        }
        for (user_id, api_key_id) in [(Some("user-1"), None), (None, Some("key-1"))] {
            for explicit in ["public", "public-name"] {
                let selected = select_gateway_routing_group(
                    &repository,
                    GatewayRoutingSelectionInput {
                        explicit_group: Some(explicit),
                        user_id,
                        api_key_id,
                        user_group_ids: &[],
                        ..Default::default()
                    },
                )
                .await
                .unwrap();
                assert_eq!(selected.group.unwrap().id, "public");
            }
        }
        for id in ["private", "legacy", "malformed"] {
            assert_eq!(
                select_gateway_routing_group(
                    &repository,
                    GatewayRoutingSelectionInput {
                        explicit_group: Some(id),
                        user_id: Some("user-1"),
                        ..Default::default()
                    }
                )
                .await
                .unwrap_err(),
                GatewayRoutingSelectionError::Forbidden(id.into())
            );
        }
        assert_eq!(
            select_gateway_routing_group(
                &repository,
                GatewayRoutingSelectionInput {
                    explicit_group: Some("disabled"),
                    user_id: Some("user-1"),
                    ..Default::default()
                }
            )
            .await
            .unwrap_err(),
            GatewayRoutingSelectionError::Disabled("disabled".into())
        );
        assert_eq!(
            select_gateway_routing_group(
                &repository,
                GatewayRoutingSelectionInput {
                    explicit_group: Some("public"),
                    ..Default::default()
                }
            )
            .await
            .unwrap_err(),
            GatewayRoutingSelectionError::Forbidden("public".into())
        );
    }

    #[tokio::test]
    async fn api_key_selected_public_group_precedes_bindings_and_header_precedes_saved_choice() {
        let repository = InMemoryRoutingGroupRepository::default();
        for (id, visible) in [
            ("selected", true),
            ("header", true),
            ("private-default", false),
        ] {
            repository
                .create_routing_group(CreateRoutingGroupRecord {
                    id: id.into(),
                    name: id.into(),
                    description: None,
                    enabled: true,
                    is_system_default: false,
                    sort_order: 0,
                    config_json: json!({ "user_visible": visible }),
                    version: 1,
                    created_at: 1,
                    updated_at: 1,
                    published_at: None,
                })
                .await
                .unwrap();
        }
        repository
            .create_routing_group_binding(CreateRoutingGroupBindingRecord {
                id: "admin-default".into(),
                group_id: "private-default".into(),
                subject_type: RoutingGroupBindingSubject::ApiKey,
                subject_id: "key-1".into(),
                is_default: true,
                allow_explicit_select: true,
                created_at: 1,
                updated_at: 1,
            })
            .await
            .unwrap();

        for (explicit, preferred, expected, source) in [
            (None, Some("selected"), "selected", "api_key_selection"),
            (
                Some("header"),
                Some("selected"),
                "header",
                "explicit_header",
            ),
            // An explicit authorized request overrides an invalid saved choice.
            (Some("header"), Some("missing"), "header", "explicit_header"),
            (
                Some("private-default"),
                Some("selected"),
                "private-default",
                "explicit_header",
            ),
            (None, None, "private-default", "api_key_default"),
        ] {
            let selection = select_gateway_routing_group(
                &repository,
                GatewayRoutingSelectionInput {
                    explicit_group: explicit,
                    preferred_group: preferred,
                    api_key_id: Some("key-1"),
                    user_id: Some("user-1"),
                    user_group_ids: &[],
                },
            )
            .await
            .unwrap();
            assert_eq!(selection.group.unwrap().id, expected);
            assert_eq!(selection.source, source);
        }
    }

    #[tokio::test]
    async fn unavailable_api_key_group_selection_fails_closed_without_default_fallback() {
        let repository = InMemoryRoutingGroupRepository::default();
        for (id, visible, enabled, is_default) in [
            ("private-default", false, true, true),
            ("disabled", true, false, false),
        ] {
            repository
                .create_routing_group(CreateRoutingGroupRecord {
                    id: id.into(),
                    name: format!("{id}-name"),
                    description: None,
                    enabled,
                    is_system_default: is_default,
                    sort_order: 0,
                    config_json: json!({ "user_visible": visible }),
                    version: 1,
                    created_at: 1,
                    updated_at: 1,
                    published_at: None,
                })
                .await
                .unwrap();
        }
        for (id, error) in [
            (
                "private-default",
                GatewayRoutingSelectionError::Forbidden("private-default".into()),
            ),
            (
                "disabled",
                GatewayRoutingSelectionError::Disabled("disabled".into()),
            ),
            (
                "missing",
                GatewayRoutingSelectionError::NotFound("missing".into()),
            ),
            // Saved selections are stable IDs, not mutable group names.
            (
                "private-default-name",
                GatewayRoutingSelectionError::NotFound("private-default-name".into()),
            ),
        ] {
            assert_eq!(
                select_gateway_routing_group(
                    &repository,
                    GatewayRoutingSelectionInput {
                        preferred_group: Some(id),
                        user_id: Some("user-1"),
                        ..Default::default()
                    }
                )
                .await
                .unwrap_err(),
                error
            );
        }
    }

    #[tokio::test]
    async fn selects_api_key_default_binding() {
        let repository = InMemoryRoutingGroupRepository::default();
        repository
            .create_routing_group(CreateRoutingGroupRecord {
                id: "group-1".to_string(),
                name: "default".to_string(),
                description: None,
                enabled: true,
                is_system_default: false,
                sort_order: 0,
                config_json: json!({}),
                version: 1,
                created_at: 1,
                updated_at: 1,
                published_at: None,
            })
            .await
            .unwrap();
        repository
            .create_routing_group_binding(CreateRoutingGroupBindingRecord {
                id: "binding-1".to_string(),
                group_id: "group-1".to_string(),
                subject_type: RoutingGroupBindingSubject::ApiKey,
                subject_id: "api-key-1".to_string(),
                is_default: true,
                allow_explicit_select: true,
                created_at: 1,
                updated_at: 1,
            })
            .await
            .unwrap();

        let selection = select_gateway_routing_group(
            &repository,
            GatewayRoutingSelectionInput {
                explicit_group: None,
                preferred_group: None,
                user_id: None,
                api_key_id: Some("api-key-1"),
                user_group_ids: &[],
            },
        )
        .await
        .unwrap();

        assert_eq!(selection.source, "api_key_default");
        assert_eq!(selection.group.unwrap().id, "group-1");
    }

    #[tokio::test]
    async fn selects_system_default_when_no_bindings_exist() {
        let repository = InMemoryRoutingGroupRepository::default();
        repository
            .create_routing_group(CreateRoutingGroupRecord {
                id: "system-default".to_string(),
                name: "system-default".to_string(),
                description: None,
                enabled: true,
                is_system_default: true,
                sort_order: 0,
                config_json: json!({}),
                version: 1,
                created_at: 1,
                updated_at: 1,
                published_at: None,
            })
            .await
            .unwrap();

        let selection = select_gateway_routing_group(
            &repository,
            GatewayRoutingSelectionInput {
                explicit_group: None,
                preferred_group: None,
                user_id: Some("user-1"),
                api_key_id: Some("api-key-1"),
                user_group_ids: &["user-group-1".to_string()],
            },
        )
        .await
        .unwrap();

        assert_eq!(selection.source, "system_default");
        assert_eq!(selection.group.unwrap().id, "system-default");
    }

    #[tokio::test]
    async fn selects_explicit_group_allowed_by_user_group_binding() {
        let repository = InMemoryRoutingGroupRepository::default();
        repository
            .create_routing_group(CreateRoutingGroupRecord {
                id: "private-group".to_string(),
                name: "private".to_string(),
                description: None,
                enabled: true,
                is_system_default: false,
                sort_order: 0,
                config_json: json!({ "user_visible": false }),
                version: 1,
                created_at: 1,
                updated_at: 1,
                published_at: None,
            })
            .await
            .unwrap();
        repository
            .create_routing_group_binding(CreateRoutingGroupBindingRecord {
                id: "binding-explicit".to_string(),
                group_id: "private-group".to_string(),
                subject_type: RoutingGroupBindingSubject::UserGroup,
                subject_id: "team-1".to_string(),
                is_default: false,
                allow_explicit_select: true,
                created_at: 1,
                updated_at: 1,
            })
            .await
            .unwrap();

        let selection = select_gateway_routing_group(
            &repository,
            GatewayRoutingSelectionInput {
                explicit_group: Some("private-group"),
                preferred_group: None,
                user_id: Some("user-1"),
                api_key_id: Some("api-key-1"),
                user_group_ids: &["team-1".to_string()],
            },
        )
        .await
        .unwrap();

        assert_eq!(selection.source, "explicit_header");
        assert_eq!(selection.group.unwrap().id, "private-group");
    }

    #[tokio::test]
    async fn rejects_explicit_group_that_does_not_exist() {
        let repository = InMemoryRoutingGroupRepository::default();

        let error = select_gateway_routing_group(
            &repository,
            GatewayRoutingSelectionInput {
                explicit_group: Some("missing"),
                preferred_group: None,
                user_id: Some("user-1"),
                api_key_id: Some("api-key-1"),
                user_group_ids: &[],
            },
        )
        .await
        .unwrap_err();

        assert_eq!(
            error,
            GatewayRoutingSelectionError::NotFound("missing".to_string())
        );
    }

    #[tokio::test]
    async fn propagates_explicit_name_lookup_failure_after_missing_id() {
        let repository = FailingRoutingGroupRepository {
            id_lookup_is_missing: true,
        };

        let error = select_gateway_routing_group(
            &repository,
            GatewayRoutingSelectionInput {
                explicit_group: Some("group-name"),
                preferred_group: None,
                user_id: Some("user-1"),
                api_key_id: Some("api-key-1"),
                user_group_ids: &[],
            },
        )
        .await
        .unwrap_err();

        assert_eq!(
            error,
            GatewayRoutingSelectionError::Repository(
                "sql error: routing repository unavailable".to_string()
            )
        );
    }

    #[tokio::test]
    async fn propagates_implicit_binding_lookup_failure() {
        let repository = FailingRoutingGroupRepository {
            id_lookup_is_missing: false,
        };

        let error = select_gateway_routing_group(
            &repository,
            GatewayRoutingSelectionInput {
                explicit_group: None,
                preferred_group: None,
                user_id: Some("user-1"),
                api_key_id: Some("api-key-1"),
                user_group_ids: &[],
            },
        )
        .await
        .unwrap_err();

        assert_eq!(
            error,
            GatewayRoutingSelectionError::Repository(
                "sql error: routing repository unavailable".to_string()
            )
        );
    }

    #[tokio::test]
    async fn rejects_explicit_disabled_group() {
        let repository = InMemoryRoutingGroupRepository::default();
        repository
            .create_routing_group(CreateRoutingGroupRecord {
                id: "disabled-group".to_string(),
                name: "disabled".to_string(),
                description: None,
                enabled: false,
                is_system_default: false,
                sort_order: 0,
                config_json: json!({}),
                version: 1,
                created_at: 1,
                updated_at: 1,
                published_at: None,
            })
            .await
            .unwrap();

        let error = select_gateway_routing_group(
            &repository,
            GatewayRoutingSelectionInput {
                explicit_group: Some("disabled-group"),
                preferred_group: None,
                user_id: Some("user-1"),
                api_key_id: Some("api-key-1"),
                user_group_ids: &[],
            },
        )
        .await
        .unwrap_err();

        assert_eq!(
            error,
            GatewayRoutingSelectionError::Disabled("disabled-group".to_string())
        );
    }

    #[tokio::test]
    async fn rejects_explicit_group_without_binding_permission() {
        let repository = InMemoryRoutingGroupRepository::default();
        repository
            .create_routing_group(CreateRoutingGroupRecord {
                id: "private-group".to_string(),
                name: "private".to_string(),
                description: None,
                enabled: true,
                is_system_default: false,
                sort_order: 0,
                config_json: json!({}),
                version: 1,
                created_at: 1,
                updated_at: 1,
                published_at: None,
            })
            .await
            .unwrap();
        repository
            .create_routing_group_binding(CreateRoutingGroupBindingRecord {
                id: "binding-1".to_string(),
                group_id: "private-group".to_string(),
                subject_type: RoutingGroupBindingSubject::ApiKey,
                subject_id: "api-key-1".to_string(),
                is_default: true,
                allow_explicit_select: false,
                created_at: 1,
                updated_at: 1,
            })
            .await
            .unwrap();

        let error = select_gateway_routing_group(
            &repository,
            GatewayRoutingSelectionInput {
                explicit_group: Some("private-group"),
                preferred_group: None,
                user_id: Some("user-1"),
                api_key_id: Some("api-key-1"),
                user_group_ids: &[],
            },
        )
        .await
        .unwrap_err();

        assert_eq!(
            error,
            GatewayRoutingSelectionError::Forbidden("private-group".to_string())
        );
    }
}
