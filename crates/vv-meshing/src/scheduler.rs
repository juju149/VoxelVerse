/// Per-frame budgets that govern how aggressively the scheduler dispatches
/// and uploads jobs.  Lowering these reduces GPU stalls at the cost of
/// slower streaming; raising them speeds up loading at the cost of frame spikes.
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
}

impl Default for SchedulerBudget {
    fn default() -> Self {
        Self {
            upload_voxel: 4,
            dispatch_voxel: 4,
            max_pending_voxel: 16,
            upload_lod: 8,
            dispatch_lod: 10,
            max_pending_lod: 24,
        }
    }
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
    pub fn can_upload_voxel(&self, uploaded: usize) -> bool {
        uploaded < self.budget.upload_voxel
    }

    /// Returns `true` if a new LOD job can be dispatched this frame.
    pub fn can_dispatch_lod(&self, dispatched: usize, pending: usize) -> bool {
        dispatched < self.budget.dispatch_lod && pending < self.budget.max_pending_lod
    }

    /// Returns `true` if another LOD mesh can be uploaded to GPU this frame.
    pub fn can_upload_lod(&self, uploaded: usize) -> bool {
        uploaded < self.budget.upload_lod
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_allowed_within_budget() {
        let s = MeshScheduler::new(SchedulerBudget::default());
        assert!(s.can_dispatch_voxel(0, 0));
        assert!(s.can_dispatch_voxel(3, 15));
    }

    #[test]
    fn dispatch_blocked_when_dispatch_limit_reached() {
        let s = MeshScheduler::new(SchedulerBudget::default());
        // budget.dispatch_voxel = 4
        assert!(!s.can_dispatch_voxel(4, 0));
    }

    #[test]
    fn dispatch_blocked_when_pending_limit_reached() {
        let s = MeshScheduler::new(SchedulerBudget::default());
        // budget.max_pending_voxel = 16
        assert!(!s.can_dispatch_voxel(0, 16));
    }

    #[test]
    fn upload_allowed_within_budget() {
        let s = MeshScheduler::new(SchedulerBudget::default());
        assert!(s.can_upload_voxel(3));
    }

    #[test]
    fn upload_blocked_at_limit() {
        let s = MeshScheduler::new(SchedulerBudget::default());
        // budget.upload_voxel = 4
        assert!(!s.can_upload_voxel(4));
    }

    #[test]
    fn lod_budget_independent_from_voxel_budget() {
        let s = MeshScheduler::new(SchedulerBudget::default());
        // Can dispatch LOD even if voxel budget exhausted
        assert!(s.can_dispatch_lod(0, 0));
        assert!(!s.can_dispatch_lod(10, 0)); // dispatch_lod = 10
        assert!(!s.can_upload_lod(8)); // upload_lod = 8
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
