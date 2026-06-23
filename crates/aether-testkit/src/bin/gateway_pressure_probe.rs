use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use aether_testkit::{
    fetch_prometheus_samples, find_metric_value_u64, run_http_load_probe, HttpLoadProbeConfig,
    HttpLoadProbeResponseMode, HttpLoadProbeResult, PrometheusSample,
};
use reqwest::Method;
use serde::Serialize;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
struct Config {
    load: HttpLoadProbeConfig,
    metrics_url: String,
    sample_interval: Duration,
    output_path: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
struct GatewayPressureReport {
    suite: &'static str,
    target_url: String,
    metrics_url: String,
    sample_interval_ms: u64,
    load: HttpLoadProbeResult,
    metrics: GatewayPressureMetricsSummary,
}

#[derive(Debug, Clone, Default, Serialize)]
struct GatewayPressureMetricsSummary {
    samples: usize,
    db_pool_max_checked_out: u64,
    db_pool_min_idle: Option<u64>,
    db_pool_max_size: u64,
    db_pool_max_connections: u64,
    db_pool_max_usage_basis_points: u64,
    db_pool_max_idle_reserve: u64,
    db_pool_pressure_samples: usize,
    gateway_requests_max_in_flight: u64,
    gateway_requests_max_rejected_total: u64,
    gateway_requests_distributed_max_in_flight: u64,
    gateway_requests_distributed_max_rejected_total: u64,
}

impl GatewayPressureMetricsSummary {
    fn observe(&mut self, samples: &[PrometheusSample]) {
        self.samples += 1;
        self.db_pool_max_checked_out = self
            .db_pool_max_checked_out
            .max(metric_max(samples, "database_pool_checked_out_connections"));
        let idle = metric_min(samples, "database_pool_idle_connections");
        self.db_pool_min_idle = match (self.db_pool_min_idle, idle) {
            (Some(current), Some(next)) => Some(current.min(next)),
            (None, Some(next)) => Some(next),
            (current, None) => current,
        };
        self.db_pool_max_size = self
            .db_pool_max_size
            .max(metric_max(samples, "database_pool_size_connections"));
        self.db_pool_max_connections = self
            .db_pool_max_connections
            .max(metric_max(samples, "database_pool_max_connections"));
        self.db_pool_max_usage_basis_points = self
            .db_pool_max_usage_basis_points
            .max(metric_max(samples, "database_pool_usage_basis_points"));
        self.db_pool_max_idle_reserve = self.db_pool_max_idle_reserve.max(metric_max(
            samples,
            "database_pool_idle_reserve_connections",
        ));
        if metric_max(samples, "database_pool_under_maintenance_pressure") > 0 {
            self.db_pool_pressure_samples += 1;
        }
        self.gateway_requests_max_in_flight = self.gateway_requests_max_in_flight.max(
            find_metric_value_u64(
                samples,
                "concurrency_in_flight",
                &[("gate", "gateway_requests")],
            )
            .unwrap_or_default(),
        );
        self.gateway_requests_max_rejected_total = self.gateway_requests_max_rejected_total.max(
            find_metric_value_u64(
                samples,
                "concurrency_rejected_total",
                &[("gate", "gateway_requests")],
            )
            .unwrap_or_default(),
        );
        self.gateway_requests_distributed_max_in_flight =
            self.gateway_requests_distributed_max_in_flight.max(
                find_metric_value_u64(
                    samples,
                    "concurrency_in_flight",
                    &[("gate", "gateway_requests_distributed")],
                )
                .unwrap_or_default(),
            );
        self.gateway_requests_distributed_max_rejected_total =
            self.gateway_requests_distributed_max_rejected_total.max(
                find_metric_value_u64(
                    samples,
                    "concurrency_rejected_total",
                    &[("gate", "gateway_requests_distributed")],
                )
                .unwrap_or_default(),
            );
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_args(std::env::args().skip(1).collect())?;
    let stop = Arc::new(AtomicBool::new(false));
    let summary = Arc::new(Mutex::new(GatewayPressureMetricsSummary::default()));
    let sampler = spawn_metrics_sampler(
        config.metrics_url.clone(),
        config.sample_interval,
        Arc::clone(&stop),
        Arc::clone(&summary),
    );

    let load = run_http_load_probe(&config.load)
        .await
        .map_err(std::io::Error::other)?;
    stop.store(true, Ordering::Release);
    sampler.await??;

    if let Ok(samples) = fetch_prometheus_samples(&config.metrics_url).await {
        summary.lock().await.observe(&samples);
    }

    let report = GatewayPressureReport {
        suite: "gateway_pressure_probe",
        target_url: config.load.url,
        metrics_url: config.metrics_url,
        sample_interval_ms: config.sample_interval.as_millis() as u64,
        load,
        metrics: Arc::try_unwrap(summary)
            .unwrap_or_else(|_| panic!("metrics summary still referenced"))
            .into_inner(),
    };
    let raw = serde_json::to_string_pretty(&report)?;
    println!("{raw}");
    if let Some(path) = config.output_path.as_ref() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, format!("{raw}\n"))?;
    }
    Ok(())
}

fn spawn_metrics_sampler(
    metrics_url: String,
    interval: Duration,
    stop: Arc<AtomicBool>,
    summary: Arc<Mutex<GatewayPressureMetricsSummary>>,
) -> tokio::task::JoinHandle<Result<(), std::io::Error>> {
    tokio::spawn(async move {
        while !stop.load(Ordering::Acquire) {
            match fetch_prometheus_samples(&metrics_url).await {
                Ok(samples) => summary.lock().await.observe(&samples),
                Err(err) => {
                    eprintln!("gateway pressure probe metrics sample failed: {err}");
                }
            }
            tokio::time::sleep(interval).await;
        }
        Ok(())
    })
}

fn parse_args(args: Vec<String>) -> Result<Config, Box<dyn std::error::Error>> {
    let mut target_url: Option<String> = None;
    let mut warmup_url: Option<String> = None;
    let mut metrics_url: Option<String> = None;
    let mut total_requests: Option<usize> = None;
    let mut concurrency: Option<usize> = None;
    let mut warmup_connections: usize = 0;
    let mut timeout_ms: Option<u64> = None;
    let mut connect_timeout_ms: Option<u64> = None;
    let mut client_shards: Option<usize> = None;
    let mut pool_max_idle_per_host: Option<usize> = None;
    let mut start_ramp_ms: u64 = 0;
    let mut first_body_hold_ms: u64 = 0;
    let mut sample_interval_ms: u64 = 500;
    let mut method = Method::GET;
    let mut headers = BTreeMap::new();
    let mut body: Option<Vec<u8>> = None;
    let mut response_mode = HttpLoadProbeResponseMode::HeadersOnly;
    let mut http1_only = false;
    let mut http2_prior_knowledge = false;
    let mut output_path = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--url" => target_url = Some(next_value(&mut iter, "--url")?),
            "--warmup-url" => warmup_url = Some(next_value(&mut iter, "--warmup-url")?),
            "--metrics-url" => metrics_url = Some(next_value(&mut iter, "--metrics-url")?),
            "--requests" => total_requests = Some(next_value(&mut iter, "--requests")?.parse()?),
            "--concurrency" => concurrency = Some(next_value(&mut iter, "--concurrency")?.parse()?),
            "--warmup-connections" => {
                warmup_connections = next_value(&mut iter, "--warmup-connections")?.parse()?
            }
            "--timeout-ms" => timeout_ms = Some(next_value(&mut iter, "--timeout-ms")?.parse()?),
            "--connect-timeout-ms" => {
                connect_timeout_ms = Some(next_value(&mut iter, "--connect-timeout-ms")?.parse()?)
            }
            "--client-shards" => {
                client_shards = Some(next_value(&mut iter, "--client-shards")?.parse()?)
            }
            "--pool-max-idle-per-host" => {
                pool_max_idle_per_host =
                    Some(next_value(&mut iter, "--pool-max-idle-per-host")?.parse()?)
            }
            "--start-ramp-ms" => {
                start_ramp_ms = next_value(&mut iter, "--start-ramp-ms")?.parse()?
            }
            "--first-body-hold-ms" => {
                first_body_hold_ms = next_value(&mut iter, "--first-body-hold-ms")?.parse()?
            }
            "--http1-only" => http1_only = true,
            "--http2-prior-knowledge" => http2_prior_knowledge = true,
            "--sample-interval-ms" => {
                sample_interval_ms = next_value(&mut iter, "--sample-interval-ms")?.parse()?
            }
            "--method" => {
                method = Method::from_bytes(next_value(&mut iter, "--method")?.as_bytes())?
            }
            "--header" | "-H" => {
                let (name, value) = parse_header_arg(&next_value(&mut iter, "--header")?)?;
                headers.insert(name, value);
            }
            "--body" => body = Some(next_value(&mut iter, "--body")?.into_bytes()),
            "--body-file" => body = Some(std::fs::read(next_value(&mut iter, "--body-file")?)?),
            "--response-mode" => {
                response_mode = parse_response_mode(&next_value(&mut iter, "--response-mode")?)?
            }
            "--output" => output_path = Some(PathBuf::from(next_value(&mut iter, "--output")?)),
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("unknown argument: {other}"),
                )
                .into());
            }
        }
    }

    let mut load = HttpLoadProbeConfig {
        url: target_url.ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing required --url")
        })?,
        total_requests: total_requests.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "missing required --requests",
            )
        })?,
        concurrency: concurrency.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "missing required --concurrency",
            )
        })?,
        method,
        headers,
        body,
        response_mode,
        ..HttpLoadProbeConfig::default()
    };
    load.warmup_url = warmup_url;
    load.warmup_connections = warmup_connections;
    if let Some(timeout_ms) = timeout_ms {
        load.timeout = Duration::from_millis(timeout_ms);
    }
    load.connect_timeout = connect_timeout_ms.map(Duration::from_millis);
    if let Some(client_shards) = client_shards {
        load.client_shards = client_shards;
    }
    load.pool_max_idle_per_host = pool_max_idle_per_host;
    load.start_ramp = Duration::from_millis(start_ramp_ms);
    load.first_body_hold = Duration::from_millis(first_body_hold_ms);
    load.http1_only = http1_only;
    load.http2_prior_knowledge = http2_prior_knowledge;
    load.validate()
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err))?;
    if sample_interval_ms == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "--sample-interval-ms must be positive",
        )
        .into());
    }
    Ok(Config {
        load,
        metrics_url: metrics_url.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "missing required --metrics-url",
            )
        })?,
        sample_interval: Duration::from_millis(sample_interval_ms),
        output_path,
    })
}

fn parse_header_arg(value: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let (name, value) = value
        .split_once(':')
        .or_else(|| value.split_once('='))
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "--header expects `Name: value` or `Name=value`",
            )
        })?;
    let name = name.trim();
    if name.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "--header name cannot be empty",
        )
        .into());
    }
    Ok((name.to_string(), value.trim().to_string()))
}

fn parse_response_mode(
    value: &str,
) -> Result<HttpLoadProbeResponseMode, Box<dyn std::error::Error>> {
    match value.trim().to_ascii_lowercase().as_str() {
        "headers" | "headers-only" | "header" => Ok(HttpLoadProbeResponseMode::HeadersOnly),
        "first-body-byte" | "first-body" | "first-byte" | "first-chunk" => {
            Ok(HttpLoadProbeResponseMode::FirstBodyByte)
        }
        "full" | "full-body" | "body" => Ok(HttpLoadProbeResponseMode::FullBody),
        other => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "unsupported --response-mode {other}; expected headers, first-body-byte, or full"
            ),
        )
        .into()),
    }
}

fn next_value(
    iter: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    iter.next().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("missing value for {flag}"),
        )
        .into()
    })
}

fn metric_max(samples: &[PrometheusSample], metric_name: &str) -> u64 {
    samples
        .iter()
        .filter(|sample| metric_name_matches(&sample.name, metric_name))
        .filter_map(|sample| sample.value.parse::<u64>().ok())
        .max()
        .unwrap_or_default()
}

fn metric_min(samples: &[PrometheusSample], metric_name: &str) -> Option<u64> {
    samples
        .iter()
        .filter(|sample| metric_name_matches(&sample.name, metric_name))
        .filter_map(|sample| sample.value.parse::<u64>().ok())
        .min()
}

fn metric_name_matches(actual: &str, expected: &str) -> bool {
    actual == expected
        || actual
            .rsplit_once('_')
            .map(|(_, suffix)| suffix == expected)
            .unwrap_or(false)
        || actual.ends_with(&format!("_{expected}"))
}

fn print_usage() {
    eprintln!(
        "usage: cargo run -p aether-testkit --bin gateway_pressure_probe -- --url <URL> --metrics-url <URL> --requests <N> --concurrency <N> [--warmup-url <URL>] [--warmup-connections N] [--method GET] [--timeout-ms 30000] [--connect-timeout-ms 10000] [--client-shards 1] [--pool-max-idle-per-host N] [--start-ramp-ms 0] [--first-body-hold-ms 0] [--http1-only | --http2-prior-knowledge] [--sample-interval-ms 500] [-H 'Name: value'] [--body JSON | --body-file path] [--response-mode headers|first-body-byte|full] [--output /tmp/gateway_pressure.json]"
    );
}
