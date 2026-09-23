//! Codex 客户端画像的运行时发布与官方 CLI 版本刷新。

use std::collections::BTreeMap;
use std::future::Future;
use std::time::Duration;

use aether_runtime_state::RuntimeState;
use futures_util::StreamExt as _;
use reqwest::{redirect::Policy, Client};
use semver::Version;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::ai_serving::api::{codex_client_version, set_codex_cli_version};
use crate::AppState;

const CLI_RELEASE_ENDPOINT: &str = "https://registry.npmjs.org/@openai%2Fcodex/latest";
const PROFILE_CACHE_KEY: &str = "aether:codex:client-profile:v1";
const PROFILE_CACHE_TTL: Duration = Duration::from_secs(30 * 24 * 60 * 60);
const PROFILE_REFRESH_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const RELEASE_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const RELEASE_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RELEASE_BYTES: usize = 256 * 1024;
const CLI_TARGETS: [&str; 6] = [
    "darwin-arm64",
    "darwin-x64",
    "linux-arm64",
    "linux-x64",
    "win32-arm64",
    "win32-x64",
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NpmRelease {
    name: String,
    version: String,
    optional_dependencies: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CachedProfile {
    version: String,
    verified_at_unix_secs: u64,
}

#[derive(Debug, thiserror::Error)]
enum ProfileRefreshError {
    #[error("Codex CLI release client initialization failed: {0}")]
    Client(#[from] reqwest::Error),
    #[error("Codex CLI release request returned HTTP {0}")]
    HttpStatus(u16),
    #[error("Codex CLI release response exceeded {MAX_RELEASE_BYTES} bytes")]
    ResponseTooLarge,
    #[error("Codex CLI release metadata is invalid")]
    InvalidMetadata,
    #[error("Codex CLI release version is older than the active profile")]
    Rollback,
    #[error("Codex CLI profile cache operation failed: {0}")]
    Cache(String),
}

fn version_sequence(version: &str) -> Result<u64, ProfileRefreshError> {
    let parsed = Version::parse(version).map_err(|_| ProfileRefreshError::InvalidMetadata)?;
    if !parsed.pre.is_empty()
        || !parsed.build.is_empty()
        || parsed.major > 999
        || parsed.minor > 999
        || parsed.patch > 999
    {
        return Err(ProfileRefreshError::InvalidMetadata);
    }
    Ok(1 + parsed.major * 1_000_000 + parsed.minor * 1_000 + parsed.patch)
}

/// 校验官方 npm stable 标签及六个平台依赖来自同一版本发布。
fn parse_cli_release(bytes: &[u8]) -> Result<String, ProfileRefreshError> {
    if bytes.len() > MAX_RELEASE_BYTES {
        return Err(ProfileRefreshError::ResponseTooLarge);
    }
    let release = serde_json::from_slice::<NpmRelease>(bytes)
        .map_err(|_| ProfileRefreshError::InvalidMetadata)?;
    let sequence = version_sequence(&release.version)?;
    if sequence == 0
        || release.name != "@openai/codex"
        || CLI_TARGETS.iter().any(|target| {
            release
                .optional_dependencies
                .get(&format!("@openai/codex-{target}"))
                != Some(&format!("npm:@openai/codex@{}-{target}", release.version))
        })
    {
        return Err(ProfileRefreshError::InvalidMetadata);
    }
    Ok(release.version)
}

fn refresh_enabled_from(value: Option<&str>) -> bool {
    !value.is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "0" | "false" | "off"
        )
    })
}

fn refresh_enabled() -> bool {
    refresh_enabled_from(
        std::env::var("AETHER_CODEX_CLIENT_PROFILE_REFRESH")
            .ok()
            .as_deref(),
    )
}

fn fixed_version_from(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    if value.is_empty() || version_sequence(value).is_err() {
        None
    } else {
        Some(value.to_owned())
    }
}

fn fixed_version_override() -> Option<String> {
    let value = std::env::var("AETHER_CODEX_CLIENT_VERSION").ok()?;
    let version = fixed_version_from(Some(&value));
    if version.is_none() {
        warn!(
            event_name = "codex_client_profile_fixed_version_invalid",
            "AETHER_CODEX_CLIENT_VERSION is invalid; using cached or built-in profile"
        );
    }
    version
}

fn build_release_client() -> Result<Client, ProfileRefreshError> {
    Client::builder()
        .https_only(true)
        .no_proxy()
        .redirect(Policy::none())
        .connect_timeout(RELEASE_CONNECT_TIMEOUT)
        .timeout(RELEASE_REQUEST_TIMEOUT)
        .build()
        .map_err(ProfileRefreshError::Client)
}

async fn fetch_latest_cli_version(client: &Client) -> Result<String, ProfileRefreshError> {
    let response = client
        .get(CLI_RELEASE_ENDPOINT)
        .send()
        .await
        .map_err(ProfileRefreshError::Client)?;
    if !response.status().is_success() {
        return Err(ProfileRefreshError::HttpStatus(response.status().as_u16()));
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RELEASE_BYTES as u64)
    {
        return Err(ProfileRefreshError::ResponseTooLarge);
    }

    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(ProfileRefreshError::Client)?;
        if bytes.len().saturating_add(chunk.len()) > MAX_RELEASE_BYTES {
            return Err(ProfileRefreshError::ResponseTooLarge);
        }
        bytes.extend_from_slice(&chunk);
    }
    parse_cli_release(&bytes)
}

async fn restore_cached_profile(runtime: &RuntimeState) -> Result<(), ProfileRefreshError> {
    let Some(raw) = runtime
        .kv_get(PROFILE_CACHE_KEY)
        .await
        .map_err(|err| ProfileRefreshError::Cache(err.to_string()))?
    else {
        return Ok(());
    };
    let cached = serde_json::from_str::<CachedProfile>(&raw)
        .map_err(|_| ProfileRefreshError::InvalidMetadata)?;
    if let Some(version) = cached_version_to_restore(&cached, &codex_client_version())? {
        set_codex_cli_version(&version).map_err(|_| ProfileRefreshError::InvalidMetadata)?;
        info!(
            event_name = "codex_client_profile_restored",
            version = %version,
            verified_at_unix_secs = cached.verified_at_unix_secs,
            "restored cached Codex CLI profile"
        );
    }
    Ok(())
}

fn cached_version_to_restore(
    cached: &CachedProfile,
    active_version: &str,
) -> Result<Option<String>, ProfileRefreshError> {
    let cached_sequence = version_sequence(&cached.version)?;
    let active_sequence = version_sequence(active_version)?;
    Ok((cached_sequence >= active_sequence).then(|| cached.version.clone()))
}

async fn refresh_once_with_fetch<F, Fut>(
    runtime: &RuntimeState,
    fixed_version: Option<&str>,
    refresh_is_enabled: bool,
    fetch_latest: F,
) -> Result<String, ProfileRefreshError>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<String, ProfileRefreshError>>,
{
    if let Some(version) = fixed_version {
        set_codex_cli_version(version).map_err(|_| ProfileRefreshError::InvalidMetadata)?;
        return Ok(version.to_owned());
    }

    if let Err(error) = restore_cached_profile(runtime).await {
        // 缓存损坏或暂时不可用不应阻断官方版本检查；当前进程继续使用旧画像。
        warn!(
            event_name = "codex_client_profile_cache_restore_failed",
            error = %error,
            "could not restore cached Codex CLI profile"
        );
    }
    if !refresh_is_enabled {
        return Ok(codex_client_version());
    }

    let version = fetch_latest().await?;
    let current = codex_client_version();
    if version_sequence(&version)? < version_sequence(&current)? {
        return Err(ProfileRefreshError::Rollback);
    }

    let cached = CachedProfile {
        version: version.clone(),
        verified_at_unix_secs: chrono::Utc::now().timestamp().max(0) as u64,
    };
    let serialized =
        serde_json::to_string(&cached).map_err(|_| ProfileRefreshError::InvalidMetadata)?;
    set_codex_cli_version(&version).map_err(|_| ProfileRefreshError::InvalidMetadata)?;
    if let Err(error) = runtime
        .kv_set(PROFILE_CACHE_KEY, serialized, Some(PROFILE_CACHE_TTL))
        .await
    {
        // 本地画像已经完成原子替换；缓存写失败只影响下次进程启动的恢复。
        warn!(
            event_name = "codex_client_profile_cache_write_failed",
            error = %error,
            "published Codex CLI profile locally but could not persist the cache"
        );
    }
    Ok(version)
}

async fn refresh_once(runtime: &RuntimeState) -> Result<String, ProfileRefreshError> {
    let fixed_version = fixed_version_override();
    refresh_once_with_fetch(
        runtime,
        fixed_version.as_deref(),
        refresh_enabled(),
        || async {
            let client = build_release_client()?;
            fetch_latest_cli_version(&client).await
        },
    )
    .await
}

pub(crate) async fn prewarm(runtime: &RuntimeState) -> Result<String, String> {
    refresh_once(runtime).await.map_err(|err| err.to_string())
}

pub(crate) fn spawn_worker(app: AppState) -> tokio::task::JoinHandle<()> {
    crate::task_runtime::spawn_singleton_worker(
        app,
        crate::task_runtime::TASK_KEY_CODEX_CLIENT_PROFILE,
        |app| async move {
            let mut interval = tokio::time::interval(PROFILE_REFRESH_INTERVAL);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            // 启动阶段由 prewarm 完成一次检查；后台任务只负责后续每日刷新，避免重复建连。
            interval.tick().await;
            loop {
                interval.tick().await;
                match refresh_once(app.runtime_state()).await {
                    Ok(version) => info!(
                        event_name = "codex_client_profile_refreshed",
                        version = %version,
                        "refreshed Codex CLI profile"
                    ),
                    Err(error) => warn!(
                        event_name = "codex_client_profile_refresh_failed",
                        error = %error,
                        "keeping the previous Codex CLI profile after refresh failure"
                    ),
                }
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, OnceLock,
    };
    use std::time::Duration;

    use aether_runtime_state::{MemoryRuntimeStateConfig, RuntimeState};

    use super::{
        cached_version_to_restore, fixed_version_from, parse_cli_release, refresh_enabled_from,
        refresh_once_with_fetch, CachedProfile, ProfileRefreshError, PROFILE_CACHE_KEY,
    };
    use crate::ai_serving::api::{
        codex_client_profile, codex_client_version, set_codex_cli_version,
        set_codex_client_profile, CodexClientProfile,
    };

    static PROFILE_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    struct ProfileRestore(CodexClientProfile);

    impl Drop for ProfileRestore {
        fn drop(&mut self) {
            set_codex_client_profile(self.0.clone());
        }
    }

    fn profile_restore_guard() -> (std::sync::MutexGuard<'static, ()>, ProfileRestore) {
        let lock = PROFILE_TEST_LOCK.get_or_init(|| Mutex::new(()));
        let guard = lock.lock().expect("profile test lock");
        let restore = ProfileRestore(codex_client_profile());
        (guard, restore)
    }

    #[test]
    fn accepts_only_one_verified_cli_release_for_all_targets() {
        let body = serde_json::json!({
            "name": "@openai/codex",
            "version": "0.200.1",
            "optionalDependencies": {
                "@openai/codex-darwin-arm64": "npm:@openai/codex@0.200.1-darwin-arm64",
                "@openai/codex-darwin-x64": "npm:@openai/codex@0.200.1-darwin-x64",
                "@openai/codex-linux-arm64": "npm:@openai/codex@0.200.1-linux-arm64",
                "@openai/codex-linux-x64": "npm:@openai/codex@0.200.1-linux-x64",
                "@openai/codex-win32-arm64": "npm:@openai/codex@0.200.1-win32-arm64",
                "@openai/codex-win32-x64": "npm:@openai/codex@0.200.1-win32-x64"
            }
        });
        assert_eq!(
            parse_cli_release(&serde_json::to_vec(&body).unwrap()).unwrap(),
            "0.200.1"
        );
    }

    #[test]
    fn rejects_incomplete_platform_release() {
        let body = serde_json::json!({
            "name": "@openai/codex",
            "version": "0.200.1",
            "optionalDependencies": {}
        });
        assert!(parse_cli_release(&serde_json::to_vec(&body).unwrap()).is_err());
    }

    #[test]
    fn refresh_and_fixed_version_environment_policies_are_strict() {
        assert!(!refresh_enabled_from(Some("off")));
        assert!(!refresh_enabled_from(Some(" FALSE ")));
        assert!(refresh_enabled_from(None));
        assert_eq!(
            fixed_version_from(Some(" 0.200.1 ")).as_deref(),
            Some("0.200.1")
        );
        assert!(fixed_version_from(Some("0.200.1-beta.1")).is_none());
        assert!(fixed_version_from(Some("1.2")).is_none());
    }

    #[test]
    fn cached_profile_never_rewinds_active_profile() {
        let cached = CachedProfile {
            version: "0.200.1".to_string(),
            verified_at_unix_secs: 1,
        };
        assert_eq!(
            cached_version_to_restore(&cached, "0.200.0").unwrap(),
            Some("0.200.1".to_string())
        );
        assert_eq!(cached_version_to_restore(&cached, "0.201.0").unwrap(), None);
    }

    #[tokio::test]
    async fn cache_hit_is_restored_without_network_when_refresh_is_disabled() {
        let (_lock, _restore) = profile_restore_guard();
        let runtime = RuntimeState::memory(MemoryRuntimeStateConfig::default());
        runtime
            .kv_set(
                PROFILE_CACHE_KEY,
                serde_json::to_string(&CachedProfile {
                    version: "0.200.1".to_string(),
                    verified_at_unix_secs: 1,
                })
                .unwrap(),
                Some(Duration::from_secs(60)),
            )
            .await
            .unwrap();

        let result = refresh_once_with_fetch(&runtime, None, false, || async {
            Err(ProfileRefreshError::HttpStatus(599))
        })
        .await
        .unwrap();

        assert_eq!(result, "0.200.1");
        assert_eq!(codex_client_version(), "0.200.1");
    }

    #[tokio::test]
    async fn refresh_failure_keeps_previous_profile() {
        let (_lock, _restore) = profile_restore_guard();
        let runtime = RuntimeState::memory(MemoryRuntimeStateConfig::default());
        let before = codex_client_profile();
        let result = refresh_once_with_fetch(&runtime, None, true, || async {
            Err(ProfileRefreshError::HttpStatus(503))
        })
        .await;

        assert!(matches!(result, Err(ProfileRefreshError::HttpStatus(503))));
        assert_eq!(codex_client_profile(), before);
    }

    #[tokio::test]
    async fn fixed_version_override_skips_network_and_publishes_profile() {
        let (_lock, _restore) = profile_restore_guard();
        let runtime = RuntimeState::memory(MemoryRuntimeStateConfig::default());
        let fetch_called = AtomicBool::new(false);
        let result = refresh_once_with_fetch(&runtime, Some("0.220.0"), true, || async {
            fetch_called.store(true, Ordering::SeqCst);
            Ok("0.221.0".to_string())
        })
        .await
        .unwrap();

        assert_eq!(result, "0.220.0");
        assert!(!fetch_called.load(Ordering::SeqCst));
        assert_eq!(codex_client_version(), "0.220.0");
    }

    #[tokio::test]
    async fn rollback_is_rejected_without_replacing_profile() {
        let (_lock, _restore) = profile_restore_guard();
        set_codex_cli_version("0.220.0").unwrap();
        let runtime = RuntimeState::memory(MemoryRuntimeStateConfig::default());
        let result =
            refresh_once_with_fetch(&runtime, None, true, || async { Ok("0.219.9".to_string()) })
                .await;

        assert!(matches!(result, Err(ProfileRefreshError::Rollback)));
        assert_eq!(codex_client_version(), "0.220.0");
    }
}
