use vv_audio::SoundKind;
use vv_voxel::SurfaceChunkKey;

/// Result returned by gameplay actions (place_block, mine_block).
/// Callers apply commands and forward feedback events — actions never touch
/// the renderer, audio engine, or window directly.
pub(super) struct ActionResult {
    pub(super) feedback: Vec<FeedbackEvent>,
    pub(super) commands: Vec<FrameCommand>,
}

impl ActionResult {
    pub(super) fn none() -> Self {
        Self {
            feedback: Vec::new(),
            commands: Vec::new(),
        }
    }

    pub(super) fn push_redraw(&mut self) {
        self.commands.push(FrameCommand::Redraw);
    }

    pub(super) fn push_grab_cursor(&mut self) {
        self.commands.push(FrameCommand::GrabCursor);
    }

    pub(super) fn push_feedback(&mut self, event: FeedbackEvent) {
        self.feedback.push(event);
    }

    pub(super) fn push_refresh_chunks(&mut self, chunks: Vec<SurfaceChunkKey>) {
        if !chunks.is_empty() {
            self.commands.push(FrameCommand::RefreshDirtyChunks(chunks));
        }
    }
}

/// Audio and visual feedback that an action produced.
/// Processed by feedback_router after the action returns.
pub(super) enum FeedbackEvent {
    ToolSwing {
        strength: f32,
    },
    BlockHit {
        sound_kind: SoundKind,
        strength: f32,
    },
    BlockBreak {
        sound_kind: SoundKind,
        strength: f32,
    },
    BlockPlace {
        sound_kind: SoundKind,
    },
}

/// Commands the frame driver applies after every action or tick.
pub(super) enum FrameCommand {
    /// Request an immediate window repaint.
    Redraw,
    /// Grab and lock the cursor for first-person mode.
    GrabCursor,
    /// Submit dirty chunk keys to the renderer for mesh rebuild.
    RefreshDirtyChunks(Vec<SurfaceChunkKey>),
}
