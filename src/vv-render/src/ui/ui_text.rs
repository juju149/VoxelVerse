use vv_ui::{UiLayer, UiTextCommand};

#[derive(Debug, Clone, PartialEq)]
pub struct UiTextItem {
    pub layer: UiLayer,
    pub command: UiTextCommand,
}

impl UiTextItem {
    pub fn new(layer: UiLayer, command: UiTextCommand) -> Self {
        Self { layer, command }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UiTextFrame {
    items: Vec<UiTextItem>,
}

impl UiTextFrame {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(&mut self, item: UiTextItem) {
        self.items.push(item);
    }

    pub fn items(&self) -> &[UiTextItem] {
        &self.items
    }

    pub fn into_items(self) -> Vec<UiTextItem> {
        self.items
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}
