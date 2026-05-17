use crate::HotbarNotice;
use vv_voxel::SurfaceChunkKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlockSoundKind {
    #[default]
    None,
    Grass,
    Stone,
    Wood,
    Sand,
    Snow,
    Dirt,
}

#[derive(Debug, Clone, Copy)]
pub enum GameFeedbackEvent {
    ToolSwing {
        strength: f32,
    },
    BlockHit {
        sound_kind: BlockSoundKind,
        strength: f32,
    },
    BlockBreak {
        sound_kind: BlockSoundKind,
        strength: f32,
    },
    BlockPlace {
        sound_kind: BlockSoundKind,
    },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CursorCapture {
    Grab,
    Release,
}

pub struct GameActionResult {
    pub feedback: Vec<GameFeedbackEvent>,
    pub dirty_chunks: Vec<SurfaceChunkKey>,
    pub hotbar_notices: Vec<HotbarNotice>,
    pub cursor_capture: Option<CursorCapture>,
    pub needs_redraw: bool,
}

impl GameActionResult {
    pub fn none() -> Self {
        Self {
            feedback: Vec::new(),
            dirty_chunks: Vec::new(),
            hotbar_notices: Vec::new(),
            cursor_capture: None,
            needs_redraw: false,
        }
    }

    pub fn push_redraw(&mut self) {
        self.needs_redraw = true;
    }

    pub fn push_feedback(&mut self, event: GameFeedbackEvent) {
        self.feedback.push(event);
    }

    pub fn push_hotbar_notice(&mut self, notice: HotbarNotice) {
        self.hotbar_notices.push(notice);
    }

    pub fn push_dirty_chunks(&mut self, chunks: Vec<SurfaceChunkKey>) {
        self.dirty_chunks.extend(chunks);
    }

    pub fn request_cursor_capture(&mut self, capture: CursorCapture) {
        self.cursor_capture = Some(capture);
    }

    pub fn extend(&mut self, other: GameActionResult) {
        self.feedback.extend(other.feedback);
        self.dirty_chunks.extend(other.dirty_chunks);
        self.hotbar_notices.extend(other.hotbar_notices);
        self.cursor_capture = other.cursor_capture.or(self.cursor_capture);
        self.needs_redraw |= other.needs_redraw;
    }
}

impl Default for GameActionResult {
    fn default() -> Self {
        Self::none()
    }
}

#[cfg(test)]
mod tests {
    use super::{CursorCapture, GameActionResult, GameFeedbackEvent};
    use crate::HotbarNotice;
    use vv_voxel::SurfaceChunkKey;

    #[test]
    fn result_starts_empty() {
        let result = GameActionResult::none();

        assert!(result.feedback.is_empty());
        assert!(result.dirty_chunks.is_empty());
        assert!(result.hotbar_notices.is_empty());
        assert_eq!(result.cursor_capture, None);
        assert!(!result.needs_redraw);
    }

    #[test]
    fn redraw_is_explicit_state() {
        let mut result = GameActionResult::none();

        result.push_redraw();

        assert!(result.needs_redraw);
    }

    #[test]
    fn empty_dirty_chunks_are_noop() {
        let mut result = GameActionResult::none();

        result.push_dirty_chunks(vec![]);

        assert!(result.dirty_chunks.is_empty());
    }

    #[test]
    fn dirty_chunks_accumulate() {
        let mut result = GameActionResult::none();
        let key = SurfaceChunkKey {
            face: 0,
            u_idx: 0,
            v_idx: 0,
        };

        result.push_dirty_chunks(vec![key]);

        assert_eq!(result.dirty_chunks, vec![key]);
    }

    #[test]
    fn cursor_capture_is_recorded() {
        let mut result = GameActionResult::none();

        result.request_cursor_capture(CursorCapture::Grab);

        assert_eq!(result.cursor_capture, Some(CursorCapture::Grab));
    }

    #[test]
    fn notices_are_recorded_without_ui_state_mutation() {
        let mut result = GameActionResult::none();

        result.push_hotbar_notice(HotbarNotice::EmptySlot);

        assert_eq!(result.hotbar_notices, vec![HotbarNotice::EmptySlot]);
    }

    #[test]
    fn feedback_events_accumulate() {
        let mut result = GameActionResult::none();

        result.push_feedback(GameFeedbackEvent::ToolSwing { strength: 0.5 });
        result.push_feedback(GameFeedbackEvent::ToolSwing { strength: 1.0 });

        assert_eq!(result.feedback.len(), 2);
    }

    #[test]
    fn extending_merges_all_outputs() {
        let mut left = GameActionResult::none();
        let mut right = GameActionResult::none();
        right.push_redraw();
        right.request_cursor_capture(CursorCapture::Grab);
        right.push_hotbar_notice(HotbarNotice::Full);

        left.extend(right);

        assert!(left.needs_redraw);
        assert_eq!(left.cursor_capture, Some(CursorCapture::Grab));
        assert_eq!(left.hotbar_notices, vec![HotbarNotice::Full]);
    }
}
