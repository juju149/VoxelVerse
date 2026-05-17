//! Raw content schemas for VoxelVerse packs.
//!
//! Every type here is a deserialise-only view of an authored `.ron` file.
//! No runtime IDs, no compilation, no validation beyond what serde provides.
//!
//! Target layout (kept deliberately small — split inner modules when one of
//! them grows past ~800 lines):
//!
//! ```text
//! src
//! ├── lib.rs        — re-exports
//! ├── common.rs     — ContentRef and tiny shared helpers
//! ├── version.rs    — format-version constants
//! ├── pack.rs       — pack.ron manifest
//! ├── object.rs     — defs/objects/*.object.ron
//! ├── world.rs      — defs/world/**.*
//! ├── media.rs      — texture / material descriptors
//! ├── skeleton.rs   — defs/skeletons/**
//! ├── sound.rs      — defs/sounds/**
//! └── generated.rs  — generated/registries/**
//! ```

mod biome_ambience;
mod celestial;
mod common;
mod generated;
mod media;
pub mod object;
mod pack;
mod skeleton;
mod sound;
mod version;
mod voxel_model;
mod weather;
mod world;

pub use biome_ambience::*;
pub use celestial::*;
pub use common::ContentRef;
pub use generated::*;
pub use media::{
    RawAuthoringDef, RawMaterialCategory, RawMaterialDef, RawMaterialTextureSet, RawMaterialTint,
    RawRenderMode, RawTextureSampling, TextureRef,
};
pub use object::*;
pub use pack::{RawIdentityMode, RawPackContentRoots, RawPackKind, RawPackManifest, RawPackRules};
pub use skeleton::*;
pub use sound::*;
pub use version::*;
pub use voxel_model::*;
pub use weather::*;
pub use world::*;
