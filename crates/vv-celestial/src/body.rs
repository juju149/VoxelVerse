//! Runtime celestial body registry.
//!
//! Built from `RawCelestialBodyDef` entries the pack loader provides. Parents
//! are resolved as numeric indices so the orbit solver can walk the chain in
//! constant time per body.

use std::collections::BTreeMap;

use vv_content_schema::{
    RawCelestialBodyDef, RawCelestialCoronaDef, RawCelestialKind, RawCelestialOrbitDef,
    RawCelestialSpinDef, RawCelestialSurfaceDef,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct CelestialBodyId(pub u16);

#[derive(Clone, Debug)]
pub struct ResolvedBody {
    pub id: CelestialBodyId,
    pub key: String,
    pub short_id: String,
    pub display_name: String,
    pub kind: RawCelestialKind,
    pub voxel_model: Option<String>,
    pub radius_m: f64,
    pub orbit: Option<ResolvedOrbit>,
    pub spin: ResolvedSpin,
    pub surface: ResolvedSurface,
    pub visible_from_surface: bool,
    pub lod_billboard_distance_m: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct ResolvedOrbit {
    /// Index of the parent body. `None` ↔ orbits the system barycentre.
    pub parent: Option<CelestialBodyId>,
    pub semi_major_axis_m: f64,
    pub eccentricity: f64,
    pub period_s: f64,
    pub phase_rad: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct ResolvedSpin {
    pub axis: glam::Vec3,
    pub period_s: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct ResolvedSurface {
    pub emissive_color: glam::Vec3,
    pub emissive_intensity: f32,
    pub corona: Option<RawCelestialCoronaDef>,
}

#[derive(Default)]
pub struct CelestialRegistry {
    bodies: Vec<ResolvedBody>,
    by_short_id: BTreeMap<String, CelestialBodyId>,
}

#[derive(Debug)]
pub enum RegistryError {
    UnknownParent { body: String, parent: String },
    DuplicateShortId(String),
    /// A parent chain cycles back on itself (e.g. `earth.parent = moon`,
    /// `moon.parent = earth`). Caught here so `body_position` cannot recurse
    /// forever at runtime.
    ParentCycle { chain: Vec<String> },
    /// `CelestialBodyId` is a `u16`; packs with more bodies than that would
    /// truncate silently. Refuse them at load instead.
    TooManyBodies { count: usize, max: usize },
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::UnknownParent { body, parent } => {
                write!(
                    f,
                    "celestial body '{body}' references unknown parent '{parent}'"
                )
            }
            RegistryError::DuplicateShortId(id) => {
                write!(f, "duplicate celestial short id '{id}'")
            }
            RegistryError::ParentCycle { chain } => {
                write!(f, "celestial parent cycle: {}", chain.join(" -> "))
            }
            RegistryError::TooManyBodies { count, max } => {
                write!(f, "too many celestial bodies: {count} > {max} (CelestialBodyId is u16)")
            }
        }
    }
}

impl std::error::Error for RegistryError {}

impl CelestialRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a registry from the loader's raw output. Two passes: first index
    /// every entry so parents can be resolved into ids, then convert each
    /// orbit's parent ref.
    pub fn from_raw(items: &[(String, RawCelestialBodyDef)]) -> Result<Self, RegistryError> {
        // ID-overflow guard — see `RegistryError::TooManyBodies`.
        let max = u16::MAX as usize;
        if items.len() > max {
            return Err(RegistryError::TooManyBodies {
                count: items.len(),
                max,
            });
        }

        // Pass 1 — short ids and indices.
        let mut by_short_id: BTreeMap<String, CelestialBodyId> = BTreeMap::new();
        for (idx, (key, _raw)) in items.iter().enumerate() {
            let short = short_id_from_key(key);
            if by_short_id
                .insert(short.clone(), CelestialBodyId(idx as u16))
                .is_some()
            {
                return Err(RegistryError::DuplicateShortId(short));
            }
        }

        // Pass 2 — resolve bodies, mapping parent ContentRef → CelestialBodyId.
        let mut bodies = Vec::with_capacity(items.len());
        for (idx, (key, raw)) in items.iter().enumerate() {
            let short = short_id_from_key(key);
            let orbit = if let Some(o) = &raw.orbit {
                Some(resolve_orbit(short.as_str(), o, &by_short_id)?)
            } else {
                None
            };
            bodies.push(ResolvedBody {
                id: CelestialBodyId(idx as u16),
                key: key.clone(),
                short_id: short,
                display_name: raw.display_name.clone(),
                kind: raw.kind,
                voxel_model: raw.voxel_model.as_ref().map(|r| r.0.clone()),
                radius_m: raw.radius_m.max(0.0),
                orbit,
                spin: resolve_spin(&raw.spin),
                surface: resolve_surface(&raw.surface),
                visible_from_surface: raw.visible_from_surface,
                lod_billboard_distance_m: raw.lod_billboard_distance_m.max(0.0),
            });
        }

        // Pass 3 — reject parent cycles. Without this guard, `body_position`
        // recurses forever on a cyclic chain at runtime.
        for body in &bodies {
            if let Some(chain) = detect_cycle_from(&bodies, body.id) {
                let names = chain
                    .into_iter()
                    .map(|id| bodies[id.0 as usize].short_id.clone())
                    .collect();
                return Err(RegistryError::ParentCycle { chain: names });
            }
        }

        Ok(Self {
            bodies,
            by_short_id,
        })
    }

    pub fn len(&self) -> usize {
        self.bodies.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bodies.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ResolvedBody> {
        self.bodies.iter()
    }

    pub fn get(&self, id: CelestialBodyId) -> &ResolvedBody {
        &self.bodies[id.0 as usize]
    }

    pub fn id_of(&self, short_id: &str) -> Option<CelestialBodyId> {
        self.by_short_id.get(short_id).copied()
    }

    /// First body matching the supplied kind, in registry (sort) order.
    pub fn first_of_kind(&self, kind: RawCelestialKind) -> Option<CelestialBodyId> {
        self.bodies.iter().find(|b| b.kind == kind).map(|b| b.id)
    }
}

/// Walk the parent chain from `start` and return the cyclic loop if one
/// exists. The returned vector starts and ends on the same id so the error
/// message reads e.g. `earth -> moon -> earth`. Bounded by `bodies.len()`.
fn detect_cycle_from(bodies: &[ResolvedBody], start: CelestialBodyId) -> Option<Vec<CelestialBodyId>> {
    let mut chain: Vec<CelestialBodyId> = vec![start];
    let mut current = start;
    loop {
        let parent = bodies[current.0 as usize]
            .orbit
            .as_ref()
            .and_then(|o| o.parent);
        match parent {
            None => return None,
            Some(p) => {
                if chain.contains(&p) {
                    chain.push(p);
                    return Some(chain);
                }
                chain.push(p);
                current = p;
                if chain.len() > bodies.len() + 1 {
                    // Defensive — shouldn't be reached given the `contains`
                    // check above.
                    return Some(chain);
                }
            }
        }
    }
}

fn resolve_orbit(
    body_short: &str,
    raw: &RawCelestialOrbitDef,
    index: &BTreeMap<String, CelestialBodyId>,
) -> Result<ResolvedOrbit, RegistryError> {
    let parent = match &raw.parent {
        Some(parent_ref) => {
            let parent_short = short_id_from_key(&parent_ref.0);
            let parent_id = index.get(parent_short.as_str()).copied().ok_or_else(|| {
                RegistryError::UnknownParent {
                    body: body_short.to_string(),
                    parent: parent_ref.0.clone(),
                }
            })?;
            Some(parent_id)
        }
        None => None,
    };
    Ok(ResolvedOrbit {
        parent,
        semi_major_axis_m: raw.semi_major_axis_m.max(0.0),
        eccentricity: raw.eccentricity.clamp(0.0, 0.999),
        period_s: raw.period_s.max(1.0e-3),
        phase_rad: raw.phase_rad,
    })
}

fn resolve_spin(raw: &RawCelestialSpinDef) -> ResolvedSpin {
    let axis = glam::Vec3::new(raw.axis.0, raw.axis.1, raw.axis.2);
    let axis = if axis.length_squared() > 1e-6 {
        axis.normalize()
    } else {
        glam::Vec3::Y
    };
    ResolvedSpin {
        axis,
        period_s: raw.period_s.max(1.0e-3),
    }
}

fn resolve_surface(raw: &RawCelestialSurfaceDef) -> ResolvedSurface {
    ResolvedSurface {
        emissive_color: glam::Vec3::new(
            raw.emissive_color.0,
            raw.emissive_color.1,
            raw.emissive_color.2,
        ),
        emissive_intensity: raw.emissive_intensity.max(0.0),
        corona: raw.corona,
    }
}

fn short_id_from_key(key: &str) -> String {
    key.rsplit('/').next().unwrap_or(key).to_string()
}
