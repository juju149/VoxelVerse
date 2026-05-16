use std::collections::HashMap;
use vv_voxel::{VoxelCoord, VoxelId};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockDamage {
    pub voxel: VoxelId,
    pub amount: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlockDamageResult {
    Unchanged,
    Damaged { amount: f32, fraction: f32 },
    Broken,
}

#[derive(Clone, Debug, Default)]
pub struct BlockDamageLayer {
    entries: HashMap<VoxelCoord, BlockDamage>,
}

impl BlockDamageLayer {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, coord: VoxelCoord) -> Option<BlockDamage> {
        self.entries.get(&coord).copied()
    }

    pub fn apply_hit(
        &mut self,
        coord: VoxelCoord,
        voxel: VoxelId,
        amount: f32,
        break_threshold: f32,
    ) -> BlockDamageResult {
        if voxel == VoxelId::AIR || amount <= 0.0 || break_threshold <= 0.0 {
            return BlockDamageResult::Unchanged;
        }

        self.clear_if_voxel_changed(coord, voxel);
        let entry = self
            .entries
            .entry(coord)
            .or_insert(BlockDamage { voxel, amount: 0.0 });
        entry.amount += amount;

        if entry.amount >= break_threshold {
            self.entries.remove(&coord);
            return BlockDamageResult::Broken;
        }

        BlockDamageResult::Damaged {
            amount: entry.amount,
            fraction: (entry.amount / break_threshold).clamp(0.0, 1.0),
        }
    }

    pub fn damage_fraction(&self, coord: VoxelCoord, break_threshold: f32) -> Option<f32> {
        if break_threshold <= 0.0 {
            return None;
        }
        self.entries
            .get(&coord)
            .map(|damage| (damage.amount / break_threshold).clamp(0.0, 1.0))
    }

    pub fn damage_fraction_for_voxel(
        &self,
        coord: VoxelCoord,
        voxel: VoxelId,
        break_threshold: f32,
    ) -> Option<f32> {
        let damage = self.entries.get(&coord)?;
        (damage.voxel == voxel)
            .then(|| (damage.amount / break_threshold).clamp(0.0, 1.0))
            .filter(|_| break_threshold > 0.0)
    }

    pub fn clear(&mut self, coord: VoxelCoord) {
        self.entries.remove(&coord);
    }

    pub fn clear_all(&mut self) {
        self.entries.clear();
    }

    pub fn clear_if_voxel_changed(&mut self, coord: VoxelCoord, voxel: VoxelId) {
        if self
            .entries
            .get(&coord)
            .is_some_and(|damage| damage.voxel != voxel)
        {
            self.entries.remove(&coord);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (VoxelCoord, BlockDamage)> + '_ {
        self.entries.iter().map(|(coord, damage)| (*coord, *damage))
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{BlockDamageLayer, BlockDamageResult};
    use vv_voxel::{VoxelCoord, VoxelId};

    const STONE: VoxelId = VoxelId::new(1);
    const DIRT: VoxelId = VoxelId::new(2);

    fn coord(layer: u32) -> VoxelCoord {
        VoxelCoord {
            face: 0,
            layer,
            u: 3,
            v: 4,
        }
    }

    #[test]
    fn hit_creates_damage_entry() {
        let mut layer = BlockDamageLayer::new();

        assert_eq!(
            layer.apply_hit(coord(5), STONE, 0.25, 1.0),
            BlockDamageResult::Damaged {
                amount: 0.25,
                fraction: 0.25
            }
        );
        assert_eq!(layer.get(coord(5)).unwrap().voxel, STONE);
    }

    #[test]
    fn repeated_hits_accumulate() {
        let mut layer = BlockDamageLayer::new();

        layer.apply_hit(coord(5), STONE, 0.25, 1.0);
        let result = layer.apply_hit(coord(5), STONE, 0.25, 1.0);

        assert_eq!(
            result,
            BlockDamageResult::Damaged {
                amount: 0.5,
                fraction: 0.5
            }
        );
    }

    #[test]
    fn reaching_threshold_breaks_and_clears() {
        let mut layer = BlockDamageLayer::new();

        layer.apply_hit(coord(5), STONE, 0.5, 1.0);
        assert_eq!(
            layer.apply_hit(coord(5), STONE, 0.5, 1.0),
            BlockDamageResult::Broken
        );
        assert!(layer.get(coord(5)).is_none());
    }

    #[test]
    fn clearing_removes_damage() {
        let mut layer = BlockDamageLayer::new();

        layer.apply_hit(coord(5), STONE, 0.25, 1.0);
        layer.clear(coord(5));

        assert!(layer.get(coord(5)).is_none());
    }

    #[test]
    fn clear_all_removes_everything() {
        let mut layer = BlockDamageLayer::new();

        layer.apply_hit(coord(5), STONE, 0.25, 1.0);
        layer.apply_hit(coord(6), STONE, 0.25, 1.0);
        layer.clear_all();

        assert!(layer.is_empty());
    }

    #[test]
    fn changed_voxel_discards_previous_damage() {
        let mut layer = BlockDamageLayer::new();

        layer.apply_hit(coord(5), STONE, 0.75, 1.0);
        layer.clear_if_voxel_changed(coord(5), DIRT);

        assert!(layer.get(coord(5)).is_none());
    }

    #[test]
    fn damage_fraction_ignores_stale_voxel() {
        let mut layer = BlockDamageLayer::new();

        layer.apply_hit(coord(5), STONE, 0.75, 1.0);

        assert_eq!(layer.damage_fraction_for_voxel(coord(5), DIRT, 1.0), None);
    }
}
