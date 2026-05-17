use crate::{
    BlockAction, BlockActionIntent, BlockInteraction, BlockSelection, BlockSelectionMode,
    Controller, CursorCapture, GameActionResult, GameFeedbackEvent, Hotbar, HotbarNotice,
    Inventory, MiningFeedback, MiningState, MiningStrikeInput, Player,
};
use vv_pack_compiler::{ItemId, LootRegistry};
use vv_world::{BlockDamageResult, PlanetData};

pub struct PlaceBlockContext<'a> {
    pub controller: &'a mut Controller,
    pub player: &'a Player,
    pub planet: &'a mut PlanetData,
    pub hotbar: &'a mut Hotbar,
    pub view_width: f32,
    pub view_height: f32,
}

pub struct MineBlockContext<'a> {
    pub controller: &'a Controller,
    pub planet: &'a mut PlanetData,
    pub hotbar: &'a mut Hotbar,
    pub inventory: &'a mut Inventory,
    pub mining: &'a mut MiningState,
    pub loot: &'a LootRegistry,
}

struct BreakBlockContext<'a> {
    planet: &'a mut PlanetData,
    hotbar: &'a mut Hotbar,
    inventory: &'a mut Inventory,
    loot: &'a LootRegistry,
    coord: vv_voxel::VoxelCoord,
    voxel: vv_voxel::VoxelId,
    drops_enabled: bool,
}

pub fn place_block(ctx: PlaceBlockContext<'_>) -> GameActionResult {
    let mut result = GameActionResult::none();

    let selected = ctx.hotbar.selected_item_id();
    let active_voxel = match selected.and_then(|id| ctx.planet.resolve_item_voxel(id)) {
        Some(voxel) => Some(voxel),
        None => {
            if selected.is_none() {
                result.push_hotbar_notice(HotbarNotice::EmptySlot);
            } else {
                result.push_hotbar_notice(HotbarNotice::InvalidPlacement);
            }
            result.push_redraw();
            return result;
        }
    };

    let ray = ctx
        .controller
        .view_ray(ctx.player, ctx.view_width, ctx.view_height);
    let placement = BlockSelection::trace(
        ray,
        ctx.controller.interaction_reach(),
        ctx.planet,
        BlockSelectionMode::Placement,
    )
    .map(|(id, _)| id);
    if placement.is_none() {
        result.push_hotbar_notice(HotbarNotice::InvalidPlacement);
        result.push_redraw();
        return result;
    }

    if let Some(action) = BlockInteraction::resolve(
        BlockActionIntent::Place,
        ctx.controller.cursor_id,
        placement,
        active_voxel,
    ) {
        let edit = BlockInteraction::apply(action, ctx.planet);
        if !edit.dirty_chunks.is_empty() {
            ctx.hotbar.consume_selected();
            let sound_kind = active_voxel
                .and_then(|voxel| ctx.planet.block(voxel))
                .map(|block| block.sound_kind)
                .unwrap_or_default();
            result.push_feedback(GameFeedbackEvent::BlockPlace { sound_kind });
            result.push_dirty_chunks(edit.dirty_chunks);
            result.push_redraw();
        }
    } else if ctx.controller.cursor_id.is_none() && ctx.controller.first_person {
        result.request_cursor_capture(CursorCapture::Grab);
    }

    result
}

pub fn mine_block(ctx: MineBlockContext<'_>, dt: f32) -> GameActionResult {
    let mut result = GameActionResult::none();

    let coord = ctx.controller.cursor_id;
    let voxel = coord.map(|c| ctx.planet.get_voxel(c));
    let block = voxel.and_then(|v| ctx.planet.block(v));
    let feedback = ctx.mining.tick(MiningStrikeInput {
        coord,
        voxel,
        block,
        selected_item: ctx.hotbar.selected_item_id(),
        items: ctx.planet.items(),
        dt,
        wants_mining: true,
    });

    match feedback {
        MiningFeedback::None => {}
        MiningFeedback::Blocked => {
            result.push_hotbar_notice(HotbarNotice::ProtectedBlock);
            result.push_redraw();
        }
        MiningFeedback::Hit {
            coord,
            voxel,
            damage,
            break_threshold,
            impact_strength,
            drops_enabled,
            ..
        } => {
            let sound_kind = ctx
                .planet
                .block(voxel)
                .map(|block| block.sound_kind)
                .unwrap_or_default();
            result.push_feedback(GameFeedbackEvent::ToolSwing {
                strength: impact_strength,
            });
            match ctx
                .planet
                .apply_block_damage(coord, damage, break_threshold)
            {
                BlockDamageResult::Unchanged => {}
                BlockDamageResult::Damaged { .. } => {
                    result.push_feedback(GameFeedbackEvent::BlockHit {
                        sound_kind,
                        strength: impact_strength,
                    });
                    result.push_redraw();
                }
                BlockDamageResult::Broken => {
                    let break_result = break_block(BreakBlockContext {
                        planet: ctx.planet,
                        hotbar: ctx.hotbar,
                        inventory: ctx.inventory,
                        loot: ctx.loot,
                        coord,
                        voxel,
                        drops_enabled,
                    });
                    result.extend(break_result);
                    result.push_feedback(GameFeedbackEvent::BlockBreak {
                        sound_kind,
                        strength: impact_strength,
                    });
                }
            }
        }
        MiningFeedback::Broken { .. } => {}
    }

    result
}

fn break_block(ctx: BreakBlockContext<'_>) -> GameActionResult {
    let mut result = GameActionResult::none();
    let drops = if ctx.drops_enabled {
        ctx.planet
            .block(ctx.voxel)
            .map(|block| roll_block_drops(&block.drops_key, ctx.loot))
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    if !can_collect_drops(&drops, ctx.planet, ctx.hotbar, ctx.inventory) {
        result.push_hotbar_notice(HotbarNotice::Full);
        result.push_redraw();
        return result;
    }

    let edit = BlockInteraction::apply(BlockAction::Mine(ctx.coord), ctx.planet);
    if edit.dirty_chunks.is_empty() {
        result.push_hotbar_notice(HotbarNotice::ProtectedBlock);
        result.push_redraw();
        return result;
    }

    for (item_id, count) in drops {
        let max_stack = max_stack_for_item(item_id, ctx.planet);
        let collected = collect_drop(item_id, count, max_stack, ctx.hotbar, ctx.inventory);
        debug_assert!(collected, "drop capacity was checked before breaking block");
    }
    result.push_dirty_chunks(edit.dirty_chunks);
    result.push_redraw();
    result
}

fn can_collect_drops(
    drops: &[(ItemId, u32)],
    planet: &PlanetData,
    hotbar: &Hotbar,
    inventory: &Inventory,
) -> bool {
    let mut hotbar = hotbar.clone();
    let mut inventory = inventory.clone();
    drops.iter().all(|(item_id, count)| {
        collect_drop(
            *item_id,
            *count,
            max_stack_for_item(*item_id, planet),
            &mut hotbar,
            &mut inventory,
        )
    })
}

fn collect_drop(
    item_id: ItemId,
    count: u32,
    max_stack: u32,
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
) -> bool {
    if hotbar
        .available_capacity(item_id, max_stack)
        .saturating_add(inventory.available_capacity(item_id, max_stack))
        < count
    {
        return false;
    }

    let to_hotbar = count.min(hotbar.available_capacity(item_id, max_stack));
    if to_hotbar > 0 && !hotbar.add(item_id, to_hotbar, max_stack) {
        return false;
    }
    let remaining = count - to_hotbar;
    remaining == 0 || inventory.add(item_id, remaining, max_stack)
}

fn roll_block_drops(drops_key: &str, loot: &LootRegistry) -> Vec<(ItemId, u32)> {
    match loot.get_by_key(drops_key) {
        Some(table) => table.roll(|| 0.0),
        None => Vec::new(),
    }
}

fn max_stack_for_item(item_id: ItemId, planet: &PlanetData) -> u32 {
    planet.item(item_id).map(|i| i.stack_size.0).unwrap_or(99)
}

#[cfg(test)]
mod tests {
    use super::collect_drop;
    use crate::{Hotbar, Inventory, ItemStack, HOTBAR_SLOT_COUNT, INVENTORY_SIZE};
    use vv_pack_compiler::ItemId;

    const MAX_STACK: u32 = 99;

    fn id(raw: u32) -> ItemId {
        ItemId::from_raw(raw)
    }

    #[test]
    fn collecting_drop_splits_between_hotbar_and_inventory() {
        let mut hotbar = Hotbar::new();
        let mut inventory = Inventory::new();
        assert!(hotbar.add(id(1), 98, MAX_STACK));
        for index in 1..HOTBAR_SLOT_COUNT {
            assert!(hotbar.add(id(index as u32 + 10), MAX_STACK, MAX_STACK));
        }

        assert!(collect_drop(
            id(1),
            3,
            MAX_STACK,
            &mut hotbar,
            &mut inventory
        ));

        assert_eq!(hotbar.slots()[0].unwrap().quantity, 99);
        assert_eq!(inventory.slot(0).unwrap().item_id, id(1));
        assert_eq!(inventory.slot(0).unwrap().quantity, 2);
    }

    #[test]
    fn collecting_drop_rejects_full_storage_without_partial_mutation() {
        let mut hotbar = Hotbar::new();
        let mut inventory = Inventory::new();
        for index in 0..HOTBAR_SLOT_COUNT {
            assert!(hotbar.add(id(index as u32 + 10), MAX_STACK, MAX_STACK));
        }
        for index in 0..INVENTORY_SIZE {
            assert!(inventory.set(
                index,
                Some(ItemStack::new(id(index as u32 + 100), MAX_STACK))
            ));
        }
        let hotbar_before = *hotbar.slots();
        let inventory_before = *inventory.slots();

        assert!(!collect_drop(
            id(1),
            1,
            MAX_STACK,
            &mut hotbar,
            &mut inventory
        ));

        assert_eq!(*hotbar.slots(), hotbar_before);
        assert_eq!(*inventory.slots(), inventory_before);
    }
}
