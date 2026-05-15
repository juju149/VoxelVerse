use vv_pack_compiler::{CompiledBlock, CompiledItemGameplay, ItemId, ItemRegistry};
use vv_voxel::{VoxelCoord, VoxelId};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MiningOutcome {
    pub coord: VoxelCoord,
    pub voxel: VoxelId,
    pub drops_enabled: bool,
}

#[derive(Clone, Debug, Default)]
pub struct MiningProgress {
    target: Option<MiningTarget>,
}

#[derive(Clone, Copy, Debug)]
struct MiningTarget {
    coord: VoxelCoord,
    voxel: VoxelId,
    required_seconds: f32,
    elapsed_seconds: f32,
    drops_enabled: bool,
}

impl MiningProgress {
    pub fn cancel(&mut self) {
        self.target = None;
    }

    pub fn tick(
        &mut self,
        dt: f32,
        coord: Option<VoxelCoord>,
        voxel: Option<VoxelId>,
        block: Option<&CompiledBlock>,
        selected_item: Option<ItemId>,
        items: &ItemRegistry,
    ) -> Option<MiningOutcome> {
        let Some(coord) = coord else {
            self.cancel();
            return None;
        };
        let Some(voxel) = voxel else {
            self.cancel();
            return None;
        };
        let Some(block) = block else {
            self.cancel();
            return None;
        };
        let Some((required_seconds, drops_enabled)) =
            mining_requirements(block, selected_item, items)
        else {
            self.cancel();
            return None;
        };

        let reset = self.target.is_none_or(|target| {
            target.coord != coord
                || target.voxel != voxel
                || (target.required_seconds - required_seconds).abs() > f32::EPSILON
                || target.drops_enabled != drops_enabled
        });
        if reset {
            self.target = Some(MiningTarget {
                coord,
                voxel,
                required_seconds,
                elapsed_seconds: 0.0,
                drops_enabled,
            });
        }

        let target = self.target.as_mut()?;
        target.elapsed_seconds += dt.max(0.0);
        if target.elapsed_seconds < target.required_seconds {
            return None;
        }

        let outcome = MiningOutcome {
            coord: target.coord,
            voxel: target.voxel,
            drops_enabled: target.drops_enabled,
        };
        self.target = None;
        Some(outcome)
    }
}

fn mining_requirements(
    block: &CompiledBlock,
    selected_item: Option<ItemId>,
    items: &ItemRegistry,
) -> Option<(f32, bool)> {
    if block.hardness < 0.0 {
        return None;
    }

    let hardness = block.hardness.max(0.05);
    let preferred_tool = block
        .preferred_tool_tag
        .as_deref()
        .filter(|tag| !tag.is_empty());

    let mut speed = 0.35;
    let mut drops_enabled = block.required_tool_tier == 0;

    if let Some(item) = selected_item.and_then(|id| items.get(id)) {
        if let CompiledItemGameplay::Tool(tool) = &item.gameplay {
            let tool_matches = preferred_tool
                .map(|required| tool.tool_tag_keys.iter().any(|tag| tag == required))
                .unwrap_or(true);
            if tool_matches {
                speed = tool.mining_speed.max(0.1);
                drops_enabled = tool.tier >= block.required_tool_tier;
            }
        }
    }

    let seconds = (hardness * 1.5 / speed).clamp(0.08, 8.0);
    Some((seconds, drops_enabled))
}
