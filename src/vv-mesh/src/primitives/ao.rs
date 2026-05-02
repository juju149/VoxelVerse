use crate::MeshGen;

impl MeshGen {
    #[inline]
    pub(crate) fn calculate_ao(side1: bool, side2: bool, corner: bool) -> f32 {
        let mut occ = 0;

        if side1 {
            occ += 1;
        }

        if side2 {
            occ += 1;
        }

        if corner && (side1 || side2) {
            occ += 1;
        }

        match occ {
            0 => 1.0,
            1 => 0.8,
            2 => 0.6,
            _ => 0.4,
        }
    }
}
