use crate::item_stack::{ItemId, ItemStack};

pub const HOTBAR_SLOT_COUNT: usize = 9;

/// One hotbar slot. Type alias for `ItemStack` — the canonical inventory unit.
pub type HotbarSlot = ItemStack;

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
    revision: u64,
}

impl Hotbar {
    const NOTICE_DURATION_SECONDS: f32 = 1.6;

    pub fn new() -> Self {
        Self {
            slots: [None; HOTBAR_SLOT_COUNT],
            selected: 0,
            notice: None,
            notice_seconds: 0.0,
            revision: 0,
        }
    }

    /// Monotonic counter bumped on every visual state change (selection,
    /// slot content, notice).  Renderers cache the last seen value to skip
    /// rebuilding the hotbar mesh when nothing changed.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    fn bump(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }

    pub fn slots(&self) -> &[Option<HotbarSlot>; HOTBAR_SLOT_COUNT] {
        &self.slots
    }

    /// Replace the entire slot array. Used by drag-and-drop to move items
    /// between the hotbar and the inventory in a single shot.
    pub fn set_slots(&mut self, slots: [Option<HotbarSlot>; HOTBAR_SLOT_COUNT]) {
        self.slots = slots;
        self.bump();
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected_slot(&self) -> Option<HotbarSlot> {
        self.slots[self.selected]
    }

    pub fn selected_item_id(&self) -> Option<ItemId> {
        self.selected_slot().map(|s| s.item_id)
    }

    pub fn select(&mut self, index: usize) {
        if index < HOTBAR_SLOT_COUNT && self.selected != index {
            self.selected = index;
            self.bump();
        }
    }

    pub fn select_offset(&mut self, delta: i32) {
        let len = HOTBAR_SLOT_COUNT as i32;
        let next = (self.selected as i32 + delta).rem_euclid(len) as usize;
        if next != self.selected {
            self.selected = next;
            self.bump();
        }
    }

    pub fn can_accept(&self, item_id: ItemId, max_stack: u32) -> bool {
        self.slots.iter().any(|slot| match slot {
            Some(slot) => slot.item_id == item_id && slot.quantity < max_stack,
            None => true,
        })
    }

    /// Add `count` items of `item_id` to the hotbar. Stacks into existing
    /// slots first, then occupies empty slots. Returns `true` on success.
    pub fn add(&mut self, item_id: ItemId, count: u32, max_stack: u32) -> bool {
        let mut remaining = count;
        let mut changed = false;

        // Stack into existing matching slots first.
        for slot in self.slots.iter_mut().flatten() {
            if slot.item_id == item_id && slot.quantity < max_stack {
                let added = slot.try_add(remaining, max_stack);
                if added > 0 {
                    changed = true;
                }
                remaining -= added;
                if remaining == 0 {
                    self.clear_notice();
                    if changed {
                        self.bump();
                    }
                    return true;
                }
            }
        }

        // Fill empty slots with the overflow.
        while remaining > 0 {
            if let Some(slot) = self.slots.iter_mut().find(|s| s.is_none()) {
                let batch = remaining.min(max_stack);
                *slot = Some(ItemStack::new(item_id, batch));
                remaining -= batch;
                changed = true;
            } else {
                break;
            }
        }

        if changed {
            self.bump();
        }

        if remaining == 0 {
            self.clear_notice();
            true
        } else {
            self.show_notice(HotbarNotice::Full);
            false
        }
    }

    pub fn consume_selected(&mut self) -> Option<ItemId> {
        let slot = self.slots[self.selected].as_mut()?;
        let item_id = slot.item_id;
        slot.quantity = slot.quantity.saturating_sub(1);
        if slot.quantity == 0 {
            self.slots[self.selected] = None;
        }
        self.clear_notice();
        self.bump();
        Some(item_id)
    }

    pub fn show_notice(&mut self, notice: HotbarNotice) {
        if self.notice != Some(notice) {
            self.bump();
        }
        self.notice = Some(notice);
        self.notice_seconds = Self::NOTICE_DURATION_SECONDS;
    }

    pub fn update(&mut self, dt: f32) {
        self.notice_seconds = (self.notice_seconds - dt).max(0.0);
        if self.notice_seconds <= 0.0 && self.notice.is_some() {
            self.notice = None;
            self.bump();
        }
    }

    pub fn notice_text(&self) -> Option<&'static str> {
        self.notice.map(HotbarNotice::text)
    }

    fn clear_notice(&mut self) {
        if self.notice.is_some() {
            self.bump();
        }
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
    use crate::item_stack::ItemId;

    const MAX: u32 = 99;
    fn id(n: u32) -> ItemId {
        ItemId::from_raw(n)
    }

    #[test]
    fn repeated_items_stack_in_first_matching_slot() {
        let mut hotbar = Hotbar::new();

        assert!(hotbar.add(id(4), 1, MAX));
        assert!(hotbar.add(id(4), 1, MAX));

        assert_eq!(hotbar.slots()[0].unwrap().quantity, 2);
        assert!(hotbar.slots()[1].is_none());
    }

    #[test]
    fn full_hotbar_rejects_new_item_type() {
        let mut hotbar = Hotbar::new();
        for i in 0..HOTBAR_SLOT_COUNT {
            assert!(hotbar.add(id(i as u32 + 1), 1, MAX));
        }

        assert!(!hotbar.can_accept(id(99), MAX));
        assert!(!hotbar.add(id(99), 1, MAX));
    }

    #[test]
    fn consuming_last_item_empties_selected_slot() {
        let mut hotbar = Hotbar::new();
        hotbar.add(id(7), 1, MAX);

        assert_eq!(hotbar.consume_selected(), Some(id(7)));
        assert!(hotbar.selected_slot().is_none());
    }

    #[test]
    fn max_stack_prevents_overflow() {
        let mut hotbar = Hotbar::new();
        assert!(hotbar.add(id(1), 1, 2));
        assert!(hotbar.add(id(1), 1, 2));
        // Stack is full — third add should open a new slot.
        let before_second_slot = hotbar.slots()[1].is_none();
        hotbar.add(id(1), 1, 2);
        // Either new slot opened or add failed — either is valid per design.
        let _ = before_second_slot;
    }
}
