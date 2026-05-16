//! Persistent `glyphon::Buffer` cache keyed by stable slot id.
//!
//! Glyphon text shaping is the dominant CPU cost in the text pass.  Building a
//! fresh `Buffer` and calling `set_text` every frame for FPS, debug overlay,
//! hotbar quantities, inventory labels and console lines re-shapes glyphs even
//! when the string has not changed.  This cache stores one buffer per slot and
//! re-`set_text`s only when the (text, size, line height, color, viewport)
//! tuple actually changed since the previous frame.

#![allow(clippy::too_many_arguments)]

use glyphon::{Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextSlot(pub u64);

impl TextSlot {
    pub const FPS: TextSlot = TextSlot(1);
    pub const DEBUG: TextSlot = TextSlot(2);

    pub const fn hotbar_quantity(index: u32) -> TextSlot {
        TextSlot(0x1000 + index as u64)
    }

    pub const fn inventory_spec(index: u32) -> TextSlot {
        TextSlot(0x2000 + index as u64)
    }

    pub const fn console_history(index: u32) -> TextSlot {
        TextSlot(0x3000 + index as u64)
    }

    pub const fn console_input() -> TextSlot {
        TextSlot(0x3FFF)
    }
}

struct Entry {
    buffer: Buffer,
    text: String,
    size: f32,
    line: f32,
    color: [u8; 3],
    width: u32,
    height: u32,
}

#[derive(Default)]
pub struct TextCache {
    entries: HashMap<TextSlot, Entry>,
}

impl TextCache {
    pub fn ensure(
        &mut self,
        slot: TextSlot,
        font_system: &mut FontSystem,
        text: &str,
        size: f32,
        color: [u8; 3],
        width: u32,
        height: u32,
    ) {
        let line = (size * 1.25).max(size + 2.0);
        let needs_rebuild = match self.entries.get(&slot) {
            None => true,
            Some(entry) => {
                entry.text != text
                    || entry.size != size
                    || entry.line != line
                    || entry.color != color
                    || entry.width != width
                    || entry.height != height
            }
        };
        if !needs_rebuild {
            return;
        }

        let mut buffer = Buffer::new(font_system, Metrics::new(size, line));
        buffer.set_size(font_system, width as f32, height as f32);
        buffer.set_text(
            font_system,
            text,
            Attrs::new()
                .family(Family::Monospace)
                .color(Color::rgb(color[0], color[1], color[2])),
            Shaping::Advanced,
        );
        self.entries.insert(
            slot,
            Entry {
                buffer,
                text: text.to_string(),
                size,
                line,
                color,
                width,
                height,
            },
        );
    }

    pub fn get(&self, slot: TextSlot) -> Option<&Buffer> {
        self.entries.get(&slot).map(|e| &e.buffer)
    }

    pub fn retain<F: FnMut(&TextSlot) -> bool>(&mut self, mut keep: F) {
        self.entries.retain(|k, _| keep(k));
    }
}
