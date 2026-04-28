use crate::common::{ResourceRef, RgbColor};
use serde::{Deserialize, Serialize};

/// UI theme definition. Deserialized from defs/ui/theme.ron.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct UiThemeDef {
    pub font: String,
    #[serde(default)]
    pub font_resource: Option<ResourceRef>,
    pub health_color: RgbColor,
    pub hunger_color: RgbColor,
    pub armor_color: RgbColor,
    pub xp_color: RgbColor,
    pub hotbar_bg_color: RgbColor,
    pub hotbar_selected_color: RgbColor,
    pub text_color: RgbColor,
    pub text_shadow_color: RgbColor,
}

impl Default for UiThemeDef {
    fn default() -> Self {
        UiThemeDef {
            font: "default".into(),
            font_resource: None,
            health_color: RgbColor {
                r: 0.9,
                g: 0.2,
                b: 0.2,
            },
            hunger_color: RgbColor {
                r: 0.85,
                g: 0.55,
                b: 0.1,
            },
            armor_color: RgbColor {
                r: 0.5,
                g: 0.6,
                b: 0.75,
            },
            xp_color: RgbColor {
                r: 0.3,
                g: 0.9,
                b: 0.3,
            },
            hotbar_bg_color: RgbColor {
                r: 0.15,
                g: 0.15,
                b: 0.15,
            },
            hotbar_selected_color: RgbColor {
                r: 1.0,
                g: 0.85,
                b: 0.2,
            },
            text_color: RgbColor {
                r: 1.0,
                g: 1.0,
                b: 1.0,
            },
            text_shadow_color: RgbColor {
                r: 0.1,
                g: 0.1,
                b: 0.1,
            },
        }
    }
}
