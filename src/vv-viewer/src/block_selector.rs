/// Block list built from compiled content. Used by the UI for search, filter, and selection.
use vv_registry::{BlockId, CompiledContent, ContentKey};

pub struct BlockEntry {
    pub key: ContentKey,
    pub display_name: String,
    pub namespace: String,
    pub tags: Vec<String>,
    pub base_color: [f32; 3],
    pub block_id: BlockId,
}

pub struct BlockSelector {
    pub all_blocks: Vec<BlockEntry>,
    pub search: String,
    pub namespace_filter: String,
    pub filtered_indices: Vec<usize>,
    /// Index into `all_blocks` (not into `filtered_indices`).
    pub selected_idx: Option<usize>,
    /// Signal to the UI to scroll the list to the selection on the next frame.
    pub scroll_to_selected: bool,
}

impl BlockSelector {
    pub fn from_content(content: &CompiledContent) -> Self {
        let all_blocks = content
            .blocks
            .keys()
            .iter()
            .filter_map(|key| {
                let block_id = content.blocks.id(key)?;
                let block = content.blocks.get(block_id)?;
                let display_name = block
                    .display_key
                    .clone()
                    .unwrap_or_else(|| key.name().to_string());
                let tags = block
                    .tags
                    .iter()
                    .filter_map(|tid| content.tags.key(*tid))
                    .map(|k| k.name().to_string())
                    .collect();
                Some(BlockEntry {
                    key: key.clone(),
                    display_name,
                    namespace: key.namespace().to_string(),
                    tags,
                    base_color: block.render.color,
                    block_id,
                })
            })
            .collect::<Vec<_>>();

        let filtered_indices = (0..all_blocks.len()).collect();
        let selected_idx = if all_blocks.is_empty() { None } else { Some(0) };

        BlockSelector {
            all_blocks,
            search: String::new(),
            namespace_filter: String::new(),
            filtered_indices,
            selected_idx,
            scroll_to_selected: false,
        }
    }

    /// Rebuild the list from new content, preserving the current selection if possible.
    pub fn rebuild_from_content(&mut self, content: &CompiledContent) {
        let prev_key = self.selected().map(|e| e.key.clone());
        *self = Self::from_content(content);
        if let Some(key) = prev_key {
            self.select_by_key(&key);
        }
    }

    pub fn update_filter(&mut self) {
        let search = self.search.to_lowercase();
        let ns = self.namespace_filter.to_lowercase();
        self.filtered_indices = self
            .all_blocks
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                let key_str = format!("{}:{}", e.namespace, e.key.name()).to_lowercase();
                let name_str = e.display_name.to_lowercase();
                let search_ok = search.is_empty()
                    || key_str.contains(&search)
                    || name_str.contains(&search)
                    || e.tags.iter().any(|t| t.to_lowercase().contains(&search));
                let ns_ok = ns.is_empty() || e.namespace.to_lowercase() == ns;
                search_ok && ns_ok
            })
            .map(|(i, _)| i)
            .collect();
    }

    pub fn selected(&self) -> Option<&BlockEntry> {
        self.selected_idx.and_then(|i| self.all_blocks.get(i))
    }

    pub fn select_by_key(&mut self, key: &ContentKey) -> bool {
        if let Some(pos) = self
            .all_blocks
            .iter()
            .position(|e| e.key.namespace() == key.namespace() && e.key.name() == key.name())
        {
            self.selected_idx = Some(pos);
            self.scroll_to_selected = true;
            true
        } else {
            false
        }
    }

    pub fn all_namespaces(&self) -> Vec<String> {
        let mut ns: std::collections::BTreeSet<String> = self
            .all_blocks
            .iter()
            .map(|e| e.namespace.clone())
            .collect();
        ns.into_iter().collect()
    }
}
