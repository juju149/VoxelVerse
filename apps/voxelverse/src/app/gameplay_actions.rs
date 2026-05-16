use crate::app::cursor::grab_cursor;
use crate::app::feedback_router::{route_feedback, sound_kind, AppFeedback};
use vv_audio::AudioEngine;
use vv_gameplay::{
    BlockAction, BlockActionIntent, BlockInteraction, BlockSelection, BlockSelectionMode,
    Controller, Hotbar, HotbarNotice, Inventory, ItemId, MiningFeedback, MiningState,
    MiningStrikeInput, Player,
};
use vv_pack_compiler::LootRegistry;
use vv_render::Renderer;
use vv_world::{BlockDamageResult, PlanetData};

pub(super) struct PlaceBlockContext<'a, 'window> {
    pub(super) renderer: &'a mut Renderer<'window>,
    pub(super) audio: &'a mut AudioEngine,
    pub(super) controller: &'a mut Controller,
    pub(super) player: &'a Player,
    pub(super) planet: &'a mut PlanetData,
    pub(super) hotbar: &'a mut Hotbar,
}

pub(super) struct MineBlockContext<'a, 'window> {
    pub(super) renderer: &'a mut Renderer<'window>,
    pub(super) audio: &'a mut AudioEngine,
    pub(super) controller: &'a Controller,
    pub(super) planet: &'a mut PlanetData,
    pub(super) hotbar: &'a mut Hotbar,
    pub(super) inventory: &'a mut Inventory,
    pub(super) mining: &'a mut MiningState,
    pub(super) loot: &'a LootRegistry,
}

struct BreakBlockContext<'a, 'window> {
    renderer: &'a mut Renderer<'window>,
    planet: &'a mut PlanetData,
    hotbar: &'a mut Hotbar,
    inventory: &'a mut Inventory,
    loot: &'a LootRegistry,
    coord: vv_voxel::VoxelCoord,
    voxel: vv_voxel::VoxelId,
    drops_enabled: bool,
}

pub(super) fn place_block(ctx: PlaceBlockContext<'_, '_>) {
    let selected = ctx.hotbar.selected_item_id();
    let active_voxel = match selected.and_then(|id| ctx.planet.resolve_item_voxel(id)) {
        Some(voxel) => Some(voxel),
        None => {
            if selected.is_none() {
                ctx.hotbar.show_notice(HotbarNotice::EmptySlot);
            } else {
                ctx.hotbar.show_notice(HotbarNotice::InvalidPlacement);
            }
            ctx.renderer.window.request_redraw();
            return;
        }
    };

    let ray = ctx.controller.view_ray(
        ctx.player,
        ctx.renderer.config.width as f32,
        ctx.renderer.config.height as f32,
    );
    let placement = BlockSelection::trace(
        ray,
        ctx.controller.interaction_reach(),
        ctx.planet,
        BlockSelectionMode::Placement,
    )
    .map(|(id, _)| id);
    if placement.is_none() {
        ctx.hotbar.show_notice(HotbarNotice::InvalidPlacement);
        ctx.renderer.window.request_redraw();
        return;
    }

    if let Some(action) = BlockInteraction::resolve(
        BlockActionIntent::Place,
        ctx.controller.cursor_id,
        placement,
        active_voxel,
    ) {
        let edit = BlockInteraction::apply(action, ctx.planet);
        let changed = !edit.dirty_chunks.is_empty();
        if changed {
            ctx.hotbar.consume_selected();
            let sound_kind = active_voxel
                .and_then(|voxel| ctx.planet.content.block(voxel))
                .map(|block| sound_kind(block.sound_kind))
                .unwrap_or_default();
            route_feedback(
                ctx.renderer,
                ctx.audio,
                AppFeedback::BlockPlace { sound_kind },
            );
            ctx.renderer.refresh_dirty_chunks(edit.dirty_chunks);
            ctx.renderer.window.request_redraw();
        }
    } else if ctx.controller.cursor_id.is_none() && ctx.controller.first_person {
        grab_cursor(ctx.renderer.window);
    }
}

pub(super) fn mine_block(ctx: MineBlockContext<'_, '_>, dt: f32) {
    let coord = ctx.controller.cursor_id;
    let voxel = coord.map(|coord| ctx.planet.get_voxel(coord));
    let block = voxel.and_then(|voxel| ctx.planet.content.block(voxel));
    let feedback = ctx.mining.tick(MiningStrikeInput {
        coord,
        voxel,
        block,
        selected_item: ctx.hotbar.selected_item_id(),
        items: &ctx.planet.items,
        dt,
        wants_mining: true,
    });

    match feedback {
        MiningFeedback::None => {}
        MiningFeedback::Blocked => {
            ctx.hotbar.show_notice(HotbarNotice::ProtectedBlock);
            ctx.renderer.window.request_redraw();
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
            let sound = ctx
                .planet
                .content
                .block(voxel)
                .map(|block| sound_kind(block.sound_kind))
                .unwrap_or_default();
            route_feedback(
                ctx.renderer,
                ctx.audio,
                AppFeedback::ToolSwing {
                    strength: impact_strength,
                },
            );
            match ctx
                .planet
                .apply_block_damage(coord, damage, break_threshold)
            {
                BlockDamageResult::Unchanged => {}
                BlockDamageResult::Damaged { .. } => {
                    route_feedback(
                        ctx.renderer,
                        ctx.audio,
                        AppFeedback::BlockHit {
                            sound_kind: sound,
                            strength: impact_strength,
                        },
                    );
                    ctx.renderer.window.request_redraw();
                }
                BlockDamageResult::Broken => {
                    break_block(BreakBlockContext {
                        renderer: &mut *ctx.renderer,
                        planet: &mut *ctx.planet,
                        hotbar: &mut *ctx.hotbar,
                        inventory: &mut *ctx.inventory,
                        loot: ctx.loot,
                        coord,
                        voxel,
                        drops_enabled,
                    });
                    route_feedback(
                        ctx.renderer,
                        ctx.audio,
                        AppFeedback::BlockBreak {
                            sound_kind: sound,
                            strength: impact_strength,
                        },
                    );
                }
            }
        }
        MiningFeedback::Broken { .. } => {}
    }
}

fn break_block(ctx: BreakBlockContext<'_, '_>) {
    let edit = BlockInteraction::apply(BlockAction::Mine(ctx.coord), ctx.planet);
    if edit.dirty_chunks.is_empty() {
        ctx.hotbar.show_notice(HotbarNotice::ProtectedBlock);
        ctx.renderer.window.request_redraw();
        return;
    }

    if ctx.drops_enabled {
        if let Some(block) = ctx.planet.content.block(ctx.voxel) {
            for (item_id, count) in roll_block_drops(&block.drops_key, ctx.loot) {
                collect_drop(item_id, count, ctx.planet, ctx.hotbar, ctx.inventory);
            }
        }
    }
    ctx.renderer.refresh_dirty_chunks(edit.dirty_chunks);
    ctx.renderer.window.request_redraw();
}

fn collect_drop(
    item_id: ItemId,
    count: u32,
    planet: &PlanetData,
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
) {
    let item = planet.items.get(item_id);
    let max_stack = item.map(|i| i.stack_size.0).unwrap_or(99);
    if !hotbar.add(item_id, count, max_stack) && !inventory.add(item_id, count, max_stack) {
        hotbar.show_notice(HotbarNotice::Full);
    }
}

fn roll_block_drops(drops_key: &str, loot: &LootRegistry) -> Vec<(ItemId, u32)> {
    match loot.get_by_key(drops_key) {
        Some(table) => table.roll(|| 0.0),
        None => Vec::new(),
    }
}
