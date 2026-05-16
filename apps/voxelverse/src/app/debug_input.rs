use vv_gameplay::{PlanetResize, PlanetResizeIntent, Player};
use vv_render::Renderer;
use vv_world::PlanetData;
use winit::keyboard::{KeyCode, PhysicalKey};

/// Returns `true` if the key is a dev/debug hotkey that was handled.
///
/// These keys are intentionally separated from normal player input so that
/// production input routers stay clean and feature-flagging is explicit.
pub(super) fn handle_dev_key(
    key: PhysicalKey,
    renderer: &mut Renderer<'_>,
    planet: &mut PlanetData,
    player: &mut Player,
) -> bool {
    if handle_planet_resize(key, renderer, planet, player) {
        return true;
    }
    if handle_quality_toggle(key, renderer) {
        return true;
    }
    false
}

/// `[` / `]` — shrink / grow the active planet in dev mode.
fn handle_planet_resize(
    key: PhysicalKey,
    renderer: &mut Renderer<'_>,
    planet: &mut PlanetData,
    player: &mut Player,
) -> bool {
    let intent = match key {
        PhysicalKey::Code(KeyCode::BracketRight) => PlanetResizeIntent::Grow,
        PhysicalKey::Code(KeyCode::BracketLeft) => PlanetResizeIntent::Shrink,
        _ => return false,
    };
    PlanetResize::apply(intent, planet, player);
    renderer.force_reload_all(planet, player.position);
    renderer.log_memory(planet);
    renderer.window.request_redraw();
    true
}

/// F2 / F3 / F5 / F6 — render quality toggles, all behind a dev mode gate.
fn handle_quality_toggle(key: PhysicalKey, renderer: &mut Renderer<'_>) -> bool {
    match key {
        PhysicalKey::Code(KeyCode::F2) => {
            renderer.toggle_engine_debug_page();
            renderer.window.request_redraw();
            true
        }
        PhysicalKey::Code(KeyCode::F3) | PhysicalKey::Code(KeyCode::Fn) => {
            renderer.quality.color_only_mode = !renderer.quality.color_only_mode;
            println!(
                "[quality] color-only mode = {} (textures {})",
                renderer.quality.color_only_mode,
                if renderer.quality.color_only_mode {
                    "OFF"
                } else {
                    "ON"
                }
            );
            true
        }
        PhysicalKey::Code(KeyCode::F5) => {
            renderer.quality.triplanar_grain = !renderer.quality.triplanar_grain;
            println!(
                "[quality] triplanar grain = {}",
                renderer.quality.triplanar_grain
            );
            true
        }
        PhysicalKey::Code(KeyCode::F6) => {
            use vv_render::PcfQuality;
            renderer.quality.pcf = match renderer.quality.pcf {
                PcfQuality::Low => PcfQuality::Medium,
                PcfQuality::Medium => PcfQuality::High,
                PcfQuality::High => PcfQuality::Low,
            };
            println!("[quality] PCF = {:?}", renderer.quality.pcf);
            true
        }
        _ => false,
    }
}
