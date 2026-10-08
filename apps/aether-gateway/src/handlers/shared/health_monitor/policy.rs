use serde::Serialize;

pub(super) const POLICY_VERSION: &str = "service-health-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum HealthStatus {
    Healthy,
    Degraded,
    Unavailable,
    Unknown,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct HealthPolicy {
    pub version: &'static str,
    pub minimum_samples: u64,
    pub healthy_threshold: f64,
    pub degraded_threshold: f64,
}

impl Default for HealthPolicy {
    fn default() -> Self {
        Self {
            version: POLICY_VERSION,
            minimum_samples: 20,
            healthy_threshold: 0.99,
            degraded_threshold: 0.95,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct HealthRatio {
    pub numerator: u64,
    pub denominator: u64,
    pub value: Option<f64>,
}

impl HealthRatio {
    pub fn new(numerator: u64, denominator: u64) -> Self {
        Self {
            numerator,
            denominator,
            value: (denominator > 0).then(|| numerator as f64 / denominator as f64),
        }
    }
}

impl HealthPolicy {
    pub fn status(&self, successes: u64, failures: u64, unknown: u64) -> HealthStatus {
        let samples = successes.saturating_add(failures);
        if samples < self.minimum_samples {
            return HealthStatus::Unknown;
        }
        if unknown > 0 {
            let best_possible_rate =
                successes.saturating_add(unknown) as f64 / samples.saturating_add(unknown) as f64;
            return if best_possible_rate < self.degraded_threshold {
                HealthStatus::Unavailable
            } else {
                HealthStatus::Unknown
            };
        }
        let ratio = successes as f64 / samples as f64;
        if ratio >= self.healthy_threshold {
            HealthStatus::Healthy
        } else if ratio >= self.degraded_threshold {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unavailable
        }
    }
}

pub(super) fn overall_status(statuses: impl Iterator<Item = HealthStatus>) -> HealthStatus {
    let mut result = HealthStatus::Healthy;
    let mut count = 0;
    for status in statuses {
        count += 1;
        match status {
            HealthStatus::Unavailable => return HealthStatus::Unavailable,
            HealthStatus::Degraded => result = HealthStatus::Degraded,
            HealthStatus::Unknown if result == HealthStatus::Healthy => {
                result = HealthStatus::Unknown
            }
            _ => {}
        }
    }
    if count == 0 {
        HealthStatus::Unknown
    } else {
        result
    }
}
