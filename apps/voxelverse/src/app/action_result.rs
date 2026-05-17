use vv_gameplay::{
    CursorCapture, GameActionResult, GameFeedbackEvent, HotbarNotice, PlanetResizeIntent,
};
use vv_voxel::SurfaceChunkKey;

/// UI-layer events produced by gameplay actions.
///
/// These are applied by the UI router after the action returns — gameplay code
/// never calls hotbar or UI methods directly.
#[derive(Debug, Clone, Copy)]
pub(super) enum UiEvent {
    /// Display a transient notice on the hotbar (e.g. "Empty slot", "Inventory full").
    HotbarNotice(HotbarNotice),
}

/// Result returned by gameplay actions (place_block, mine_block, …).
///
/// Callers apply commands, forward feedback, and dispatch UI events.
/// Actions never touch the renderer, audio engine, window, or hotbar UI directly.
pub(super) struct ActionResult {
    pub(super) feedback: Vec<GameFeedbackEvent>,
    pub(super) commands: Vec<FrameCommand>,
    pub(super) ui_events: Vec<UiEvent>,
}

impl ActionResult {
    pub(super) fn none() -> Self {
        Self {
            feedback: Vec::new(),
            commands: Vec::new(),
            ui_events: Vec::new(),
        }
    }

    pub(super) fn from_gameplay(result: GameActionResult) -> Self {
        let mut action = Self {
            feedback: result.feedback,
            commands: Vec::new(),
            ui_events: result
                .hotbar_notices
                .into_iter()
                .map(UiEvent::HotbarNotice)
                .collect(),
        };
        if result.needs_redraw {
            action.push_redraw();
        }
        if !result.dirty_chunks.is_empty() {
            action.push_refresh_chunks(result.dirty_chunks);
        }
        match result.cursor_capture {
            Some(CursorCapture::Grab) => action.push_grab_cursor(),
            Some(CursorCapture::Release) => action.push_release_cursor(),
            None => {}
        }
        action
    }

    pub(super) fn push_redraw(&mut self) {
        self.commands.push(FrameCommand::Redraw);
    }

    pub(super) fn push_grab_cursor(&mut self) {
        self.commands.push(FrameCommand::GrabCursor);
    }

    pub(super) fn push_release_cursor(&mut self) {
        self.commands.push(FrameCommand::ReleaseCursor);
    }

    pub(super) fn push_refresh_chunks(&mut self, chunks: Vec<SurfaceChunkKey>) {
        if !chunks.is_empty() {
            self.commands.push(FrameCommand::RefreshDirtyChunks(chunks));
        }
    }
}

/// Commands the frame driver applies after each action or tick.
///
/// Every command in this list has a single, unambiguous application site in
/// `frame_commands::apply_frame_commands`. No command is applied inside the
/// routers or actions themselves.
#[derive(Debug)]
pub(super) enum FrameCommand {
    /// Request an immediate window repaint.
    Redraw,
    /// Grab and lock the OS cursor for first-person mode.
    GrabCursor,
    /// Release the OS cursor (UI mode).
    ReleaseCursor,
    /// Submit dirty chunk keys to the renderer for mesh rebuild.
    RefreshDirtyChunks(Vec<SurfaceChunkKey>),

    // ── Dev / debug commands (only dispatched when dev_mode is active) ────────
    /// Force a full world streaming reload after a planet resize.
    ForceReloadWorld,
    /// Toggle the engine debug stats overlay.
    ToggleDebugPage,
    /// Toggle colour-only (texture-less) render mode.
    ToggleColorOnlyMode,
    /// Toggle triplanar grain on the terrain shader.
    ToggleTriplanarGrain,
    /// Cycle through PCF shadow quality levels (Low → Medium → High → Low).
    CyclePcfQuality,
    /// Apply a resize intent to the active planet.
    ResizePlanet(PlanetResizeIntent),
    /// Toggle wireframe render mode (terrain + player body).
    ToggleWireframe,
    /// Toggle collision debug mesh overlay.
    ToggleCollisions,
    /// Freeze or unfreeze the frustum used for culling.
    ToggleCullingFreeze,
}
