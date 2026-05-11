use crate::voxel::VoxelId;

pub const HOTBAR_SLOT_COUNT: usize = 9;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HotbarSlot {
    pub voxel: VoxelId,
    pub quantity: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HotbarNotice {
    EmptySlot,
    Full,
    InvalidPlacement,
    ProtectedBlock,
}

impl HotbarNotice {
    pub fn text(self) -> &'static str {
        match self {
            HotbarNotice::EmptySlot => "Case vide",
            HotbarNotice::Full => "Hotbar pleine",
            HotbarNotice::InvalidPlacement => "Placement impossible",
            HotbarNotice::ProtectedBlock => "Bloc protege",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Hotbar {
    slots: [Option<HotbarSlot>; HOTBAR_SLOT_COUNT],
    selected: usize,
    notice: Option<HotbarNotice>,
    notice_seconds: f32,
}

impl Hotbar {
    const NOTICE_DURATION_SECONDS: f32 = 1.6;

    pub fn new() -> Self {
        Self {
            slots: [None; HOTBAR_SLOT_COUNT],
            selected: 0,
            notice: None,
            notice_seconds: 0.0,
        }
    }

    pub fn slots(&self) -> &[Option<HotbarSlot>; HOTBAR_SLOT_COUNT] {
        &self.slots
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected_slot(&self) -> Option<HotbarSlot> {
        self.slots[self.selected]
    }

    pub fn selected_voxel(&self) -> Option<VoxelId> {
        self.selected_slot().map(|slot| slot.voxel)
    }

    pub fn select(&mut self, index: usize) {
        if index < HOTBAR_SLOT_COUNT {
            self.selected = index;
        }
    }

    pub fn select_offset(&mut self, delta: i32) {
        let len = HOTBAR_SLOT_COUNT as i32;
        self.selected = (self.selected as i32 + delta).rem_euclid(len) as usize;
    }

    pub fn can_accept(&self, voxel: VoxelId) -> bool {
        voxel != VoxelId::AIR
            && self.slots.iter().any(|slot| match slot {
                Some(slot) => slot.voxel == voxel,
                None => true,
            })
    }

    pub fn add(&mut self, voxel: VoxelId) -> bool {
        if voxel == VoxelId::AIR {
            return false;
        }

        if let Some(slot) = self
            .slots
            .iter_mut()
            .flatten()
            .find(|slot| slot.voxel == voxel)
        {
            slot.quantity = slot.quantity.saturating_add(1);
            self.clear_notice();
            return true;
        }

        if let Some(slot) = self.slots.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(HotbarSlot { voxel, quantity: 1 });
            self.clear_notice();
            return true;
        }

        self.show_notice(HotbarNotice::Full);
        false
    }

    pub fn consume_selected(&mut self) -> Option<VoxelId> {
        let slot = self.slots[self.selected].as_mut()?;
        let voxel = slot.voxel;
        slot.quantity = slot.quantity.saturating_sub(1);
        if slot.quantity == 0 {
            self.slots[self.selected] = None;
        }
        self.clear_notice();
        Some(voxel)
    }

    pub fn show_notice(&mut self, notice: HotbarNotice) {
        self.notice = Some(notice);
        self.notice_seconds = Self::NOTICE_DURATION_SECONDS;
    }

    pub fn update(&mut self, dt: f32) {
        self.notice_seconds = (self.notice_seconds - dt).max(0.0);
        if self.notice_seconds <= 0.0 {
            self.notice = None;
        }
    }

    pub fn notice_text(&self) -> Option<&'static str> {
        self.notice.map(HotbarNotice::text)
    }

    fn clear_notice(&mut self) {
        self.notice = None;
        self.notice_seconds = 0.0;
    }
}

impl Default for Hotbar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{Hotbar, HOTBAR_SLOT_COUNT};
    use crate::voxel::VoxelId;

    #[test]
    fn repeated_blocks_stack_in_first_matching_slot() {
        let mut hotbar = Hotbar::new();

        assert!(hotbar.add(VoxelId::new(4)));
        assert!(hotbar.add(VoxelId::new(4)));

        assert_eq!(hotbar.slots()[0].unwrap().quantity, 2);
        assert!(hotbar.slots()[1].is_none());
    }

    #[test]
    fn full_hotbar_rejects_new_block_type() {
        let mut hotbar = Hotbar::new();
        for i in 0..HOTBAR_SLOT_COUNT {
            assert!(hotbar.add(VoxelId::new((i + 1) as u16)));
        }

        assert!(!hotbar.can_accept(VoxelId::new(99)));
        assert!(!hotbar.add(VoxelId::new(99)));
    }

    #[test]
    fn consuming_last_block_empties_selected_slot() {
        let mut hotbar = Hotbar::new();
        hotbar.add(VoxelId::new(7));

        assert_eq!(hotbar.consume_selected(), Some(VoxelId::new(7)));
        assert!(hotbar.selected_slot().is_none());
    }
}
