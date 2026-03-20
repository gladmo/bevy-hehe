/// Item data module for 合合游戏 (HeHe Game).
mod chains;
pub mod types;

pub use types::ItemDef;

use chains::all_items;
use std::collections::HashMap;
use bevy::prelude::Resource;

/// Lookup table for items by ID.
#[derive(Debug, Resource)]
pub struct ItemDatabase {
    pub items: HashMap<String, ItemDef>,
}

impl ItemDatabase {
    pub fn new() -> Self {
        let mut items = HashMap::new();
        for item in all_items() {
            items.insert(item.id.to_string(), item);
        }
        Self { items }
    }

    pub fn get(&self, id: &str) -> Option<&ItemDef> {
        self.items.get(id)
    }

    /// Check whether two items can be merged.
    ///
    /// Merge is possible when both items:
    /// - exist in the database
    /// - have the same item ID (which implies same chain and same level)
    /// - have a `merge_result_id` (i.e. are not at the maximum level)
    ///
    /// The `chain` field on each `ItemDef` uniquely identifies which production
    /// family the item belongs to; because we require `id_a == id_b`, the chains
    /// are guaranteed to match without a redundant comparison.
    pub fn can_merge(&self, id_a: &str, id_b: &str) -> bool {
        if id_a != id_b {
            return false;
        }
        if let Some(item) = self.get(id_a) {
            // Sanity-check: same id implies same chain and same level
            debug_assert!(
                matches!(self.get(id_b), Some(b) if std::mem::discriminant(&b.chain)
                    == std::mem::discriminant(&item.chain) && b.level == item.level),
                "Two items with the same id must belong to the same chain and level"
            );
            item.merge_result_id.is_some()
        } else {
            false
        }
    }
}

impl Default for ItemDatabase {
    fn default() -> Self {
        Self::new()
    }
}
