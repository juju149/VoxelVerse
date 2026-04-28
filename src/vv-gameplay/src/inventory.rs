use vv_registry::{CompiledContent, ItemId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemStack {
    pub item: ItemId,
    pub count: u32,
}

impl ItemStack {
    pub fn new(item: ItemId, count: u32) -> Self {
        Self { item, count }
    }

    pub fn is_empty(self) -> bool {
        self.count == 0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Slot {
    pub stack: Option<ItemStack>,
}

#[derive(Debug, Clone)]
pub struct Inventory {
    slots: Vec<Slot>,
    hotbar_len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryMoveError {
    SourceEmpty,
    SlotOutOfRange,
}

impl Inventory {
    pub const DEFAULT_HOTBAR_LEN: usize = 9;
    pub const DEFAULT_MAIN_ROWS: usize = 3;
    pub const DEFAULT_MAIN_COLUMNS: usize = 9;

    pub fn player_default() -> Self {
        Self::new(
            Self::DEFAULT_HOTBAR_LEN + Self::DEFAULT_MAIN_ROWS * Self::DEFAULT_MAIN_COLUMNS,
            Self::DEFAULT_HOTBAR_LEN,
        )
    }

    pub fn new(slot_count: usize, hotbar_len: usize) -> Self {
        assert!(hotbar_len <= slot_count);
        Self {
            slots: vec![Slot::default(); slot_count],
            hotbar_len,
        }
    }

    pub fn slots(&self) -> &[Slot] {
        &self.slots
    }

    pub fn hotbar_slots(&self) -> &[Slot] {
        &self.slots[..self.hotbar_len]
    }

    pub fn hotbar_len(&self) -> usize {
        self.hotbar_len
    }

    pub fn selected_stack(&self, selected_hotbar_slot: usize) -> Option<ItemStack> {
        self.slots
            .get(selected_hotbar_slot)
            .and_then(|slot| slot.stack)
    }

    pub fn insert_stack(
        &mut self,
        stack: ItemStack,
        content: &CompiledContent,
    ) -> Option<ItemStack> {
        if stack.is_empty() {
            return None;
        }

        let max = self.stack_max(stack.item, content) as u32;
        let mut remaining = stack.count;

        for slot in &mut self.slots {
            let Some(existing) = &mut slot.stack else {
                continue;
            };
            if existing.item != stack.item || existing.count >= max {
                continue;
            }
            let room = max - existing.count;
            let moved = remaining.min(room);
            existing.count += moved;
            remaining -= moved;
            if remaining == 0 {
                return None;
            }
        }

        for slot in &mut self.slots {
            if slot.stack.is_some() {
                continue;
            }
            let moved = remaining.min(max);
            slot.stack = Some(ItemStack::new(stack.item, moved));
            remaining -= moved;
            if remaining == 0 {
                return None;
            }
        }

        Some(ItemStack::new(stack.item, remaining))
    }

    pub fn remove_from_slot(&mut self, slot_index: usize, count: u32) -> Option<ItemStack> {
        let slot = self.slots.get_mut(slot_index)?;
        let stack = slot.stack.as_mut()?;
        let removed = count.min(stack.count);
        let item = stack.item;
        stack.count -= removed;
        if stack.count == 0 {
            slot.stack = None;
        }
        Some(ItemStack::new(item, removed))
    }

    pub fn move_or_merge(
        &mut self,
        from: usize,
        to: usize,
        content: &CompiledContent,
    ) -> Result<(), InventoryMoveError> {
        if from >= self.slots.len() || to >= self.slots.len() {
            return Err(InventoryMoveError::SlotOutOfRange);
        }
        let Some(source) = self.slots[from].stack else {
            return Err(InventoryMoveError::SourceEmpty);
        };
        if from == to {
            return Ok(());
        }

        match self.slots[to].stack {
            Some(target) if target.item == source.item => {
                let max = self.stack_max(source.item, content) as u32;
                let room = max.saturating_sub(target.count);
                let moved = source.count.min(room);
                if moved == 0 {
                    self.slots.swap(from, to);
                    return Ok(());
                }
                self.slots[to].stack.as_mut().expect("target stack").count += moved;
                self.remove_from_slot(from, moved);
            }
            _ => self.slots.swap(from, to),
        }
        Ok(())
    }

    pub fn split_half(&mut self, slot_index: usize) -> Option<ItemStack> {
        let slot = self.slots.get_mut(slot_index)?;
        let stack = slot.stack.as_mut()?;
        if stack.count <= 1 {
            return None;
        }
        let removed = stack.count / 2;
        stack.count -= removed;
        Some(ItemStack::new(stack.item, removed))
    }

    fn stack_max(&self, item: ItemId, content: &CompiledContent) -> u8 {
        content
            .items
            .get(item)
            .map(|item| item.stack_max.max(1))
            .unwrap_or(64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vv_registry::{CompiledContent, CompiledItem, CompiledItemKind, ContentKey};

    fn content_with_item(stack_max: u8) -> (CompiledContent, ItemId) {
        let mut content = CompiledContent::default();
        let item = content.items.push(
            ContentKey::new("test", "stone").unwrap(),
            CompiledItem {
                display_key: None,
                stack_max,
                tags: Vec::new(),
                kind: CompiledItemKind::Resource,
            },
        );
        (content, item)
    }

    #[test]
    fn insert_merges_before_using_empty_slots() {
        let (content, item) = content_with_item(64);
        let mut inventory = Inventory::new(2, 1);
        assert!(inventory
            .insert_stack(ItemStack::new(item, 32), &content)
            .is_none());
        assert!(inventory
            .insert_stack(ItemStack::new(item, 40), &content)
            .is_none());

        assert_eq!(inventory.slots()[0].stack.unwrap().count, 64);
        assert_eq!(inventory.slots()[1].stack.unwrap().count, 8);
    }

    #[test]
    fn full_inventory_returns_remaining_stack() {
        let (content, item) = content_with_item(4);
        let mut inventory = Inventory::new(1, 1);
        assert!(inventory
            .insert_stack(ItemStack::new(item, 4), &content)
            .is_none());
        let remaining = inventory
            .insert_stack(ItemStack::new(item, 3), &content)
            .expect("remaining");

        assert_eq!(remaining.count, 3);
    }
}
