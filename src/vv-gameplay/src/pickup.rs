use glam::Vec3;

use crate::ItemStack;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DroppedItem {
    pub stack: ItemStack,
    pub position: Vec3,
}

impl DroppedItem {
    pub fn new(stack: ItemStack, position: Vec3) -> Self {
        Self { stack, position }
    }
}
