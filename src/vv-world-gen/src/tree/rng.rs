use super::super::hash01;

#[derive(Clone, Debug)]
pub(crate) struct TreeRng {
    face: u8,
    u: u32,
    v: u32,
    seed: u32,
    cursor: u32,
}

impl TreeRng {
    pub(crate) fn new(face: u8, u: u32, v: u32, seed: u32, flora_index: u32) -> Self {
        Self {
            face,
            u,
            v,
            seed: seed
                .rotate_left(13)
                .wrapping_add(flora_index.wrapping_mul(0x9E37_79B9)),
            cursor: 0,
        }
    }

    pub(crate) fn next_f32(&mut self) -> f32 {
        let value = hash01(
            self.face,
            self.u,
            self.v,
            self.cursor,
            self.seed
                .wrapping_add(self.cursor.wrapping_mul(0x85EB_CA6B)),
        );
        self.cursor = self.cursor.wrapping_add(1);
        value
    }

    pub(crate) fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        if min >= max {
            return min;
        }
        min + (max - min) * self.next_f32()
    }

    pub(crate) fn range_u32(&mut self, min: u32, max: u32) -> u32 {
        if min >= max {
            return min;
        }
        min + ((max - min + 1) as f32 * self.next_f32()).floor() as u32
    }

    pub(crate) fn chance(&mut self, probability: f32) -> bool {
        self.next_f32() < probability.clamp(0.0, 1.0)
    }

    pub(crate) fn direction8(&mut self) -> (i32, i32) {
        const DIRECTIONS: [(i32, i32); 8] = [
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
        ];

        let index = (self.next_f32() * DIRECTIONS.len() as f32)
            .floor()
            .clamp(0.0, (DIRECTIONS.len() - 1) as f32) as usize;

        DIRECTIONS[index]
    }
}
