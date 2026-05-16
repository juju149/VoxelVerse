use crate::app::action_result::{ActionResult, BlockSoundKind, FeedbackEvent, UiEvent};
use vv_gameplay::{
    BlockAction, BlockActionIntent, BlockInteraction, BlockSelection, BlockSelectionMode,
    Controller, Hotbar, HotbarNotice, Inventory, ItemId, MiningFeedback, MiningState,
    MiningStrikeInput, Player,
};
use vv_pack_compiler::{CompiledSoundKind, LootRegistry};
use vv_world::{BlockDamageResult, PlanetData};

/// Context for placing a block. Contains only gameplay state — no renderer, no audio.
/// `view_width` and `view_height` are plain floats copied from the renderer config.
pub(super) struct PlaceBlockContext<'a> {
    pub(super) controller: &'a mut Controller,
    pub(super) player: &'a Player,
    pub(super) planet: &'a mut PlanetData,
    pub(super) hotbar: &'a mut Hotbar,
    pub(super) view_width: f32,
    pub(super) view_height: f32,
}

/// Context for the mining tick. Contains only gameplay state — no renderer, no audio.
pub(super) struct MineBlockContext<'a> {
    pub(super) controller: &'a Controller,
    pub(super) planet: &'a mut PlanetData,
    pub(super) hotbar: &'a mut Hotbar,
    pub(super) inventory: &'a mut Inventory,
    pub(super) mining: &'a mut MiningState,
    pub(super) loot: &'a LootRegistry,
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

/// Convert a pack-level sound kind to the app-layer equivalent.
/// The conversion lives here so no gameplay action imports a feedback or audio crate.
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

/// Place the block currently selected in the hotbar onto the targeted face.
/// Returns all feedback and frame commands — never touches renderer or audio directly.
pub(super) fn place_block(ctx: PlaceBlockContext<'_>) -> ActionResult {
    let mut result = ActionResult::none();

    let selected = ctx.hotbar.selected_item_id();
    let active_voxel = match selected.and_then(|id| ctx.planet.resolve_item_voxel(id)) {
        Some(voxel) => Some(voxel),
        None => {
            if selected.is_none() {
                result.push_ui_event(UiEvent::HotbarNotice(HotbarNotice::EmptySlot));
            } else {
                result.push_ui_event(UiEvent::HotbarNotice(HotbarNotice::InvalidPlacement));
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
        result.push_ui_event(UiEvent::HotbarNotice(HotbarNotice::InvalidPlacement));
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
            let snd = active_voxel
                .and_then(|voxel| ctx.planet.content.block(voxel))
                .map(|block| block_sound(block.sound_kind))
                .unwrap_or_default();
            result.push_feedback(FeedbackEvent::BlockPlace { sound_kind: snd });
            result.push_refresh_chunks(edit.dirty_chunks);
            result.push_redraw();
        }
    } else if ctx.controller.cursor_id.is_none() && ctx.controller.first_person {
        result.push_grab_cursor();
    }

    result
}

/// Advance the mining state by one frame.
/// Returns all feedback and frame commands — never touches renderer or audio directly.
pub(super) fn mine_block(ctx: MineBlockContext<'_>, dt: f32) -> ActionResult {
    let mut result = ActionResult::none();

    let coord = ctx.controller.cursor_id;
    let voxel = coord.map(|c| ctx.planet.get_voxel(c));
    let block = voxel.and_then(|v| ctx.planet.content.block(v));
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
            result.push_ui_event(UiEvent::HotbarNotice(HotbarNotice::ProtectedBlock));
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
            let snd = ctx
                .planet
                .content
                .block(voxel)
                .map(|b| block_sound(b.sound_kind))
                .unwrap_or_default();
            result.push_feedback(FeedbackEvent::ToolSwing {
                strength: impact_strength,
            });
            match ctx
                .planet
                .apply_block_damage(coord, damage, break_threshold)
            {
                BlockDamageResult::Unchanged => {}
                BlockDamageResult::Damaged { .. } => {
                    result.push_feedback(FeedbackEvent::BlockHit {
                        sound_kind: snd,
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
                    result.feedback.extend(break_result.feedback);
                    result.commands.extend(break_result.commands);
                    result.push_feedback(FeedbackEvent::BlockBreak {
                        sound_kind: snd,
                        strength: impact_strength,
                    });
                }
            }
        }
        MiningFeedback::Broken { .. } => {}
    }

    result
}

fn break_block(ctx: BreakBlockContext<'_>) -> ActionResult {
    let mut result = ActionResult::none();
    let edit = BlockInteraction::apply(BlockAction::Mine(ctx.coord), ctx.planet);
    if edit.dirty_chunks.is_empty() {
        result.push_ui_event(UiEvent::HotbarNotice(HotbarNotice::ProtectedBlock));
        result.push_redraw();
        return result;
    }

    if ctx.drops_enabled {
        if let Some(block) = ctx.planet.content.block(ctx.voxel) {
            let mut inventory_full = false;
            for (item_id, count) in roll_block_drops(&block.drops_key, ctx.loot) {
                if collect_drop(item_id, count, ctx.planet, ctx.hotbar, ctx.inventory) {
                    inventory_full = true;
                }
            }
            if inventory_full {
                result.push_ui_event(UiEvent::HotbarNotice(HotbarNotice::Full));
            }
        }
    }
    result.push_refresh_chunks(edit.dirty_chunks);
    result.push_redraw();
    result
}

/// Returns `true` if the item could not be collected (inventory and hotbar both full).
fn collect_drop(
    item_id: ItemId,
    count: u32,
    planet: &PlanetData,
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
) -> bool {
    let item = planet.items.get(item_id);
    let max_stack = item.map(|i| i.stack_size.0).unwrap_or(99);
    !hotbar.add(item_id, count, max_stack) && !inventory.add(item_id, count, max_stack)
}

fn roll_block_drops(drops_key: &str, loot: &LootRegistry) -> Vec<(ItemId, u32)> {
    match loot.get_by_key(drops_key) {
        Some(table) => table.roll(|| 0.0),
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::action_result::{FrameCommand, UiEvent};

    #[test]
    fn action_result_starts_empty() {
        let r = ActionResult::none();
        assert!(r.feedback.is_empty());
        assert!(r.commands.is_empty());
        assert!(r.ui_events.is_empty());
    }

    #[test]
    fn push_redraw_adds_one_command() {
        let mut r = ActionResult::none();
        r.push_redraw();
        assert_eq!(r.commands.len(), 1);
        assert!(matches!(r.commands[0], FrameCommand::Redraw));
    }

    #[test]
    fn push_refresh_chunks_empty_is_noop() {
        let mut r = ActionResult::none();
        r.push_refresh_chunks(vec![]);
        assert!(
            r.commands.is_empty(),
            "empty dirty list must not push a command"
        );
    }

    #[test]
    fn push_refresh_chunks_nonempty_adds_command() {
        use vv_voxel::SurfaceChunkKey;
        let mut r = ActionResult::none();
        let key = SurfaceChunkKey {
            face: 0,
            u_idx: 0,
            v_idx: 0,
        };
        r.push_refresh_chunks(vec![key]);
        assert_eq!(r.commands.len(), 1);
        assert!(matches!(&r.commands[0], FrameCommand::RefreshDirtyChunks(v) if v.len() == 1));
    }

    #[test]
    fn push_grab_cursor_adds_command() {
        let mut r = ActionResult::none();
        r.push_grab_cursor();
        assert_eq!(r.commands.len(), 1);
        assert!(matches!(r.commands[0], FrameCommand::GrabCursor));
    }

    #[test]
    fn push_ui_event_adds_hotbar_notice() {
        let mut r = ActionResult::none();
        r.push_ui_event(UiEvent::HotbarNotice(HotbarNotice::EmptySlot));
        assert_eq!(r.ui_events.len(), 1);
        assert!(r.commands.is_empty());
    }

    #[test]
    fn feedback_events_accumulate_independently_from_commands() {
        let mut r = ActionResult::none();
        r.push_feedback(FeedbackEvent::ToolSwing { strength: 0.5 });
        r.push_redraw();
        r.push_feedback(FeedbackEvent::ToolSwing { strength: 1.0 });
        assert_eq!(r.feedback.len(), 2);
        assert_eq!(r.commands.len(), 1);
    }
}
