use crate::{
    BlockAction, BlockActionIntent, BlockInteraction, BlockSelection, BlockSelectionMode,
    BlockSoundKind, Controller, CursorCapture, GameActionResult, GameFeedbackEvent, Hotbar,
    HotbarNotice, Inventory, MiningFeedback, MiningState, MiningStrikeInput, Player,
};
use vv_pack_compiler::{CompiledSoundKind, ItemId, LootRegistry};
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
                .map(|block| block_sound(block.sound_kind))
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
                .map(|block| block_sound(block.sound_kind))
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
    let edit = BlockInteraction::apply(BlockAction::Mine(ctx.coord), ctx.planet);
    if edit.dirty_chunks.is_empty() {
        result.push_hotbar_notice(HotbarNotice::ProtectedBlock);
        result.push_redraw();
        return result;
    }

    if ctx.drops_enabled {
        if let Some(block) = ctx.planet.block(ctx.voxel) {
            let mut inventory_full = false;
            for (item_id, count) in roll_block_drops(&block.drops_key, ctx.loot) {
                if collect_drop(item_id, count, ctx.planet, ctx.hotbar, ctx.inventory) {
                    inventory_full = true;
                }
            }
            if inventory_full {
                result.push_hotbar_notice(HotbarNotice::Full);
            }
        }
    }
    result.push_dirty_chunks(edit.dirty_chunks);
    result.push_redraw();
    result
}

fn collect_drop(
    item_id: ItemId,
    count: u32,
    planet: &PlanetData,
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
) -> bool {
    let item = planet.item(item_id);
    let max_stack = item.map(|i| i.stack_size.0).unwrap_or(99);
    !hotbar.add(item_id, count, max_stack) && !inventory.add(item_id, count, max_stack)
}

fn roll_block_drops(drops_key: &str, loot: &LootRegistry) -> Vec<(ItemId, u32)> {
    match loot.get_by_key(drops_key) {
        Some(table) => table.roll(|| 0.0),
        None => Vec::new(),
    }
}

fn block_sound(kind: CompiledSoundKind) -> BlockSoundKind {
    match kind {
        CompiledSoundKind::None => BlockSoundKind::None,
        CompiledSoundKind::Grass => BlockSoundKind::Grass,
        CompiledSoundKind::Stone => BlockSoundKind::Stone,
        CompiledSoundKind::Wood => BlockSoundKind::Wood,
        CompiledSoundKind::Sand => BlockSoundKind::Sand,
        CompiledSoundKind::Snow => BlockSoundKind::Snow,
        CompiledSoundKind::Dirt => BlockSoundKind::Dirt,
    }
}
