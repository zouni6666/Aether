mod catalog;
mod runtime;
#[cfg(test)]
mod tests;

pub(crate) use aether_model_fetch::ModelFetchRunSummary;
pub(crate) use catalog::{
    codex_catalog_credential_scope_from_stored_key, codex_catalog_targets, load_codex_catalogs,
    normalize_codex_client_version, read_recent_codex_catalog_client_version,
    refresh_codex_catalog_target, CodexCatalogLoad, CodexCatalogRuntime, CodexCatalogTarget,
    NormalizedCodexClientVersion,
};
pub(crate) use runtime::state::ModelFetchRuntimeState;
pub(crate) use runtime::{
    perform_model_fetch_for_key, perform_model_fetch_for_keys, perform_model_fetch_once,
    spawn_model_fetch_worker,
};
