pub const MATERIAL_INDEX_MASK: u32 = 0x0000_FFFF;

const EDGE_MIN_U: u32 = 1 << 0;
const EDGE_MAX_U: u32 = 1 << 1;
const EDGE_MIN_V: u32 = 1 << 2;
const EDGE_MAX_V: u32 = 1 << 3;

/// Bit 20 of the packed tex index — the fragment shader runs an alpha test only
/// when this flag is set. Used by cross-plane foliage (Minecraft-style) where
/// the texture is alpha-cutout. Cube blocks must NOT set it: their textures
/// often carry partial alpha (oak leaves, glass) that the engine treats as
/// fully opaque outside dedicated transparency passes.
pub const FLAG_ALPHA_TEST: u32 = 1 << 20;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FaceEdgeMask {
    pub min_u: bool,
    pub max_u: bool,
    pub min_v: bool,
    pub max_v: bool,
}

impl FaceEdgeMask {
    fn bits(self) -> u32 {
        (u32::from(self.min_u) * EDGE_MIN_U)
            | (u32::from(self.max_u) * EDGE_MAX_U)
            | (u32::from(self.min_v) * EDGE_MIN_V)
            | (u32::from(self.max_v) * EDGE_MAX_V)
    }
}

pub fn pack_material_edges(material_layer: u32, edges: FaceEdgeMask) -> u32 {
    debug_assert!(material_layer <= MATERIAL_INDEX_MASK);
    (material_layer & MATERIAL_INDEX_MASK) | (edges.bits() << 16)
}

pub fn pack_material_flags(material_layer: u32, edges: FaceEdgeMask, flags: u32) -> u32 {
    pack_material_edges(material_layer, edges) | flags
}

#[cfg(test)]
mod tests {
    use super::{pack_material_edges, FaceEdgeMask, MATERIAL_INDEX_MASK};

    #[test]
    fn material_layer_stays_in_low_bits() {
        let packed = pack_material_edges(
            42,
            FaceEdgeMask {
                min_u: true,
                max_u: false,
                min_v: true,
                max_v: false,
            },
        );

        assert_eq!(packed & MATERIAL_INDEX_MASK, 42);
        assert_eq!(packed >> 16, 0b0101);
    }
}
