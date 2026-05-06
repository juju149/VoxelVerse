pub(super) fn calculate(side1: bool, side2: bool, corner: bool) -> f32 {
    let mut occlusion = 0;
    if side1 {
        occlusion += 1;
    }
    if side2 {
        occlusion += 1;
    }
    if corner && (side1 || side2) {
        occlusion += 1;
    }

    match occlusion {
        0 => 1.0,
        1 => 0.8,
        2 => 0.6,
        _ => 0.4,
    }
}
