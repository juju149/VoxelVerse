use crate::block_registry::CompiledMeshClass;
use vv_content_schema::{RawObjectBlock, RawObjectMeshClass, RawObjectRenderMode, RawObjectShape};

pub(crate) fn compile_mesh_class(block: &RawObjectBlock, has_light: bool) -> CompiledMeshClass {
    if let Some(class) = block.mesh_class {
        return match class {
            RawObjectMeshClass::OpaqueCube => CompiledMeshClass::OpaqueCube,
            RawObjectMeshClass::Cutout => CompiledMeshClass::Cutout,
            RawObjectMeshClass::Prop => CompiledMeshClass::Prop,
            RawObjectMeshClass::Water => CompiledMeshClass::Water,
            RawObjectMeshClass::Foliage => CompiledMeshClass::Foliage,
            RawObjectMeshClass::Emissive => CompiledMeshClass::Emissive,
        };
    }

    match block.render {
        RawObjectRenderMode::Invisible => CompiledMeshClass::Invisible,
        RawObjectRenderMode::Translucent => CompiledMeshClass::Water,
        RawObjectRenderMode::Cutout => CompiledMeshClass::Cutout,
        RawObjectRenderMode::Opaque => {
            if has_light {
                CompiledMeshClass::Emissive
            } else if matches!(block.shape, RawObjectShape::Cross) {
                CompiledMeshClass::Foliage
            } else if block.solid && matches!(block.shape, RawObjectShape::Cube) {
                CompiledMeshClass::OpaqueCube
            } else {
                CompiledMeshClass::Prop
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::compile_mesh_class;
    use crate::block_registry::CompiledMeshClass;
    use vv_content_schema::{
        RawObjectBlock, RawObjectRenderMode, RawObjectShape, RawObjectTexture,
    };

    fn block(render: RawObjectRenderMode, shape: RawObjectShape, solid: bool) -> RawObjectBlock {
        RawObjectBlock {
            texture: RawObjectTexture::None,
            render,
            solid,
            replaceable: false,
            hardness: 0.0,
            sound: Default::default(),
            tint: None,
            shape,
            mesh_class: None,
            states: None,
        }
    }

    #[test]
    fn defaults_keep_massive_cube_terrain_greedy_eligible() {
        let class = compile_mesh_class(
            &block(RawObjectRenderMode::Opaque, RawObjectShape::Cube, true),
            false,
        );
        assert_eq!(class, CompiledMeshClass::OpaqueCube);
    }

    #[test]
    fn routes_special_blocks_away_from_greedy_path() {
        let cutout = compile_mesh_class(
            &block(RawObjectRenderMode::Cutout, RawObjectShape::Cube, true),
            false,
        );
        let foliage = compile_mesh_class(
            &block(RawObjectRenderMode::Opaque, RawObjectShape::Cross, false),
            false,
        );
        assert_eq!(cutout, CompiledMeshClass::Cutout);
        assert_eq!(foliage, CompiledMeshClass::Foliage);
    }
}
