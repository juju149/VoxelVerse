//! Streaming pre-warm: gives the spawn view a bounded terrain seed before
//! gameplay starts.
//!
//! The streamer continues progressively during gameplay.  Startup only waits
//! long enough for initial coverage, which prevents the first frame from
//! absorbing the full quadtree and mesh queue cost.

use super::Renderer;
use crate::world_streaming::StreamingView;
use std::time::{Duration, Instant};
use vv_world::PlanetData;

const MAX_DURATION: Duration = Duration::from_millis(1_500);
const MIN_COVERAGE_DURATION: Duration = Duration::from_millis(250);

impl<'a> Renderer<'a> {
    /// Total mesh jobs still in flight for the current streaming view.
    pub fn streaming_pending(&self) -> usize {
        self.pending_chunks.len() + self.pending_lods.len()
    }

    /// Run the streaming pipeline briefly, refreshing the loading screen
    /// between ticks.  This method intentionally returns before every required
    /// tile is ready; per-frame scheduler budgets handle the rest.
    ///
    /// The progress callback runs `render_loading` itself; we pass `&mut
    /// self` through so the closure can drive the GPU without fighting the
    /// borrow checker.
    pub fn prewarm_until_idle(
        &mut self,
        planet: &PlanetData,
        view: StreamingView,
        mut on_progress: impl FnMut(&mut Renderer<'a>, f32, &str),
    ) {
        self.update_view(view, planet);
        let initial = self.streaming_pending().max(1);
        let start = Instant::now();
        loop {
            let remaining = self.streaming_pending();
            let elapsed = start.elapsed();
            let has_coverage = self.has_active_scene_chunks();
            if remaining == 0
                || elapsed > MAX_DURATION
                || (has_coverage && elapsed >= MIN_COVERAGE_DURATION)
            {
                break;
            }
            let progress = 1.0 - (remaining as f32 / initial as f32);
            on_progress(self, progress.clamp(0.0, 0.99), "Pré-chargement terrain");
            std::thread::sleep(Duration::from_millis(2));
            self.update_view(view, planet);
        }
        on_progress(self, 1.0, "Terrain prêt");
    }
}
