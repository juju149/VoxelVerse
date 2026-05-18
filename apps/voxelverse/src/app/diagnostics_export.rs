use crate::app::game_app::GameApp;
use vv_diagnostics::{
    AudioStatsSnapshot, DiagnosticCategory, Diagnostics, DiagnosticsConfig, DiagnosticsFileSink,
    DiagnosticsFrame, DiagnosticsProfile, GameplayStatsSnapshot, PlayerCameraSnapshot,
};
use vv_voxel::CHUNK_SIZE;

pub(super) fn create_diagnostics() -> (Diagnostics, Option<DiagnosticsFileSink>) {
    let config = DiagnosticsConfig::from_env();
    let sink = if config.profile == DiagnosticsProfile::Off {
        None
    } else {
        match DiagnosticsFileSink::new(config.export_dir.clone()) {
            Ok(sink) => {
                println!(
                    "[diagnostics] profile={:?} export_dir={}",
                    config.profile,
                    sink.root().display()
                );
                Some(sink)
            }
            Err(err) => {
                eprintln!("[diagnostics] export disabled: {err}");
                None
            }
        }
    };
    (Diagnostics::new(config), sink)
}

pub(super) fn record_rendered_frame(app: &mut GameApp<'_>, camera_direction: [f32; 3]) {
    if app.diagnostics.profile() == DiagnosticsProfile::Off {
        return;
    }

    let render = app.renderer.diagnostics_render_stats();
    let audio = app.audio.diagnostics();
    let cursor = app.runtime.cursor_id();
    let player_pos = app.runtime.player().position.to_array();
    let current_chunk = cursor.map(|coord| {
        [
            coord.face as i32,
            (coord.u / CHUNK_SIZE) as i32,
            (coord.v / CHUNK_SIZE) as i32,
        ]
    });

    let target_voxel_key =
        cursor.map(|coord| format!("f{} l{} u{} v{}", coord.face, coord.layer, coord.u, coord.v));

    let mut frame = DiagnosticsFrame::new()
        .with_frame_snapshot(app.renderer.diagnostics_frame_stats())
        .with_render(render)
        .with_worldgen(app.runtime.planet().worldgen_stats_snapshot())
        .with_audio(AudioStatsSnapshot {
            voices_started: audio.voices_started,
            voices_throttled: audio.voices_throttled,
            file_open_errors: audio.file_open_errors,
            decode_errors: audio.decode_errors,
            play_errors: audio.play_errors,
            output_unavailable_drops: audio.output_unavailable_drops,
            last_error: audio.last_error.clone(),
        })
        .with_gameplay(GameplayStatsSnapshot {
            player_pos,
            player_chunk: current_chunk.unwrap_or_default(),
            current_biome: None,
            target_voxel_key,
        });

    frame.camera = PlayerCameraSnapshot {
        player_position: player_pos,
        camera_direction,
        current_chunk,
        current_face: cursor.map(|coord| coord.face),
        current_lod: None,
    };
    frame.gauge(
        DiagnosticCategory::Renderer,
        "renderer.active_chunks",
        render.active_chunks as f64,
    );
    frame.gauge(
        DiagnosticCategory::Renderer,
        "renderer.active_lods",
        render.active_lods as f64,
    );
    frame.gauge(
        DiagnosticCategory::Gpu,
        "gpu.upload_bytes",
        render.gpu_upload_bytes as f64,
    );

    app.diagnostics.begin_frame(frame);
    let spike = app.diagnostics.end_frame();

    if let Some(sink) = &mut app.diagnostics_sink {
        let Some(latest) = app.diagnostics.latest_frame() else {
            return;
        };
        let summary = app.diagnostics.rolling_summary();
        if let Err(err) = sink.write_frame(&latest, &summary, spike.as_ref()) {
            eprintln!("[diagnostics] export failed: {err}");
        }
    }
}
