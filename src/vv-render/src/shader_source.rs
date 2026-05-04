pub const MAIN_SHADER_SOURCE: &str = concat!(
    include_str!("shaders/bindings.wgsl"),
    "\n",
    include_str!("shaders/common.wgsl"),
    "\n",
    include_str!("shaders/block_access.wgsl"),
    "\n",
    include_str!("shaders/block_albedo.wgsl"),
    "\n",
    include_str!("shaders/lighting.wgsl"),
    "\n",
    include_str!("shaders/terrain_pass.wgsl"),
);

pub fn main_shader_source() -> &'static str {
    MAIN_SHADER_SOURCE
}
