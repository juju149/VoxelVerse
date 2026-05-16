use vv_pack_compiler::{CompiledBlock, CompiledItemGameplay, ItemId, ItemRegistry};
use vv_voxel::{VoxelCoord, VoxelId};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MiningStrike {
    pub coord: VoxelCoord,
    pub voxel: VoxelId,
    pub damage: f32,
    pub break_threshold: f32,
    pub drops_enabled: bool,
    pub impact_strength: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MiningFeedback {
    None,
    Blocked,
    Hit {
        coord: VoxelCoord,
        voxel: VoxelId,
        damage: f32,
        break_threshold: f32,
        fraction: f32,
        impact_strength: f32,
        drops_enabled: bool,
    },
    Broken {
        coord: VoxelCoord,
        voxel: VoxelId,
        drops_enabled: bool,
        impact_strength: f32,
    },
}

impl MiningFeedback {
    pub fn from_strike(strike: MiningStrike) -> Self {
        Self::Hit {
            coord: strike.coord,
            voxel: strike.voxel,
            damage: strike.damage,
            break_threshold: strike.break_threshold,
            fraction: 0.0,
            impact_strength: strike.impact_strength,
            drops_enabled: strike.drops_enabled,
        }
    }
}

pub struct MiningStrikeInput<'a> {
    pub coord: Option<VoxelCoord>,
    pub voxel: Option<VoxelId>,
    pub block: Option<&'a CompiledBlock>,
    pub selected_item: Option<ItemId>,
    pub items: &'a ItemRegistry,
    pub dt: f32,
    pub wants_mining: bool,
}

#[derive(Clone, Debug, Default)]
pub struct MiningState {
    cooldown_remaining: f32,
}

#[derive(Clone, Copy, Debug)]
struct StrikeTuning {
    damage: f32,
    cooldown: f32,
    drops_enabled: bool,
    impact_strength: f32,
}

impl MiningState {
    pub fn tick(&mut self, input: MiningStrikeInput<'_>) -> MiningFeedback {
        self.cooldown_remaining = (self.cooldown_remaining - input.dt.max(0.0)).max(0.0);
        if !input.wants_mining {
            return MiningFeedback::None;
        }
        if self.cooldown_remaining > 0.0 {
            return MiningFeedback::None;
        }

        let Some(strike) = resolve_strike(
            input.coord,
            input.voxel,
            input.block,
            input.selected_item,
            input.items,
        ) else {
            return MiningFeedback::Blocked;
        };

        self.cooldown_remaining = strike_cooldown(input.block, input.selected_item, input.items);
        MiningFeedback::from_strike(strike)
    }

    pub fn cooldown_remaining(&self) -> f32 {
        self.cooldown_remaining
    }
}

fn resolve_strike(
    coord: Option<VoxelCoord>,
    voxel: Option<VoxelId>,
    block: Option<&CompiledBlock>,
    selected_item: Option<ItemId>,
    items: &ItemRegistry,
) -> Option<MiningStrike> {
    let coord = coord?;
    let voxel = voxel?;
    let block = block?;
    if block.hardness < 0.0 || voxel == VoxelId::AIR {
        return None;
    }

    let tuning = strike_tuning(block, selected_item, items);
    Some(MiningStrike {
        coord,
        voxel,
        damage: tuning.damage,
        break_threshold: block.hardness.max(1.0),
        drops_enabled: tuning.drops_enabled,
        impact_strength: tuning.impact_strength,
    })
}

fn strike_cooldown(
    block: Option<&CompiledBlock>,
    selected_item: Option<ItemId>,
    items: &ItemRegistry,
) -> f32 {
    let Some(block) = block else {
        return 0.25;
    };
    strike_tuning(block, selected_item, items).cooldown
}

fn strike_tuning(
    block: &CompiledBlock,
    selected_item: Option<ItemId>,
    items: &ItemRegistry,
) -> StrikeTuning {
    let preferred_tool = block
        .preferred_tool_tag
        .as_deref()
        .filter(|tag| !tag.is_empty());
    let mut tuning = StrikeTuning {
        damage: 0.25,
        cooldown: 0.45,
        drops_enabled: block.required_tool_tier == 0,
        impact_strength: 0.35,
    };

    let Some(item) = selected_item.and_then(|id| items.get(id)) else {
        return tuning;
    };
    let CompiledItemGameplay::Tool(tool) = &item.gameplay else {
        return tuning;
    };

    let tool_matches = preferred_tool
        .map(|required| tool.tool_tag_keys.iter().any(|tag| tag == required))
        .unwrap_or(true);

    if tool_matches {
        let speed = tool.mining_speed.max(0.1);
        tuning.damage = speed;
        tuning.cooldown = (0.55 / speed.sqrt()).clamp(0.16, 0.65);
        tuning.drops_enabled =
            block.required_tool_tier == 0 || tool.tier >= block.required_tool_tier;
        tuning.impact_strength = (0.45 + speed * 0.12).clamp(0.45, 1.0);
    } else {
        tuning.damage = 0.35;
        tuning.cooldown = 0.5;
        tuning.drops_enabled = block.required_tool_tier == 0;
        tuning.impact_strength = 0.4;
    }

    tuning
}

#[cfg(test)]
mod tests {
    use super::{MiningFeedback, MiningState, MiningStrikeInput};
    use vv_pack_compiler::{
        BlockMaterialLayers, BlockModelId, CompiledBlock, CompiledBlockVisual, CompiledItem,
        CompiledItemGameplay, CompiledItemVisual, CompiledItemWorldModel, CompiledMeshClass,
        CompiledToolData, ItemId, ItemRegistry, StackSize,
    };
    use vv_voxel::{VoxelCoord, VoxelId};

    const STONE: VoxelId = VoxelId::new(2);

    fn coord() -> VoxelCoord {
        VoxelCoord {
            face: 0,
            layer: 4,
            u: 2,
            v: 3,
        }
    }

    fn block(
        hardness: f32,
        preferred_tool_tag: Option<&str>,
        required_tool_tier: u32,
    ) -> CompiledBlock {
        CompiledBlock {
            id: STONE,
            family_key: "core:test/stone".into(),
            state: Default::default(),
            display_name: "Stone".into(),
            solid: true,
            color: [1.0; 3],
            hardness,
            visual: CompiledBlockVisual {
                layers: BlockMaterialLayers::default(),
                tint: [1.0; 3],
                model_id: BlockModelId::for_tests(0),
            },
            category: "test".into(),
            max_stack: 99,
            drops_key: "core:loot/test/stone".into(),
            preferred_tool_tag: preferred_tool_tag.map(str::to_string),
            required_tool_tier,
            sound_kind: vv_pack_compiler::CompiledSoundKind::Stone,
            mesh_class: CompiledMeshClass::OpaqueCube,
        }
    }

    fn item_registry() -> ItemRegistry {
        ItemRegistry::from_items_for_tests(vec![
            tool(0, "core:tag/tool/pickaxe", 1, 2.0),
            tool(1, "core:tag/tool/axe", 1, 1.0),
            tool(2, "core:tag/tool/pickaxe", 0, 2.0),
        ])
    }

    fn tool(raw_id: u32, tag: &str, tier: u32, speed: f32) -> CompiledItem {
        CompiledItem {
            id: ItemId::from_raw(raw_id),
            key: format!("core:test/tool/{raw_id}"),
            display_name: "Tool".into(),
            category: "tool".into(),
            stack_size: StackSize(1),
            visual: CompiledItemVisual {
                icon_key: "items/tool".into(),
                world_model: CompiledItemWorldModel::None,
                hand_model_key: None,
            },
            gameplay: CompiledItemGameplay::Tool(CompiledToolData {
                tool_tag_keys: vec![tag.into()],
                tier,
                mining_speed: speed,
                durability: 32,
            }),
            tag_keys: vec![tag.into()],
        }
    }

    fn input<'a>(
        block: &'a CompiledBlock,
        items: &'a ItemRegistry,
        selected_item: Option<ItemId>,
        dt: f32,
    ) -> MiningStrikeInput<'a> {
        MiningStrikeInput {
            coord: Some(coord()),
            voxel: Some(STONE),
            block: Some(block),
            selected_item,
            items,
            dt,
            wants_mining: true,
        }
    }

    #[test]
    fn click_produces_strike() {
        let block = block(2.0, Some("core:tag/tool/pickaxe"), 0);
        let items = item_registry();
        let mut mining = MiningState::default();

        let feedback = mining.tick(input(&block, &items, Some(ItemId::from_raw(0)), 0.016));

        assert!(matches!(feedback, MiningFeedback::Hit { damage, .. } if damage == 2.0));
    }

    #[test]
    fn cooldown_blocks_fast_repeated_strikes() {
        let block = block(2.0, Some("core:tag/tool/pickaxe"), 0);
        let items = item_registry();
        let mut mining = MiningState::default();

        assert!(matches!(
            mining.tick(input(&block, &items, Some(ItemId::from_raw(0)), 0.016)),
            MiningFeedback::Hit { .. }
        ));
        assert_eq!(
            mining.tick(input(&block, &items, Some(ItemId::from_raw(0)), 0.016)),
            MiningFeedback::None
        );
    }

    #[test]
    fn strike_after_cooldown_is_allowed() {
        let block = block(2.0, Some("core:tag/tool/pickaxe"), 0);
        let items = item_registry();
        let mut mining = MiningState::default();

        mining.tick(input(&block, &items, Some(ItemId::from_raw(0)), 0.016));
        let feedback = mining.tick(input(&block, &items, Some(ItemId::from_raw(0)), 1.0));

        assert!(matches!(feedback, MiningFeedback::Hit { .. }));
    }

    #[test]
    fn wrong_tool_is_weaker() {
        let block = block(2.0, Some("core:tag/tool/pickaxe"), 0);
        let items = item_registry();
        let mut mining = MiningState::default();

        let feedback = mining.tick(input(&block, &items, Some(ItemId::from_raw(1)), 0.016));

        assert!(matches!(feedback, MiningFeedback::Hit { damage, .. } if damage < 1.0));
    }

    #[test]
    fn insufficient_tier_disables_drops() {
        let block = block(2.0, Some("core:tag/tool/pickaxe"), 1);
        let items = item_registry();
        let mut mining = MiningState::default();

        let feedback = mining.tick(input(&block, &items, Some(ItemId::from_raw(2)), 0.016));

        assert!(matches!(
            feedback,
            MiningFeedback::Hit {
                drops_enabled: false,
                ..
            }
        ));
    }

    #[test]
    fn unbreakable_block_blocks_strike() {
        let block = block(-1.0, Some("core:tag/tool/pickaxe"), 0);
        let items = item_registry();
        let mut mining = MiningState::default();

        assert_eq!(
            mining.tick(input(&block, &items, Some(ItemId::from_raw(0)), 0.016)),
            MiningFeedback::Blocked
        );
    }
}
