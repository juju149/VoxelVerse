use vv_gameplay::Inventory;
use vv_registry::{
    BlockRenderSource, CompiledContent, CompiledIngredient, CompiledItemKind, ItemId, RecipeId,
};
use vv_ui::UiColor;

#[derive(Debug, Clone, PartialEq)]
pub struct ItemVisual {
    pub item: ItemId,
    pub label: String,
    pub color: UiColor,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IngredientVisual {
    pub label: String,
    pub color: UiColor,
    pub required: u32,
    pub available: u32,
}

pub fn item_visual(content: &CompiledContent, item: ItemId, count: u32) -> ItemVisual {
    ItemVisual {
        item,
        label: item_label(content, item),
        color: item_color(content, item),
        count,
    }
}

pub fn ingredient_visuals(
    content: &CompiledContent,
    inventory: &Inventory,
    recipe: RecipeId,
) -> Vec<IngredientVisual> {
    let Some(recipe) = content.recipes.get(recipe) else {
        return Vec::new();
    };

    recipe
        .ingredients
        .iter()
        .map(|ingredient| match *ingredient {
            CompiledIngredient::Item { item, count } => IngredientVisual {
                label: item_label(content, item),
                color: item_color(content, item),
                required: count,
                available: inventory.item_count(item),
            },
            CompiledIngredient::Tag { count, .. } => IngredientVisual {
                label: "Tag ingredient".to_owned(),
                color: UiColor::rgba(0.72, 0.68, 0.58, 1.0),
                required: count,
                available: 0,
            },
        })
        .collect()
}

pub fn item_label(content: &CompiledContent, item: ItemId) -> String {
    if let Some(display_key) = content
        .items
        .get(item)
        .and_then(|item| item.display_key.as_ref())
    {
        return prettify(display_key);
    }

    content
        .items
        .key(item)
        .map(|key| prettify(&key.to_string()))
        .unwrap_or_else(|| "Unknown".to_owned())
}

pub fn item_color(content: &CompiledContent, item: ItemId) -> UiColor {
    let Some(item) = content.items.get(item) else {
        return UiColor::rgb(0.75, 0.75, 0.75);
    };

    let color = match item.kind {
        CompiledItemKind::Block { block } => content
            .block_content()
            .block_render(block)
            .map(|render| render.color)
            .unwrap_or([0.75, 0.75, 0.75]),
        CompiledItemKind::Placeable { .. } => [0.95, 0.72, 0.35],
        CompiledItemKind::Tool { .. } => [0.72, 0.78, 0.85],
        CompiledItemKind::Armor => [0.62, 0.72, 0.90],
        CompiledItemKind::Food => [0.72, 0.90, 0.48],
        CompiledItemKind::Resource => [0.72, 0.68, 0.58],
    };

    UiColor::rgb(color[0], color[1], color[2])
}

fn prettify(value: &str) -> String {
    let raw = value
        .rsplit(':')
        .next()
        .unwrap_or(value)
        .replace(['_', '.', '/'], " ");

    let mut out = String::with_capacity(raw.len());
    let mut capitalize_next = true;

    for c in raw.chars() {
        if capitalize_next && c.is_ascii_alphabetic() {
            out.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            out.push(c);
        }

        if c == ' ' {
            capitalize_next = true;
        }
    }

    out
}
