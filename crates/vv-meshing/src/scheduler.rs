/// Per-frame budgets that govern how aggressively the scheduler dispatches
/// and uploads jobs.  Upload paths apply three ceilings simultaneously
/// (count, bytes, time) so a single oversized chunk cannot spike the frame
/// even if the chunk-count budget still allows another upload.
#[derive(Clone, Copy, Debug)]
pub struct SchedulerBudget {
    /// Max mesh upload operations per frame (voxel chunks).
    pub upload_voxel: usize,
    /// Max new rayon jobs dispatched per frame (voxel chunks).
    pub dispatch_voxel: usize,
    /// Max simultaneous in-flight voxel jobs.
    pub max_pending_voxel: usize,
    /// Max mesh upload operations per frame (LOD tiles).
    pub upload_lod: usize,
    /// Max new rayon jobs dispatched per frame (LOD tiles).
    pub dispatch_lod: usize,
    /// Max simultaneous in-flight LOD jobs.
    pub max_pending_lod: usize,
    /// Cumulative byte ceiling on uploads per frame (voxel + LOD combined).
    /// Picked at roughly 4–6 average chunks worth, so one giant chunk caps
    /// the frame instead of streaming six in a row.
    pub upload_bytes_per_frame: usize,
    /// Wall-clock ceiling on the upload phase, in milliseconds.  Guards
    /// against GPU driver pauses (e.g. allocator stalls) snowballing into a
    /// visible hitch.
    pub upload_time_budget_ms: f32,
}

impl Default for SchedulerBudget {
    fn default() -> Self {
        Self {
            upload_voxel: 8,
            dispatch_voxel: 8,
            max_pending_voxel: 48,
            upload_lod: 12,
            dispatch_lod: 14,
            max_pending_lod: 32,
            upload_bytes_per_frame: 16 * 1024 * 1024,
            upload_time_budget_ms: 6.0,
        }
    }
}

/// Cumulative upload state tracked per frame by the renderer and fed back
/// into the scheduler to decide whether another upload may proceed.
#[derive(Clone, Copy, Debug, Default)]
pub struct UploadBudgetState {
    pub count: usize,
    pub bytes: usize,
    pub elapsed_ms: f32,
}

/// Lightweight per-frame statistics produced by the scheduler.
/// Intended for the diagnostics overlay.
#[derive(Clone, Copy, Debug, Default)]
pub struct SchedulerStats {
    pub dispatched_voxel: usize,
    pub dispatched_lod: usize,
    pub uploaded_voxel: usize,
    pub uploaded_lod: usize,
    pub pending_voxel: usize,
    pub pending_lod: usize,
}

/// Stateless helper that enforces per-frame dispatch and upload budgets.
///
/// The renderer owns the actual queues and channels; this struct only
/// encapsulates the budget logic so it can be tested independently and
/// later replaced with a more sophisticated scheduler without touching
/// the renderer's GPU upload paths.
pub struct MeshScheduler {
    pub budget: SchedulerBudget,
}

impl MeshScheduler {
    pub fn new(budget: SchedulerBudget) -> Self {
        Self { budget }
    }

    /// Returns `true` if a new voxel job can be dispatched this frame
    /// (i.e. `dispatched_so_far < dispatch_voxel` and `pending < max_pending_voxel`).
    pub fn can_dispatch_voxel(&self, dispatched: usize, pending: usize) -> bool {
        dispatched < self.budget.dispatch_voxel && pending < self.budget.max_pending_voxel
    }

    /// Returns `true` if another voxel mesh can be uploaded to GPU this frame.
    /// Gated by count, total byte volume and wall-clock time so any single
    /// dimension can stop the upload loop.
    pub fn can_upload_voxel(&self, state: &UploadBudgetState) -> bool {
        self.within_upload_envelope(state, self.budget.upload_voxel)
    }

    /// Returns `true` if a new LOD job can be dispatched this frame.
    pub fn can_dispatch_lod(&self, dispatched: usize, pending: usize) -> bool {
        dispatched < self.budget.dispatch_lod && pending < self.budget.max_pending_lod
    }

    /// Returns `true` if another LOD mesh can be uploaded to GPU this frame.
    pub fn can_upload_lod(&self, state: &UploadBudgetState) -> bool {
        self.within_upload_envelope(state, self.budget.upload_lod)
    }

    fn within_upload_envelope(&self, state: &UploadBudgetState, count_ceiling: usize) -> bool {
        state.count < count_ceiling
            && state.bytes < self.budget.upload_bytes_per_frame
            && state.elapsed_ms < self.budget.upload_time_budget_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(count: usize, bytes: usize, elapsed_ms: f32) -> UploadBudgetState {
        UploadBudgetState {
            count,
            bytes,
            elapsed_ms,
        }
    }

    #[test]
    fn dispatch_allowed_within_budget() {
        let s = MeshScheduler::new(SchedulerBudget::default());
        assert!(s.can_dispatch_voxel(0, 0));
        assert!(s.can_dispatch_voxel(3, 15));
    }

    #[test]
    fn dispatch_blocked_when_dispatch_limit_reached() {
        let s = MeshScheduler::new(SchedulerBudget::default());
        let dispatch = s.budget.dispatch_voxel;
        assert!(!s.can_dispatch_voxel(dispatch, 0));
    }

    #[test]
    fn dispatch_blocked_when_pending_limit_reached() {
        let s = MeshScheduler::new(SchedulerBudget::default());
        let pending = s.budget.max_pending_voxel;
        assert!(!s.can_dispatch_voxel(0, pending));
    }

    #[test]
    fn upload_allowed_within_budget() {
        let s = MeshScheduler::new(SchedulerBudget::default());
        assert!(s.can_upload_voxel(&state(3, 0, 0.0)));
    }

    #[test]
    fn upload_blocked_at_count_limit() {
        let s = MeshScheduler::new(SchedulerBudget::default());
        let limit = s.budget.upload_voxel;
        assert!(!s.can_upload_voxel(&state(limit, 0, 0.0)));
    }

    #[test]
    fn upload_blocked_when_bytes_exceed_budget() {
        let s = MeshScheduler::new(SchedulerBudget::default());
        let bytes = s.budget.upload_bytes_per_frame;
        assert!(!s.can_upload_voxel(&state(0, bytes, 0.0)));
    }

    #[test]
    fn upload_blocked_when_time_exceeds_budget() {
        let s = MeshScheduler::new(SchedulerBudget::default());
        let ms = s.budget.upload_time_budget_ms;
        assert!(!s.can_upload_voxel(&state(0, 0, ms)));
    }

    #[test]
    fn lod_budget_independent_from_voxel_budget() {
        let s = MeshScheduler::new(SchedulerBudget::default());
        assert!(s.can_dispatch_lod(0, 0));
        let dispatch = s.budget.dispatch_lod;
        assert!(!s.can_dispatch_lod(dispatch, 0));
        let limit = s.budget.upload_lod;
        assert!(!s.can_upload_lod(&state(limit, 0, 0.0)));
    }

    #[test]
    fn custom_budget_respected() {
        let budget = SchedulerBudget {
            dispatch_voxel: 1,
            ..Default::default()
        };
        let s = MeshScheduler::new(budget);
        assert!(s.can_dispatch_voxel(0, 0));
        assert!(!s.can_dispatch_voxel(1, 0));
    }
}
