use glam::Vec3;
use vv_core::BlockId as VoxelId;
use vv_planet::CoordSystem;
use vv_registry::{CompiledContent, CompiledDrops, CompiledLootPool, ItemId};
use vv_world_runtime::PlanetData;

use crate::{placement, DroppedItem, InteractionTarget, Inventory, ItemStack, MiningState};

#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerIntent {
    pub mine_held: bool,
    pub place_pressed: bool,
    pub hotbar_delta: i32,
    pub hotbar_slot: Option<usize>,
    pub toggle_inventory: bool,
}

#[derive(Debug, Clone, Default)]
pub struct GameFrameEvents {
    pub changed_blocks: Vec<VoxelId>,
    pub collected: Vec<ItemStack>,
    pub placement_failed: bool,
}

#[derive(Debug, Clone)]
pub struct PlayerGameplayState {
    pub inventory: Inventory,
    pub selected_hotbar_slot: usize,
    pub mining: MiningState,
    pub target: Option<InteractionTarget>,
    pub interaction_reach: f32,
    pub inventory_open: bool,
    pub dropped_items: Vec<DroppedItem>,
    pub pickup_notice_timer: f32,
    pub placement_blocked_timer: f32,
}

impl PlayerGameplayState {
    const BASE_BREAK_SPEED: f32 = 1.0;
    const PICKUP_RADIUS: f32 = 2.5;
    const NOTICE_SECONDS: f32 = 1.2;

    pub fn new(interaction_reach: f32) -> Self {
        Self {
            inventory: Inventory::player_default(),
            selected_hotbar_slot: 0,
            mining: MiningState::idle(),
            target: None,
            interaction_reach,
            inventory_open: false,
            dropped_items: Vec::new(),
            pickup_notice_timer: 0.0,
            placement_blocked_timer: 0.0,
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Vec3,
        target: Option<InteractionTarget>,
        placement_target: Option<VoxelId>,
        intent: PlayerIntent,
        planet: &mut PlanetData,
        content: &CompiledContent,
    ) -> GameFrameEvents {
        self.target = target.filter(|target| target.distance <= self.interaction_reach);
        self.pickup_notice_timer = (self.pickup_notice_timer - dt).max(0.0);
        self.placement_blocked_timer = (self.placement_blocked_timer - dt).max(0.0);

        if intent.toggle_inventory {
            self.inventory_open = !self.inventory_open;
            self.mining.reset();
        }

        if let Some(slot) = intent.hotbar_slot {
            self.selected_hotbar_slot = slot.min(self.inventory.hotbar_len().saturating_sub(1));
        }
        if intent.hotbar_delta != 0 {
            self.select_hotbar_delta(intent.hotbar_delta);
        }

        let mut events = GameFrameEvents::default();
        self.collect_nearby_items(player_pos, content, &mut events);

        if self.inventory_open {
            self.mining.reset();
            return events;
        }

        if intent.place_pressed {
            if !self.try_place(placement_target, planet, content, &mut events) {
                self.placement_blocked_timer = Self::NOTICE_SECONDS * 0.5;
                events.placement_failed = true;
            }
        }

        if intent.mine_held {
            self.update_mining(dt, planet, content, &mut events);
        } else {
            self.mining.reset();
        }

        events
    }

    fn select_hotbar_delta(&mut self, delta: i32) {
        let len = self.inventory.hotbar_len() as i32;
        if len <= 0 {
            self.selected_hotbar_slot = 0;
            return;
        }
        let current = self.selected_hotbar_slot as i32;
        self.selected_hotbar_slot = (current + delta).rem_euclid(len) as usize;
    }

    fn update_mining(
        &mut self,
        dt: f32,
        planet: &mut PlanetData,
        content: &CompiledContent,
        events: &mut GameFrameEvents,
    ) {
        let Some(target) = self.target else {
            self.mining.reset();
            return;
        };
        if planet.has_core && target.block.layer < planet.core_protection_layers {
            self.mining.reset();
            return;
        }
        let Some(block_id) = planet.block_at(target.block) else {
            self.mining.reset();
            return;
        };
        let Some(block) = content.blocks.get(block_id) else {
            self.mining.reset();
            return;
        };

        if self.mining.advance(
            target.block,
            block.mining.hardness,
            dt,
            Self::BASE_BREAK_SPEED,
        ) {
            let drop_position = block_center(target.block, planet.resolution);
            let drops = resolve_drops(block.drops.clone(), content);
            planet.remove_block(target.block);
            for stack in drops {
                self.dropped_items
                    .push(DroppedItem::new(stack, drop_position));
            }
            events.changed_blocks.push(target.block);
            self.mining.reset();
        }
    }

    fn try_place(
        &mut self,
        placement_target: Option<VoxelId>,
        planet: &mut PlanetData,
        content: &CompiledContent,
        events: &mut GameFrameEvents,
    ) -> bool {
        let Some(place_id) = placement_target else {
            return false;
        };
        if !placement::can_place_block(planet, place_id) {
            return false;
        }
        let Some(block) = placement::selected_placeable_block(
            &self.inventory,
            self.selected_hotbar_slot,
            content,
        ) else {
            return false;
        };
        let Some(removed) = self
            .inventory
            .remove_from_slot(self.selected_hotbar_slot, 1)
        else {
            return false;
        };
        if removed.count != 1 {
            return false;
        }

        planet.add_block(place_id, block);
        events.changed_blocks.push(place_id);
        self.mining.reset();
        true
    }

    fn collect_nearby_items(
        &mut self,
        player_pos: Vec3,
        content: &CompiledContent,
        events: &mut GameFrameEvents,
    ) {
        let mut kept = Vec::with_capacity(self.dropped_items.len());
        for mut drop in self.dropped_items.drain(..) {
            if drop.position.distance(player_pos) > Self::PICKUP_RADIUS {
                kept.push(drop);
                continue;
            }

            let original = drop.stack;
            match self.inventory.insert_stack(drop.stack, content) {
                Some(remaining) => {
                    let collected_count = original.count.saturating_sub(remaining.count);
                    if collected_count > 0 {
                        events
                            .collected
                            .push(ItemStack::new(original.item, collected_count));
                        self.pickup_notice_timer = Self::NOTICE_SECONDS;
                    }
                    drop.stack = remaining;
                    kept.push(drop);
                }
                None => {
                    events.collected.push(original);
                    self.pickup_notice_timer = Self::NOTICE_SECONDS;
                }
            }
        }
        self.dropped_items = kept;
    }
}

fn resolve_drops(drops: CompiledDrops, content: &CompiledContent) -> Vec<ItemStack> {
    match drops {
        CompiledDrops::None => Vec::new(),
        CompiledDrops::Inline(pools) => resolve_pools(&pools),
        CompiledDrops::Table(table) => content
            .loot_tables
            .get(table)
            .map(|table| resolve_pools(&table.pools))
            .unwrap_or_default(),
    }
}

fn resolve_pools(pools: &[CompiledLootPool]) -> Vec<ItemStack> {
    let mut stacks = Vec::new();
    for pool in pools {
        let Some(entry) = pool.entries.iter().max_by_key(|entry| entry.weight) else {
            continue;
        };
        let rolls = pool.rolls.max(1) + pool.bonus_rolls;
        let count = entry.count_min.max(0) as u32;
        if count == 0 {
            continue;
        }
        stacks.push(ItemStack::new(entry.item, count * rolls));
    }
    stacks
}

fn block_center(id: VoxelId, resolution: u32) -> Vec3 {
    let mut center = Vec3::ZERO;
    for u in [0, 1] {
        for v in [0, 1] {
            for l in [0, 1] {
                center += CoordSystem::get_vertex_pos(
                    id.face,
                    id.u + u,
                    id.v + v,
                    id.layer + l,
                    resolution,
                );
            }
        }
    }
    center / 8.0
}

#[allow(dead_code)]
fn _assert_item_id_copy(item: ItemId) -> ItemId {
    item
}
