use std::{sync::atomic::Ordering, time::Instant};

use super::{PlanetTerrain, TerrainCacheStats, TerrainColumn};

impl PlanetTerrain {
    pub fn cache_stats(&self) -> TerrainCacheStats {
        TerrainCacheStats {
            cached_columns: self
                .columns
                .read()
                .expect("terrain cache should not be poisoned")
                .len(),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            compute_micros: self.cache_compute_micros.load(Ordering::Relaxed),
        }
    }

    pub(crate) fn column(&self, face: u8, u: u32, v: u32) -> TerrainColumn {
        let u = u.min(self.geometry.resolution - 1);
        let v = v.min(self.geometry.resolution - 1);
        let key = cache_key(face, u, v);

        if let Some(column) = self
            .columns
            .read()
            .expect("terrain cache should not be poisoned")
            .get(&key)
            .copied()
        {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
            return column;
        }

        self.cache_misses.fetch_add(1, Ordering::Relaxed);

        let compute_start = Instant::now();
        let column = self.compute_column(face, u, v);
        let compute_micros = compute_start.elapsed().as_micros().min(u64::MAX as u128) as u64;

        self.cache_compute_micros
            .fetch_add(compute_micros, Ordering::Relaxed);

        self.columns
            .write()
            .expect("terrain cache should not be poisoned")
            .insert(key, column);

        column
    }
}

#[inline(always)]
fn cache_key(face: u8, u: u32, v: u32) -> u64 {
    ((face as u64) << 56) | ((u as u64) << 28) | v as u64
}
