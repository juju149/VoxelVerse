macro_rules! compact_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u32);

        impl $name {
            pub const fn new(value: u32) -> Self {
                Self(value)
            }

            pub const fn index(self) -> usize {
                self.0 as usize
            }

            pub const fn raw(self) -> u32 {
                self.0
            }
        }

        impl From<u32> for $name {
            fn from(value: u32) -> Self {
                Self::new(value)
            }
        }

        impl From<$name> for usize {
            fn from(value: $name) -> Self {
                value.index()
            }
        }
    };
}

compact_id!(BlockId);
compact_id!(BlockVisualId);
compact_id!(MaterialId);
compact_id!(TextureId);
compact_id!(ItemId);
compact_id!(EntityId);
compact_id!(PlaceableId);
compact_id!(RecipeId);
compact_id!(LootTableId);
compact_id!(TagId);
compact_id!(PlanetTypeId);
compact_id!(BiomeId);
compact_id!(FloraId);
compact_id!(FaunaId);
compact_id!(OreId);
compact_id!(StructureId);
compact_id!(WeatherId);
