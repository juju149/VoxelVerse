//! Runtime item stack — the fundamental unit of inventory storage.
//!
//! An `ItemStack` pairs an `ItemId` with a quantity.  All inventory
//! slots (hotbar, player inventory, container) use `ItemStack`.
//!
//! The `ItemId` is a compact runtime identifier assigned by `ItemRegistry`
//! during content compilation.  It is **not** a `VoxelId`; items and blocks
//! are separate concepts.  A block item (e.g. "dirt") carries an `ItemId`
//! whose `gameplay` field is `PlaceBlock { block_key }`, which the gameplay
//! layer resolves to a `VoxelId` via `BlockRegistry` at placement time.

pub use vv_pack_compiler::ItemId;

/// A counted stack of items occupying one inventory slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ItemStack {
    pub item_id: ItemId,
    pub quantity: u32,
}

impl ItemStack {
    pub fn new(item_id: ItemId, quantity: u32) -> Self {
        Self { item_id, quantity }
    }

    pub fn single(item_id: ItemId) -> Self {
        Self { item_id, quantity: 1 }
    }

    /// Returns `true` if this stack can absorb `other` (same item, quantity fits).
    pub fn can_merge_with(self, other: Self, max_stack: u32) -> bool {
        self.item_id == other.item_id && self.quantity < max_stack
    }

    /// Try to add `amount` to this stack, capped at `max_stack`.
    /// Returns how many were actually added.
    pub fn try_add(&mut self, amount: u32, max_stack: u32) -> u32 {
        let space = max_stack.saturating_sub(self.quantity);
        let added = amount.min(space);
        self.quantity += added;
        added
    }
}

