use crate::hotbar::HotbarSlot;
use crate::item_stack::{ItemId, ItemStack};
use std::collections::HashMap;

pub const INVENTORY_COLS: usize = 9;
pub const INVENTORY_ROWS: usize = 4;
pub const INVENTORY_SIZE: usize = INVENTORY_COLS * INVENTORY_ROWS;

#[derive(Clone, Debug)]
pub struct Inventory {
    slots: [Option<HotbarSlot>; INVENTORY_SIZE],
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            slots: [None; INVENTORY_SIZE],
        }
    }

    pub fn slots(&self) -> &[Option<HotbarSlot>; INVENTORY_SIZE] {
        &self.slots
    }

    pub fn slot(&self, index: usize) -> Option<HotbarSlot> {
        self.slots.get(index).copied().flatten()
    }

    pub fn set(&mut self, index: usize, slot: Option<HotbarSlot>) -> bool {
        if index >= INVENTORY_SIZE {
            return false;
        }
        self.slots[index] = slot;
        true
    }

    pub fn take(&mut self, index: usize) -> Option<HotbarSlot> {
        if index >= INVENTORY_SIZE {
            return None;
        }
        self.slots[index].take()
    }

    /// Stack `item_id` into the first matching slot (up to `max_stack`) or
    /// insert into the first empty slot. Returns `true` on success.
    pub fn add(&mut self, item_id: ItemId, count: u32, max_stack: u32) -> bool {
        let mut remaining = count;

        for slot in self.slots.iter_mut().flatten() {
            if slot.item_id == item_id && slot.quantity < max_stack {
                let added = slot.try_add(remaining, max_stack);
                remaining -= added;
                if remaining == 0 {
                    return true;
                }
            }
        }

        while remaining > 0 {
            if let Some(slot) = self.slots.iter_mut().find(|s| s.is_none()) {
                let batch = remaining.min(max_stack);
                *slot = Some(ItemStack::new(item_id, batch));
                remaining -= batch;
            } else {
                return false;
            }
        }
        true
    }

    /// Stack same-item slots into single entries and sort by ItemId.
    pub fn sort(&mut self) {
        let mut totals: HashMap<ItemId, u32> = HashMap::new();
        for slot in self.slots.iter().flatten() {
            *totals.entry(slot.item_id).or_insert(0) += slot.quantity;
        }
        let mut entries: Vec<(ItemId, u32)> = totals.into_iter().collect();
        entries.sort_by_key(|(item_id, _)| item_id.raw());

        self.slots = [None; INVENTORY_SIZE];
        for (i, (item_id, quantity)) in entries.into_iter().enumerate() {
            if i >= INVENTORY_SIZE {
                break;
            }
            self.slots[i] = Some(ItemStack::new(item_id, quantity));
        }
    }

    /// Total mass-style weight indicator: sum of all quantities.
    pub fn total_count(&self) -> u32 {
        self.slots
            .iter()
            .flatten()
            .map(|slot| slot.quantity)
            .sum()
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

/// Identifies any slot the player can drag from / drop into.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlotRef {
    Hotbar(usize),
    Inventory(usize),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item_stack::ItemId;

    const MAX: u32 = 99;
    fn id(n: u32) -> ItemId { ItemId::from_raw(n) }

    #[test]
    fn add_stacks_same_item_into_one_slot() {
        let mut inv = Inventory::new();
        assert!(inv.add(id(4), 1, MAX));
        assert!(inv.add(id(4), 1, MAX));
        assert_eq!(inv.slot(0).unwrap().quantity, 2);
        assert!(inv.slot(1).is_none());
    }

    #[test]
    fn sort_merges_duplicate_stacks() {
        let mut inv = Inventory::new();
        inv.set(0, Some(ItemStack::new(id(2), 3)));
        inv.set(5, Some(ItemStack::new(id(2), 4)));
        inv.set(10, Some(ItemStack::new(id(1), 1)));
        inv.sort();
        // Sorted by item id: 1 first, then 2 with merged quantity 7.
        assert_eq!(inv.slot(0).unwrap().item_id, id(1));
        assert_eq!(inv.slot(1).unwrap().item_id, id(2));
        assert_eq!(inv.slot(1).unwrap().quantity, 7);
        assert!(inv.slot(2).is_none());
    }

    #[test]
    fn take_empties_slot_and_returns_content() {
        let mut inv = Inventory::new();
        inv.set(3, Some(ItemStack::new(id(9), 5)));
        let taken = inv.take(3).unwrap();
        assert_eq!(taken.quantity, 5);
        assert!(inv.slot(3).is_none());
    }
}

