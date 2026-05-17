use vv_meshing::SchedulerBudget;

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderBudgetConfig {
    pub mesh_scheduler: SchedulerBudget,
}

#[cfg(test)]
mod tests {
    use super::RenderBudgetConfig;

    #[test]
    fn default_render_budget_keeps_uploads_bounded() {
        let budget = RenderBudgetConfig::default();

        assert!(budget.mesh_scheduler.dispatch_voxel > 0);
        assert!(budget.mesh_scheduler.upload_voxel > 0);
        assert!(budget.mesh_scheduler.max_pending_voxel >= budget.mesh_scheduler.dispatch_voxel);
        assert!(budget.mesh_scheduler.upload_bytes_per_frame > 0);
        assert!(budget.mesh_scheduler.upload_time_budget_ms > 0.0);
    }
}
