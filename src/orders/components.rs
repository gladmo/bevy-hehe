/// Order UI component definitions for 合合游戏 (HeHe Game).
use bevy::prelude::*;

/// Tag component for an order slot UI entity.
#[allow(dead_code)]
#[derive(Component, Debug, Clone)]
pub struct OrderSlot {
    pub order_id: u32,
}

/// Tag component for an item icon within an order card.
/// Each order card has up to 3 of these (one per possible required item).
#[derive(Component, Debug, Clone)]
pub struct OrderItemIcon {
    /// Which order slot (0, 1, 2).
    pub slot_index: usize,
    /// Position of this icon within the slot's item list (0, 1, 2).
    pub item_pos: usize,
    /// Cache of the last item ID loaded, to avoid redundant asset loads.
    pub cached_item_id: Option<String>,
}

/// Tag component for the complete/submit button of an order.
#[derive(Component, Debug, Clone)]
pub struct OrderSubmitButton {
    pub order_id: u32,
}

/// Tag component for the order panel container.
#[derive(Component, Debug)]
pub struct OrderPanel;
