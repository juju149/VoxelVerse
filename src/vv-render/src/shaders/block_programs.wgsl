// Legacy hard-coded surface programs (grass/dirt/stone/wood/etc.) lived here.
// They were never wired into the actual albedo dispatch — the live pipeline
// goes through patterned_block_albedo / flat_block_albedo in block_albedo.wgsl.
// File kept as a placeholder so shader_source.rs doesn't need a layout change.
