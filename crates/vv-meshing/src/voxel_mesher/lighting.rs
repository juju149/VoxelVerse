/// Simple skylight calculation.
///
/// Casts a short vertical ray (up to `probe` layers above).  Returns a
/// dim value if any opaque block is found above, bright otherwise.
/// Surface voxels are always fully lit so terrain tops stay bright.
pub(super) fn skylight<F>(is_solid_above: F, at_or_above_surface: bool, probe: u32) -> f32
where
    F: Fn(i32) -> bool,
{
    if at_or_above_surface {
        return 1.0;
    }
    for i in 1..=(probe as i32) {
        if is_solid_above(i) {
            return 0.15;
        }
    }
    1.0
}
