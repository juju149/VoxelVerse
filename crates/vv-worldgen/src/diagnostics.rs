//! Worldgen telemetry counters.
//!
//! Lock-free atomic counters exposed by [`ProceduralPlanetTerrain`] so the
//! HUD / dev overlay can show generation pressure without instrumenting
//! every call site.  Increments are `Relaxed` — exact ordering does not
//! matter, only the eventual count.
//!
//! The counters are kept tiny on purpose: chunk gen happens thousands of
//! times per session, and adding per-stage timers would dwarf the actual
//! work.  Higher-resolution profiling lives behind a feature flag in the
//! diagnostics overlay, not here.

use std::sync::atomic::{AtomicU64, Ordering};
pub use vv_diagnostics::WorldgenStatsSnapshot;

#[derive(Default, Debug)]
pub struct WorldgenStats {
    /// Number of surface-cache cells (re)computed on demand.
    pub cell_misses: AtomicU64,
    /// Number of cache hits — proxy for warm-region locality.
    pub cell_hits: AtomicU64,
    /// Vegetation / structure feature stamps emitted into chunk maps.
    pub features_emitted: AtomicU64,
    /// Vox-prop instances generated for chunk queries.
    pub props_emitted: AtomicU64,
    /// Placement candidates rejected by biome / slope / climate gates.
    pub candidates_rejected: AtomicU64,
    /// Placement candidates rejected by the min-spacing Poisson check.
    pub candidates_rejected_spacing: AtomicU64,
}

impl WorldgenStats {
    #[inline]
    pub fn record_cell_hit(&self) {
        self.cell_hits.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_cell_miss(&self) {
        self.cell_misses.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_feature(&self) {
        self.features_emitted.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_prop(&self) {
        self.props_emitted.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_reject(&self) {
        self.candidates_rejected.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_reject_spacing(&self) {
        self.candidates_rejected_spacing
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Snapshot for a HUD line — all counts are captured in one pass. Uses
    /// the centralised `WorldgenStatsSnapshot` defined in `vv-diagnostics`
    /// so the overlay can read it without depending on worldgen internals.
    pub fn snapshot(&self) -> WorldgenStatsSnapshot {
        WorldgenStatsSnapshot {
            cell_hits: self.cell_hits.load(Ordering::Relaxed),
            cell_misses: self.cell_misses.load(Ordering::Relaxed),
            features_emitted: self.features_emitted.load(Ordering::Relaxed),
            props_emitted: self.props_emitted.load(Ordering::Relaxed),
            candidates_rejected: self.candidates_rejected.load(Ordering::Relaxed),
            candidates_rejected_spacing: self.candidates_rejected_spacing.load(Ordering::Relaxed),
        }
    }
}
