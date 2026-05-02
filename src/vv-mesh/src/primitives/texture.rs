use vv_registry::TextureId;

use crate::MeshGen;

impl MeshGen {
    #[inline]
    pub(crate) fn face_texture_id(texture: Option<TextureId>) -> i32 {
        texture.map(|id| id.raw() as i32).unwrap_or(-1)
    }
}
