pub(crate) use crate::admin_api::{
    admin_stats_bad_request_response, parse_bounded_u32, round_to, AdminStatsTimeRange,
    AdminStatsUsageFilter,
};

/// Sample chronologically sorted points by index, keeping both ends of the window.
/// If only one point fits, prefer the latest point.
pub(crate) fn sample_interval_timeline<T>(items: Vec<T>, limit: usize) -> Vec<T> {
    if items.len() <= limit {
        return items;
    }
    if limit == 0 {
        return Vec::new();
    }
    if limit == 1 {
        return items.into_iter().last().into_iter().collect();
    }

    let last_index = (items.len() - 1) as u128;
    let last_sample = (limit - 1) as u128;
    let mut sample = 0u128;
    items
        .into_iter()
        .enumerate()
        .filter_map(|(index, item)| {
            if index as u128 == sample * last_index / last_sample {
                sample += 1;
                Some(item)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::sample_interval_timeline;

    #[test]
    fn interval_timeline_sampling_preserves_budget_order_and_boundaries() {
        for count in 0..128 {
            for limit in 0..=130 {
                let points = sample_interval_timeline((0..count).collect(), limit);
                assert_eq!(points.len(), count.min(limit));
                assert!(points.windows(2).all(|pair| pair[0] < pair[1]));
                if let Some(last) = points.last() {
                    assert_eq!(*last, count - 1);
                }
                if points.len() > 1 {
                    assert_eq!(points[0], 0);
                    let gaps = points.windows(2).map(|pair| pair[1] - pair[0]);
                    assert!(gaps.clone().max().unwrap() - gaps.min().unwrap() <= 1);
                }
            }
        }
    }
}
