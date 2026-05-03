use vv_registry::CompiledContent;

use crate::design::InventoryUiTokens;

/// Converts compiled UI theme data into concrete inventory tokens.
///
/// Temporary fallback:
/// - returns hardcoded defaults while the compiled UiTheme model is expanded.
/// Final goal:
/// - InventoryUiTokens must be resolved from pack data.
pub fn inventory_tokens_from_content(_content: &CompiledContent) -> InventoryUiTokens {
    InventoryUiTokens::current()
}
