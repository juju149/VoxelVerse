/// CLI arguments and live session state for the viewer.
use std::path::PathBuf;
use std::str::FromStr;

use crate::debug_mode::DebugMode;

// ---------------------------------------------------------------------------
// Scene mode
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scene {
    Single,
    Wall,   // 3x3 flat grid
    Patch,  // 5x5 flat grid
    Cube,   // 3x3x3 stack
    Stairs, // ascending stair steps
}

impl Scene {
    pub const ALL: &'static [Scene] = &[
        Scene::Single,
        Scene::Wall,
        Scene::Patch,
        Scene::Cube,
        Scene::Stairs,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Scene::Single => "Single",
            Scene::Wall => "Wall 3x3",
            Scene::Patch => "Patch 5x5",
            Scene::Cube => "Cube 3x3x3",
            Scene::Stairs => "Stairs",
        }
    }
}

impl FromStr for Scene {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "single" => Ok(Scene::Single),
            "wall" => Ok(Scene::Wall),
            "patch" => Ok(Scene::Patch),
            "cube" => Ok(Scene::Cube),
            "stairs" => Ok(Scene::Stairs),
            other => Err(format!(
                "unknown scene `{other}`; expected single|wall|patch|cube|stairs"
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Command
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// No arguments: open interactive mode with first available block.
    Interactive,
    Block,
    Compare,
    Screenshot,
}

// ---------------------------------------------------------------------------
// ViewerArgs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ViewerArgs {
    pub command: Command,
    pub block_keys: Vec<String>,
    pub scene: Scene,
    pub screenshot: bool,
    pub assets_root: PathBuf,
}

impl ViewerArgs {
    pub fn parse() -> Result<Self, String> {
        let raw: Vec<String> = std::env::args().skip(1).collect();

        // No arguments: interactive mode (shows first block + full UI).
        if raw.is_empty() {
            return Ok(ViewerArgs {
                command: Command::Interactive,
                block_keys: vec![],
                scene: Scene::Single,
                screenshot: false,
                assets_root: PathBuf::from("assets"),
            });
        }

        let mut iter = raw.into_iter().peekable();
        let cmd_str = iter.next().unwrap();

        let command = match cmd_str.as_str() {
            "block" => Command::Block,
            "compare" => Command::Compare,
            "screenshot" => Command::Screenshot,
            _ => {
                if cmd_str.contains(':') {
                    // Bare content-key: treat as `block <key>`
                    let mut keys = vec![cmd_str];
                    while iter.peek().map(|a| !a.starts_with("--")).unwrap_or(false) {
                        keys.push(iter.next().unwrap());
                    }
                    let mut scene = Scene::Single;
                    let mut screenshot = false;
                    let mut assets_root = PathBuf::from("assets");
                    for arg in iter {
                        match arg.as_str() {
                            "--screenshot" => screenshot = true,
                            _ => {}
                        }
                    }
                    return Ok(ViewerArgs {
                        command: Command::Block,
                        block_keys: keys,
                        scene,
                        screenshot,
                        assets_root,
                    });
                }
                return Err(usage());
            }
        };

        let mut block_keys = Vec::new();
        let mut scene = Scene::Single;
        let mut screenshot = false;
        let mut assets_root = PathBuf::from("assets");

        while let Some(arg) = iter.next() {
            if arg.starts_with("--") {
                match arg.as_str() {
                    "--scene" => {
                        scene = iter
                            .next()
                            .ok_or_else(|| "--scene requires a value".to_string())?
                            .parse()?;
                    }
                    "--screenshot" => screenshot = true,
                    "--assets" => {
                        assets_root = iter
                            .next()
                            .ok_or_else(|| "--assets requires a path".to_string())
                            .map(PathBuf::from)?;
                    }
                    other => return Err(format!("unknown flag `{other}`")),
                }
            } else {
                block_keys.push(arg);
            }
        }

        Ok(ViewerArgs {
            command,
            block_keys,
            scene,
            screenshot,
            assets_root,
        })
    }
}

fn usage() -> String {
    r#"vv-viewer -- VoxelVerse block inspector

Usage:
  vv-viewer                                          interactive (first block)
  vv-viewer block <key> [--scene <mode>]             inspect a block
  vv-viewer compare <key1> <key2> ...                side-by-side comparison
  vv-viewer screenshot <key>                         save screenshot and exit

Scenes: single | wall | patch | cube | stairs

Keys:
  1-9   debug mode  |  G  grid  |  R  reload  |  S  screenshot  |  F  reset cam
  Space turntable   |  Mouse-drag orbit         |  Scroll zoom
"#
    .to_string()
}

// ---------------------------------------------------------------------------
// ViewerState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ViewerState {
    pub debug_mode: DebugMode,
    pub scene: Scene,
    pub show_grid: bool,
    pub turntable: bool,
    pub turntable_angle: f32,

    // Shader preview params
    pub exposure: f32,
    pub variation_scale: f32,
    pub edge_strength_mult: f32,
    pub ao_mult: f32,
    pub bevel_mult: f32,
    pub macro_strength_mult: f32,
    pub micro_strength_mult: f32,

    // Frame signals
    pub needs_screenshot: bool,
    pub reload_error: Option<String>,
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            debug_mode: DebugMode::default(),
            scene: Scene::Single,
            show_grid: true,
            turntable: false,
            turntable_angle: 0.0,
            exposure: 1.0,
            variation_scale: 1.0,
            edge_strength_mult: 1.0,
            ao_mult: 1.0,
            bevel_mult: 1.0,
            macro_strength_mult: 1.0,
            micro_strength_mult: 1.0,
            needs_screenshot: false,
            reload_error: None,
        }
    }
}

impl ViewerState {
    pub fn reset_sliders(&mut self) {
        self.exposure = 1.0;
        self.variation_scale = 1.0;
        self.edge_strength_mult = 1.0;
        self.ao_mult = 1.0;
        self.bevel_mult = 1.0;
        self.macro_strength_mult = 1.0;
        self.micro_strength_mult = 1.0;
    }
}
