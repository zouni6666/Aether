//! Node-local request concurrency, integrated at lifecycle edges rather than sampled.
//!
//! Minute buckets bound memory independently of traffic volume. A request spanning
//! a bucket/day boundary contributes to both sides, including while no API polls us.
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use aether_data_contracts::repository::usage::UsageDashboardAnalyticsQuery;
use aether_runtime::{AdmissionPermit, AdmissionPermitHealth};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};

const MINUTE_US: i64 = 60_000_000;
const RETAIN_MINUTES: i64 = 48 * 60;

#[derive(Debug, Default)]
struct Minute {
    start_us: i64,
    request_microseconds: u128,
    peak: u64,
}

#[derive(Debug)]
struct History {
    observed_from_us: i64,
    through_us: i64,
    active: u64,
    minutes: VecDeque<Minute>,
}

impl History {
    fn new(now_us: i64) -> Self {
        Self {
            observed_from_us: now_us,
            through_us: now_us,
            active: 0,
            minutes: VecDeque::new(),
        }
    }

    fn minute(&mut self, at_us: i64) -> &mut Minute {
        let start_us = at_us.div_euclid(MINUTE_US) * MINUTE_US;
        if self
            .minutes
            .back()
            .is_none_or(|bucket| bucket.start_us != start_us)
        {
            self.minutes.push_back(Minute {
                start_us,
                ..Minute::default()
            });
        }
        self.minutes.back_mut().expect("minute was inserted")
    }

    fn advance(&mut self, now_us: i64) {
        let now_us = now_us.max(self.through_us);
        let retained_from = (now_us.div_euclid(MINUTE_US) - RETAIN_MINUTES) * MINUTE_US;
        let mut cursor = self.through_us.max(retained_from);
        while cursor < now_us {
            let end = ((cursor.div_euclid(MINUTE_US) + 1) * MINUTE_US).min(now_us);
            let active = self.active;
            let bucket = self.minute(cursor);
            bucket.request_microseconds += u128::from(active) * (end - cursor) as u128;
            bucket.peak = bucket.peak.max(active);
            cursor = end;
        }
        self.through_us = now_us;
        while self
            .minutes
            .front()
            .is_some_and(|bucket| bucket.start_us < retained_from)
        {
            self.minutes.pop_front();
        }
    }

    fn change(&mut self, now_us: i64, entering: bool) {
        self.advance(now_us);
        self.active = if entering {
            self.active.saturating_add(1)
        } else {
            self.active.saturating_sub(1)
        };
        let active = self.active;
        let at_us = self.through_us;
        let bucket = self.minute(at_us);
        bucket.peak = bucket.peak.max(active);
    }

    fn today(&mut self, timezone: &str, now_us: i64) -> Result<Value, String> {
        self.advance(now_us);
        let through = DateTime::from_timestamp_micros(self.through_us)
            .ok_or_else(|| "invalid concurrency observation timestamp".to_string())?;
        let day_start = UsageDashboardAnalyticsQuery {
            timezone: timezone.into(),
        }
        .today_start(through)
        .map_err(|error| error.to_string())?;
        let day_start_us = day_start.timestamp_micros();
        // Current IANA offsets/day boundaries are minute aligned. Refuse to
        // misrepresent an unsupported sub-minute historical boundary as exact.
        if day_start_us.rem_euclid(MINUTE_US) != 0 {
            return Err("concurrency day boundary is not minute aligned".into());
        }
        let observed_from_us = self.observed_from_us.max(day_start_us);
        let duration_us = self.through_us.saturating_sub(observed_from_us);
        let (area, peak) = self
            .minutes
            .iter()
            .filter(|minute| minute.start_us >= day_start_us && minute.start_us <= self.through_us)
            .fold((0u128, self.active), |(area, peak), minute| {
                (area + minute.request_microseconds, peak.max(minute.peak))
            });
        Ok(json!({
            "avg": (duration_us > 0).then(|| area as f64 / duration_us as f64),
            "peak": peak,
            "observed_from": DateTime::from_timestamp_micros(observed_from_us),
            "observed_through": through,
            "scope": "node",
            "measurement": "http_and_responses_websocket_requests",
            "coverage": if self.observed_from_us <= day_start_us { "complete" } else { "partial" },
        }))
    }
}

#[derive(Debug)]
pub(crate) struct RequestActivity {
    started_at: Instant,
    started_at_us: i64,
    history: Mutex<History>,
}

impl Default for RequestActivity {
    fn default() -> Self {
        let started_at = Instant::now();
        let started_at_us = Utc::now().timestamp_micros();
        Self {
            started_at,
            started_at_us,
            history: Mutex::new(History::new(started_at_us)),
        }
    }
}

impl RequestActivity {
    fn now_us(&self) -> i64 {
        self.started_at_us
            .saturating_add(self.started_at.elapsed().as_micros().min(i64::MAX as u128) as i64)
    }

    pub(crate) fn begin(self: &Arc<Self>) -> RequestActivityGuard {
        self.history
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .change(self.now_us(), true);
        RequestActivityGuard {
            activity: Arc::clone(self),
        }
    }

    pub(crate) fn today(&self, timezone: &str) -> Result<Value, String> {
        self.history
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .today(timezone, self.now_us())
    }

    #[cfg(test)]
    pub(crate) fn active(&self) -> u64 {
        self.history.lock().unwrap().active
    }
}

#[derive(Debug)]
pub(crate) struct RequestActivityGuard {
    activity: Arc<RequestActivity>,
}

impl RequestActivityGuard {
    pub(crate) fn into_admission_permit(self) -> AdmissionPermit {
        // This guard observes lifecycle only; it neither limits nor cancels work.
        AdmissionPermit::from_parts(None, Some(self)).expect("activity guard is present")
    }
}

impl AdmissionPermitHealth for RequestActivityGuard {
    fn is_healthy(&self) -> bool {
        true
    }
    fn requires_health_poll(&self) -> bool {
        false
    }
}

impl Drop for RequestActivityGuard {
    fn drop(&mut self) {
        self.activity
            .history
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .change(self.activity.now_us(), false);
    }
}

impl crate::AppState {
    pub(crate) fn today_concurrency(&self, timezone: &str) -> Result<Value, String> {
        self.request_activity.today(timezone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(value: &str) -> i64 {
        DateTime::parse_from_rfc3339(value)
            .unwrap()
            .timestamp_micros()
    }

    #[test]
    fn concurrency_integrates_time_instead_of_averaging_event_samples() {
        let start = at("2026-09-19T00:00:00Z");
        let mut history = History::new(start);
        history.change(start, true);
        history.change(start + 10_000_000, true);
        history.change(start + 20_000_000, false);
        history.change(start + 30_000_000, false);
        let value = history.today("UTC", start + 100_000_000).unwrap();
        assert_eq!(value["avg"], 0.4);
        assert_eq!(value["peak"], 2);
        assert_eq!(value["coverage"], "complete");
    }

    #[test]
    fn concurrency_long_request_crosses_minutes_and_local_midnight() {
        let start = at("2026-09-18T15:59:30Z");
        let mut history = History::new(start);
        history.change(start, true);
        let value = history.today("Asia/Shanghai", start + 150_000_000).unwrap();
        assert_eq!(value["avg"], 1.0);
        assert_eq!(value["peak"], 1);
        assert_eq!(value["observed_from"], "2026-09-18T16:00:00Z");
        assert_eq!(value["coverage"], "complete");
        history.change(start + 150_000_000, false);
        let value = history.today("Asia/Shanghai", start + 270_000_000).unwrap();
        assert_eq!(value["avg"], 0.5);
    }

    #[test]
    fn concurrency_restart_only_claims_the_observed_part_of_the_day() {
        let start = at("2026-09-19T12:00:00Z");
        let mut history = History::new(start);
        let value = history.today("UTC", start + 60_000_000).unwrap();
        assert_eq!(value["avg"], 0.0);
        assert_eq!(value["peak"], 0);
        assert_eq!(value["coverage"], "partial");
        assert_eq!(value["observed_from"], "2026-09-19T12:00:00Z");
        assert!(History::new(start).today("UTC", start).unwrap()["avg"].is_null());
        assert!(history.today("not/a/timezone", start).is_err());
    }

    #[test]
    fn concurrency_does_not_carry_yesterdays_peak_into_today() {
        let start = at("2026-09-18T23:59:30Z");
        let mut history = History::new(start);
        history.change(start, true);
        history.change(start, true);
        history.change(start + 20_000_000, false);
        history.change(start + 30_000_000, false);
        let value = history.today("UTC", start + 90_000_000).unwrap();
        assert_eq!(value["avg"], 0.0);
        assert_eq!(value["peak"], 0);
    }

    #[test]
    fn concurrency_handles_dst_and_bounds_memory_after_a_long_idle_gap() {
        let start = at("2026-10-30T00:00:00Z");
        let mut history = History::new(start);
        history.change(start, true);
        let end = at("2026-11-02T04:30:00Z");
        let value = history.today("America/New_York", end).unwrap();
        assert_eq!(value["avg"], 1.0);
        assert_eq!(value["observed_from"], "2026-11-01T04:00:00Z");
        assert_eq!(value["coverage"], "complete");
        assert!(history.minutes.len() <= RETAIN_MINUTES as usize + 1);
    }

    #[test]
    fn concurrency_permit_clones_share_one_lifecycle() {
        let activity = Arc::new(RequestActivity::default());
        let permit = activity.begin().into_admission_permit();
        let background = permit.clone();
        assert_eq!(activity.active(), 1);
        drop(permit);
        assert_eq!(activity.active(), 1);
        drop(background);
        assert_eq!(activity.active(), 0);
    }
}
