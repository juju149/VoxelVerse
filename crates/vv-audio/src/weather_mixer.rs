//! Weather ambient mixer (Phase 3.C of the weather/cosmos roadmap).
//!
//! Computes target volumes for the three sustained weather audio layers
//! (precipitation loop, wind loop, distant thunder) from a [`WeatherState`].
//! The actual rodio `Sink`s that play those loops are wired by the host
//! crate once the loop assets ship in the pack — this module owns only the
//! deterministic numeric mixing so it can be tested without an audio device.

use vv_weather::{LightningStrike, PrecipitationKindSample, WeatherState};

/// Resolved volumes in `[0, 1]` for the three sustained loops, plus any
/// thunder one-shots scheduled this frame.
#[derive(Clone, Debug, Default)]
pub struct WeatherAudioMix {
    /// Volume of the precipitation loop (rain/snow). 0 when no precipitation.
    pub precipitation_volume: f32,
    /// Volume of the wind loop. Scales with wind speed.
    pub wind_volume: f32,
    /// Volume of distant thunder rumble — independent of strikes.
    pub thunder_ambient_volume: f32,
    /// Deferred thunder claps from this frame's strikes. The caller schedules
    /// each clap to play after `delay_s` seconds.
    pub thunder_events: Vec<WeatherThunderEvent>,
}

#[derive(Clone, Copy, Debug)]
pub struct WeatherThunderEvent {
    /// Seconds to wait before playing the clap (already computed from the
    /// strike's distance and `thunder_delay_per_km`).
    pub delay_s: f32,
    /// Loudness in `[0, 1]`. Falls off with distance.
    pub volume: f32,
}

/// Wind speed (m/s) at which the wind loop reaches full volume.
pub const WIND_VOLUME_CAP_M_S: f32 = 22.0;
/// Distance (m) at which a strike's thunder clap fades to inaudible.
pub const THUNDER_AUDIBLE_RANGE_M: f32 = 2_000.0;

impl WeatherAudioMix {
    pub fn from_state(state: &WeatherState) -> Self {
        let precipitation_volume = match state.precipitation.kind {
            PrecipitationKindSample::None => 0.0,
            _ => state.precipitation.intensity.clamp(0.0, 1.0),
        };
        let wind_volume = (state.wind.speed / WIND_VOLUME_CAP_M_S).clamp(0.0, 1.0);
        // Ambient rumble fades in once any strike fired in the last few
        // frames; for the mixer we approximate it from the snapshot's
        // current strike count.
        let thunder_ambient_volume = if state.lightning_events.is_empty() {
            0.0
        } else {
            0.4
        };

        let thunder_events = state
            .lightning_events
            .iter()
            .map(thunder_event_from_strike)
            .collect();

        Self {
            precipitation_volume,
            wind_volume,
            thunder_ambient_volume,
            thunder_events,
        }
    }
}

fn thunder_event_from_strike(strike: &LightningStrike) -> WeatherThunderEvent {
    let attenuation = 1.0 - (strike.distance_m / THUNDER_AUDIBLE_RANGE_M).clamp(0.0, 1.0);
    WeatherThunderEvent {
        delay_s: strike.thunder_delay_s.max(0.0),
        volume: attenuation * attenuation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vv_weather::{
        LightningStrike, PrecipitationKindSample, PrecipitationSample, WeatherProfileId,
        WeatherState, WindVector,
    };

    fn state(precip: PrecipitationKindSample, intensity: f32, wind_speed: f32) -> WeatherState {
        WeatherState {
            current: WeatherProfileId(0),
            next: None,
            blend: 0.0,
            cloud_coverage: 0.5,
            fog_multiplier: 1.0,
            cloud_density_mul: 1.0,
            cloud_speed_mul: 1.0,
            wind: WindVector {
                direction: glam::Vec3::X,
                speed: wind_speed,
            },
            precipitation: PrecipitationSample {
                kind: precip,
                intensity,
                wind_drift: 0.0,
                splash_density: 0.0,
            },
            lightning_events: Vec::new(),
        }
    }

    #[test]
    fn clear_weather_silences_loops() {
        let mix = WeatherAudioMix::from_state(&state(PrecipitationKindSample::None, 0.0, 0.0));
        assert_eq!(mix.precipitation_volume, 0.0);
        assert_eq!(mix.wind_volume, 0.0);
        assert!(mix.thunder_events.is_empty());
    }

    #[test]
    fn rain_volume_tracks_intensity() {
        let mix = WeatherAudioMix::from_state(&state(PrecipitationKindSample::Rain, 0.65, 0.0));
        assert!((mix.precipitation_volume - 0.65).abs() < 1e-5);
    }

    #[test]
    fn wind_volume_caps_at_one() {
        let mix = WeatherAudioMix::from_state(&state(
            PrecipitationKindSample::None,
            0.0,
            WIND_VOLUME_CAP_M_S * 2.0,
        ));
        assert_eq!(mix.wind_volume, 1.0);
    }

    #[test]
    fn thunder_event_volume_falls_off_with_distance() {
        let mut st = state(PrecipitationKindSample::Rain, 0.8, 5.0);
        st.lightning_events.push(LightningStrike {
            position: glam::Vec3::ZERO,
            distance_m: 100.0,
            flash_intensity: 4.0,
            thunder_delay_s: 0.3,
        });
        st.lightning_events.push(LightningStrike {
            position: glam::Vec3::ZERO,
            distance_m: 1_500.0,
            flash_intensity: 4.0,
            thunder_delay_s: 4.5,
        });
        let mix = WeatherAudioMix::from_state(&st);
        assert_eq!(mix.thunder_events.len(), 2);
        // Far strike must be quieter than near strike.
        assert!(mix.thunder_events[0].volume > mix.thunder_events[1].volume);
        // Delays match the strikes verbatim.
        assert!((mix.thunder_events[0].delay_s - 0.3).abs() < 1e-5);
        assert!((mix.thunder_events[1].delay_s - 4.5).abs() < 1e-5);
        // Ambient rumble kicks in on a strike frame.
        assert!(mix.thunder_ambient_volume > 0.0);
    }
}
