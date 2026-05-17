/// Runtime debug flags, active only when `dev_mode` is enabled in `GameRuntime`.
///
/// These replace the old `is_wireframe`, `show_collisions`, and `freeze_culling`
/// fields that were incorrectly living on `Controller` in `vv-gameplay`.
pub(super) struct DevState {
    /// Render terrain and player body as wireframe.
    pub(super) is_wireframe: bool,
    /// Render collision geometry as a line mesh overlay.
    pub(super) show_collisions: bool,
    /// Freeze the frustum used for chunk culling (debug view-culling).
    pub(super) freeze_culling: bool,
}

impl DevState {
    pub(super) fn new() -> Self {
        Self {
            is_wireframe: false,
            show_collisions: false,
            freeze_culling: false,
        }
    }
}
