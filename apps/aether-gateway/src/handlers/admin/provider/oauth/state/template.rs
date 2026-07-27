use super::super::errors::build_internal_control_error_response;
use crate::handlers::admin::provider::shared::support::ADMIN_PROVIDER_OAUTH_DATA_UNAVAILABLE_DETAIL;
use crate::handlers::admin::request::{
    admin_provider_oauth_template as request_admin_provider_oauth_template,
    admin_provider_oauth_template_types,
    is_fixed_provider_type_for_admin_oauth as request_is_fixed_provider_type_for_admin_oauth,
    AdminProviderOAuthTemplate,
};
use axum::{body::Body, http, response::Response};
use serde_json::json;

pub(crate) fn is_fixed_provider_type_for_provider_oauth(provider_type: &str) -> bool {
    request_is_fixed_provider_type_for_admin_oauth(provider_type)
}

pub(crate) fn admin_provider_oauth_template(
    provider_type: &str,
) -> Option<AdminProviderOAuthTemplate> {
    request_admin_provider_oauth_template(provider_type)
}

pub(crate) fn build_admin_provider_oauth_supported_types_payload() -> Vec<serde_json::Value> {
    let service = aether_oauth::provider::ProviderOAuthService::with_builtin_adapters();
    admin_provider_oauth_template_types()
        .filter_map(|provider_type| admin_provider_oauth_template(provider_type))
        .map(|template| {
            let capabilities = service
                .adapter(template.provider_type)
                .ok()
                .map(|adapter| adapter.capabilities());
            json!({
                "provider_type": template.provider_type,
                "display_name": template.display_name,
                "scopes": template.scopes,
                "redirect_uri": template.redirect_uri,
                "authorize_url": template.authorize_url,
                "token_url": template.token_url,
                "use_pkce": template.use_pkce,
                "supports_authorization_code": capabilities.as_ref().is_some_and(|value| value.supports_authorization_code),
                "supports_cookie_authorization": capabilities.as_ref().is_some_and(|value| value.supports_cookie_authorization),
                "supports_refresh_token_import": capabilities.as_ref().is_some_and(|value| value.supports_refresh_token_import),
                "supports_batch_import": capabilities.as_ref().is_some_and(|value| value.supports_batch_import),
            })
        })
        .collect()
}

pub(crate) fn build_admin_provider_oauth_backend_unavailable_response() -> Response<Body> {
    build_internal_control_error_response(
        http::StatusCode::SERVICE_UNAVAILABLE,
        ADMIN_PROVIDER_OAUTH_DATA_UNAVAILABLE_DETAIL,
    )
}
