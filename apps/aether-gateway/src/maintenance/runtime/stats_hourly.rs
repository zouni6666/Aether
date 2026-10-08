use aether_data::{DataLayerError, StatsHourlyAggregationInput, StatsHourlyAggregationSummary};
use chrono::Utc;

use crate::data::GatewayDataState;

use super::{stats_hourly_aggregation_target_hour, system_config_bool};

pub(super) async fn perform_stats_hourly_aggregation_once(
    data: &GatewayDataState,
) -> Result<Option<StatsHourlyAggregationSummary>, DataLayerError> {
    if !data.has_stats_hourly_aggregation_backend() {
        return Ok(None);
    }
    if !system_config_bool(data, "enable_stats_aggregation", true).await? {
        return Ok(None);
    }

    let now_utc = Utc::now();
    data.aggregate_stats_hourly(&StatsHourlyAggregationInput {
        target_hour_utc: stats_hourly_aggregation_target_hour(now_utc),
        aggregated_at: now_utc,
    })
    .await
}

pub(super) async fn perform_overview_rebuild_once(
    data: &GatewayDataState,
) -> Result<usize, DataLayerError> {
    if !data.has_stats_hourly_aggregation_backend()
        || !system_config_bool(data, "enable_stats_aggregation", true).await?
    {
        return Ok(0);
    }
    let now_utc = Utc::now();
    data.rebuild_overview_buckets(&StatsHourlyAggregationInput {
        target_hour_utc: stats_hourly_aggregation_target_hour(now_utc),
        aggregated_at: now_utc,
    })
    .await
}
