//! Streaming pre-warm: blocks gameplay startup until every mesh job that
//! the current view requires has been built and uploaded.
//!
//! Without this, the first second of play time runs the camera over a
//! quadtree whose LOD tiles are still on rayon worker threads — the planet
//! pops into existence in chunks.  Draining the queue up-front means the
//! player never sees a void.

use super::Renderer;
use crate::lod_streaming::StreamingView;
use std::time::{Duration, Instant};
use vv_world::PlanetData;

/// Hard cap on the pre-warm phase.  If meshing somehow stalls the loading
/// screen always finishes within this budget so the player isn't stuck.
const MAX_DURATION: Duration = Duration::from_secs(30);

impl<'a> Renderer<'a> {
    /// Total mesh jobs still in flight for the current streaming view.
    pub fn streaming_pending(&self) -> usize {
        self.pending_chunks.len() + self.pending_lods.len()
    }

    /// Block on the streaming pipeline until every required mesh has been
    /// built and uploaded, refreshing the loading screen between ticks.
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
            if remaining == 0 || start.elapsed() > MAX_DURATION {
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
