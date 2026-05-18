use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct RunningStats {
    pub count: u64,
    pub min: f32,
    pub avg: f32,
    pub max: f32,
    pub p50: f32,
    pub p95: f32,
    pub p99: f32,
}

impl RunningStats {
    pub fn from_samples(samples: &[f32]) -> Self {
        if samples.is_empty() {
            return Self::default();
        }

        let mut sorted = samples.to_vec();
        sorted.sort_by(|a, b| a.total_cmp(b));
        let sum: f32 = sorted.iter().sum();
        Self {
            count: sorted.len() as u64,
            min: sorted[0],
            avg: sum / sorted.len() as f32,
            max: sorted[sorted.len() - 1],
            p50: percentile_sorted(&sorted, 0.50),
            p95: percentile_sorted(&sorted, 0.95),
            p99: percentile_sorted(&sorted, 0.99),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct RollingWindowStats {
    pub one_second: RunningStats,
    pub five_seconds: RunningStats,
    pub thirty_seconds: RunningStats,
}

fn percentile_sorted(sorted: &[f32], percentile: f32) -> f32 {
    debug_assert!(!sorted.is_empty());
    if sorted.len() == 1 {
        return sorted[0];
    }
    let clamped = percentile.clamp(0.0, 1.0);
    let index = ((sorted.len() - 1) as f32 * clamped).ceil() as usize;
    sorted[index.min(sorted.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::RunningStats;

    #[test]
    fn running_stats_reports_min_avg_max_and_percentiles() {
        let stats = RunningStats::from_samples(&[10.0, 20.0, 30.0, 40.0, 50.0]);
        assert_eq!(stats.count, 5);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.avg, 30.0);
        assert_eq!(stats.max, 50.0);
        assert_eq!(stats.p50, 30.0);
        assert_eq!(stats.p95, 50.0);
        assert_eq!(stats.p99, 50.0);
    }
}
