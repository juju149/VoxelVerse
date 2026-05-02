pub const MAIN_SHADER_SOURCE: &str = concat!(
    include_str!("shaders/bindings.wgsl"),
    "\n",
    include_str!("shaders/common.wgsl"),
    "\n",
    include_str!("shaders/planetary.wgsl"),
    "\n",
    include_str!("shaders/tone_mapping.wgsl"),
    "\n",
    include_str!("shaders/block_access.wgsl"),
    "\n",
    include_str!("shaders/block_details.wgsl"),
    "\n",
    include_str!("shaders/block_programs.wgsl"),
    "\n",
    include_str!("shaders/block_albedo.wgsl"),
    "\n",
    include_str!("shaders/shadows.wgsl"),
    "\n",
    include_str!("shaders/lighting.wgsl"),
    "\n",
    include_str!("shaders/terrain_pass.wgsl"),
    "\n",
    include_str!("shaders/sky_pass.wgsl"),
);

pub fn main_shader_source() -> &'static str {
    MAIN_SHADER_SOURCE
}
