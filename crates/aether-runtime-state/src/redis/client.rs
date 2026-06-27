use crate::error::RedisResultExt;
use crate::redis::RedisKeyspace;
use crate::DataLayerError;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

pub(crate) type RedisClient = redis::Client;
pub(crate) type RedisManagedConnection = redis::aio::ConnectionManager;

const DEFAULT_BLOCKING_STREAM_LANES_FALLBACK: usize = 4;
const DEFAULT_BLOCKING_STREAM_LANES_CAP: usize = 16;
const MAX_BLOCKING_STREAM_LANES_CAP: usize = 64;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct RedisClientConfig {
    pub url: String,
    pub key_prefix: Option<String>,
}

impl RedisClientConfig {
    pub fn validate(&self) -> Result<(), DataLayerError> {
        let raw = self.url.trim();
        if raw.is_empty() {
            return Err(DataLayerError::InvalidConfiguration(
                "redis url cannot be empty".to_string(),
            ));
        }
        url::Url::parse(raw).map_err(|err| {
            DataLayerError::InvalidConfiguration(format!("invalid redis url: {err}"))
        })?;
        Ok(())
    }

    pub fn keyspace(&self) -> RedisKeyspace {
        RedisKeyspace::new(self.key_prefix.as_deref())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RedisClientFactory {
    config: RedisClientConfig,
}

impl RedisClientFactory {
    pub(crate) fn new(config: RedisClientConfig) -> Result<Self, DataLayerError> {
        config.validate()?;
        Ok(Self { config })
    }

    pub(crate) fn config(&self) -> &RedisClientConfig {
        &self.config
    }

    pub(crate) fn connect_lazy(&self) -> Result<RedisClient, DataLayerError> {
        RedisClient::open(self.config.url.clone()).map_redis_err()
    }

    pub(crate) async fn connect_router(
        &self,
        command_timeout_ms: Option<u64>,
    ) -> Result<RedisConnectionRouter, DataLayerError> {
        self.connect_router_with_blocking_stream_lanes(command_timeout_ms, None)
            .await
    }

    pub(crate) async fn connect_router_with_blocking_stream_lanes(
        &self,
        command_timeout_ms: Option<u64>,
        blocking_stream_lanes: Option<usize>,
    ) -> Result<RedisConnectionRouter, DataLayerError> {
        RedisConnectionRouter::connect(
            self.connect_lazy()?,
            command_timeout_ms,
            blocking_stream_lanes,
        )
        .await
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RedisConnectionLane {
    Fast,
    Stream,
    BlockingStream,
    Admin,
}

impl RedisConnectionLane {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Stream => "stream",
            Self::BlockingStream => "blocking_stream",
            Self::Admin => "admin",
        }
    }
}

#[derive(Clone)]
pub(crate) struct RedisConnectionRouter {
    fast: RedisManagedConnection,
    stream: RedisManagedConnection,
    blocking_stream: Arc<Vec<RedisManagedConnection>>,
    blocking_stream_next: Arc<AtomicUsize>,
    admin: RedisManagedConnection,
    metrics: Arc<RedisConnectionMetrics>,
}

impl std::fmt::Debug for RedisConnectionRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisConnectionRouter")
            .field("lanes", &["fast", "stream", "blocking_stream", "admin"])
            .field("blocking_stream_lanes", &self.blocking_stream.len())
            .finish()
    }
}

impl RedisConnectionRouter {
    pub(crate) async fn connect(
        client: RedisClient,
        command_timeout_ms: Option<u64>,
        blocking_stream_lanes: Option<usize>,
    ) -> Result<Self, DataLayerError> {
        let fast = connect_lane(
            &client,
            connection_manager_config(command_timeout_ms),
            RedisConnectionLane::Fast,
            command_timeout_ms,
        )
        .await?;
        let stream = connect_lane(
            &client,
            connection_manager_config(command_timeout_ms),
            RedisConnectionLane::Stream,
            command_timeout_ms,
        )
        .await?;
        let blocking_stream =
            connect_blocking_stream_lanes(&client, command_timeout_ms, blocking_stream_lanes)
                .await?;
        let admin = connect_lane(
            &client,
            connection_manager_config(command_timeout_ms),
            RedisConnectionLane::Admin,
            command_timeout_ms,
        )
        .await?;
        let blocking_stream_lanes = blocking_stream.len();
        info!(
            redis_lanes = "fast,stream,blocking_stream,admin",
            redis_blocking_stream_lanes = blocking_stream_lanes,
            "runtime redis connection lanes initialized"
        );
        Ok(Self {
            fast,
            stream,
            blocking_stream: Arc::new(blocking_stream),
            blocking_stream_next: Arc::new(AtomicUsize::new(0)),
            admin,
            metrics: Arc::new(RedisConnectionMetrics::default()),
        })
    }

    pub(crate) fn connection(&self, lane: RedisConnectionLane) -> RedisManagedConnection {
        match lane {
            RedisConnectionLane::Fast => self.fast.clone(),
            RedisConnectionLane::Stream => self.stream.clone(),
            RedisConnectionLane::BlockingStream => {
                let index = self.blocking_stream_next.fetch_add(1, Ordering::Relaxed)
                    % self.blocking_stream.len();
                self.blocking_stream[index].clone()
            }
            RedisConnectionLane::Admin => self.admin.clone(),
        }
    }

    pub(crate) fn record_error(&self, lane: RedisConnectionLane) {
        self.metrics
            .for_lane(lane)
            .errors
            .fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_timeout(&self, lane: RedisConnectionLane) {
        self.metrics
            .for_lane(lane)
            .timeouts
            .fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn lane_diagnostics(&self) -> Vec<RedisLaneDiagnostics> {
        [
            RedisConnectionLane::Fast,
            RedisConnectionLane::Stream,
            RedisConnectionLane::BlockingStream,
            RedisConnectionLane::Admin,
        ]
        .into_iter()
        .map(|lane| {
            let metrics = self.metrics.for_lane(lane);
            RedisLaneDiagnostics {
                lane: lane.as_str(),
                command_errors: metrics.errors.load(Ordering::Relaxed),
                command_timeouts: metrics.timeouts.load(Ordering::Relaxed),
            }
        })
        .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RedisLaneDiagnostics {
    pub lane: &'static str,
    pub command_errors: u64,
    pub command_timeouts: u64,
}

#[derive(Default)]
struct RedisConnectionMetrics {
    fast: RedisLaneMetrics,
    stream: RedisLaneMetrics,
    blocking_stream: RedisLaneMetrics,
    admin: RedisLaneMetrics,
}

impl RedisConnectionMetrics {
    fn for_lane(&self, lane: RedisConnectionLane) -> &RedisLaneMetrics {
        match lane {
            RedisConnectionLane::Fast => &self.fast,
            RedisConnectionLane::Stream => &self.stream,
            RedisConnectionLane::BlockingStream => &self.blocking_stream,
            RedisConnectionLane::Admin => &self.admin,
        }
    }
}

#[derive(Default)]
struct RedisLaneMetrics {
    errors: AtomicU64,
    timeouts: AtomicU64,
}

fn connection_manager_config(
    command_timeout_ms: Option<u64>,
) -> redis::aio::ConnectionManagerConfig {
    let mut config = redis::aio::ConnectionManagerConfig::new();
    if let Some(timeout_ms) = command_timeout_ms {
        config = config.set_connection_timeout(Duration::from_millis(timeout_ms));
    }
    config
}

async fn connect_blocking_stream_lanes(
    client: &RedisClient,
    command_timeout_ms: Option<u64>,
    requested_lanes: Option<usize>,
) -> Result<Vec<RedisManagedConnection>, DataLayerError> {
    let lane_count = blocking_stream_lane_count(requested_lanes)?;
    let mut lanes = Vec::with_capacity(lane_count);
    for _ in 0..lane_count {
        lanes.push(
            connect_lane(
                client,
                connection_manager_config(command_timeout_ms),
                RedisConnectionLane::BlockingStream,
                command_timeout_ms,
            )
            .await?,
        );
    }
    Ok(lanes)
}

fn blocking_stream_lane_count(requested_lanes: Option<usize>) -> Result<usize, DataLayerError> {
    if matches!(requested_lanes, Some(0)) {
        return Err(DataLayerError::InvalidConfiguration(
            "runtime redis blocking_stream_lanes must be positive".to_string(),
        ));
    }

    let default_lanes = default_blocking_stream_lane_count();
    Ok(requested_lanes
        .map(|lanes| lanes.max(default_lanes))
        .unwrap_or(default_lanes)
        .clamp(1, MAX_BLOCKING_STREAM_LANES_CAP))
}

fn default_blocking_stream_lane_count() -> usize {
    std::thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(DEFAULT_BLOCKING_STREAM_LANES_FALLBACK)
        .clamp(
            DEFAULT_BLOCKING_STREAM_LANES_FALLBACK,
            DEFAULT_BLOCKING_STREAM_LANES_CAP,
        )
}

async fn connect_lane(
    client: &RedisClient,
    config: redis::aio::ConnectionManagerConfig,
    lane: RedisConnectionLane,
    command_timeout_ms: Option<u64>,
) -> Result<RedisManagedConnection, DataLayerError> {
    let connect = client.get_connection_manager_with_config(config);
    let result = if let Some(timeout_ms) = command_timeout_ms {
        match tokio::time::timeout(Duration::from_millis(timeout_ms), connect).await {
            Ok(result) => result,
            Err(_) => {
                return Err(DataLayerError::TimedOut(format!(
                    "runtime redis {} lane connection exceeded {}ms timeout",
                    lane.as_str(),
                    timeout_ms
                )));
            }
        }
    } else {
        connect.await
    };
    result.map_err(|err| {
        DataLayerError::Redis(format!(
            "failed to initialize runtime redis {} lane: {err}",
            lane.as_str()
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::{
        blocking_stream_lane_count, default_blocking_stream_lane_count, RedisClientConfig,
        RedisClientFactory, MAX_BLOCKING_STREAM_LANES_CAP,
    };

    #[test]
    fn factory_builds_lazy_client_from_valid_config() {
        let config = RedisClientConfig {
            url: "redis://127.0.0.1/0".to_string(),
            key_prefix: Some("aether".to_string()),
        };
        let factory = RedisClientFactory::new(config.clone()).expect("factory should build");

        assert_eq!(factory.config(), &config);
        let _client = factory
            .connect_lazy()
            .expect("lazy redis client should build");
    }

    #[test]
    fn blocking_stream_lane_count_uses_requested_as_floor() {
        let default_lanes = default_blocking_stream_lane_count();

        assert_eq!(
            blocking_stream_lane_count(None).expect("default lanes"),
            default_lanes
        );
        assert_eq!(
            blocking_stream_lane_count(Some(1)).expect("requested below default"),
            default_lanes
        );
        assert_eq!(
            blocking_stream_lane_count(Some(default_lanes + 1)).expect("requested above default"),
            default_lanes + 1
        );
        assert_eq!(
            blocking_stream_lane_count(Some(MAX_BLOCKING_STREAM_LANES_CAP + 1))
                .expect("requested above cap"),
            MAX_BLOCKING_STREAM_LANES_CAP
        );
        assert!(blocking_stream_lane_count(Some(0)).is_err());
    }
}
