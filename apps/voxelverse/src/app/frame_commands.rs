use crate::app::action_result::{ActionResult, FrameCommand, UiEvent};
use crate::app::cursor::{grab_cursor, release_cursor};
use crate::app::feedback_router::route_feedback_events;
use crate::app::runtime_state::GameRuntime;
use vv_audio::AudioEngine;
use vv_gameplay::PlanetResize;
use vv_render::{PcfQuality, Renderer};

/// Apply every command, feedback event, and UI event produced by an action or tick.
///
/// This is the single application site. No command is applied anywhere else in the app.
pub(super) fn apply_action_result(
    result: ActionResult,
    renderer: &mut Renderer<'_>,
    audio: &mut AudioEngine,
    runtime: &mut GameRuntime,
) {
    route_feedback_events(renderer, audio, &result.feedback);
    apply_ui_events(result.ui_events, runtime);
    apply_frame_commands(result.commands, renderer, runtime);
}

/// Apply a batch of `FrameCommand`s to the renderer and runtime.
pub(super) fn apply_frame_commands(
    commands: Vec<FrameCommand>,
    renderer: &mut Renderer<'_>,
    runtime: &mut GameRuntime,
) {
    for cmd in commands {
        apply_one(cmd, renderer, runtime);
    }
}

/// Apply UI events to the hotbar and other UI state.
pub(super) fn apply_ui_events(events: Vec<UiEvent>, runtime: &mut GameRuntime) {
    for event in events {
        match event {
            UiEvent::HotbarNotice(notice) => runtime.hotbar_mut().show_notice(notice),
        }
    }
}

fn apply_one(cmd: FrameCommand, renderer: &mut Renderer<'_>, runtime: &mut GameRuntime) {
    match cmd {
        FrameCommand::Redraw => {
            renderer.window.request_redraw();
        }
        FrameCommand::GrabCursor => {
            grab_cursor(renderer.window);
        }
        FrameCommand::ReleaseCursor => {
            release_cursor(renderer.window);
        }
        FrameCommand::RefreshDirtyChunks(keys) => {
            renderer.refresh_dirty_chunks(keys);
        }
        FrameCommand::ForceReloadWorld => {
            let pos = runtime.player().position;
            renderer.force_reload_all(runtime.planet(), pos);
            renderer.log_memory(runtime.planet());
            renderer.window.request_redraw();
        }
        FrameCommand::ToggleDebugPage => {
            renderer.toggle_engine_debug_page();
            renderer.window.request_redraw();
        }
        FrameCommand::ToggleColorOnlyMode => {
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
        }
        FrameCommand::ToggleTriplanarGrain => {
            renderer.quality.triplanar_grain = !renderer.quality.triplanar_grain;
            println!(
                "[quality] triplanar grain = {}",
                renderer.quality.triplanar_grain
            );
        }
        FrameCommand::CyclePcfQuality => {
            renderer.quality.pcf = match renderer.quality.pcf {
                PcfQuality::Low => PcfQuality::Medium,
                PcfQuality::Medium => PcfQuality::High,
                PcfQuality::High => PcfQuality::Low,
            };
            println!("[quality] PCF = {:?}", renderer.quality.pcf);
        }
        FrameCommand::ResizePlanet(intent) => {
            let (planet, player) = runtime.planet_and_player_mut();
            PlanetResize::apply(intent, planet, player);
            let pos = player.position;
            renderer.force_reload_all(planet, pos);
            renderer.log_memory(planet);
            renderer.window.request_redraw();
        }
    }
}
