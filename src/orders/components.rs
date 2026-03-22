/// Order UI component definitions for 合合游戏 (HeHe Game).
use bevy::prelude::*;

/// Tag component for an order slot UI entity (reserved for future dynamic order UI).
#[allow(dead_code)]
#[derive(Component, Debug, Clone)]
pub struct OrderSlot {
    pub order_id: u32,
}

/// Tag component for the submit button of an order.
#[derive(Component, Debug, Clone)]
pub struct OrderSubmitButton {
    pub order_id: u32,
}

/// Tag component for an item icon in an order slot.
/// Each order card has up to 3 of these (item_index 0, 1, 2).
#[derive(Component, Debug, Clone)]
pub struct OrderItemIcon {
    pub order_id: u32,
    /// Index of this item in the order's `items` list (0, 1, or 2).
    pub item_index: u32,
    /// Last item ID loaded into this icon; used to skip redundant asset loads.
    pub cached_item_id: Option<String>,
}

/// Tag component for the order panel container.
#[derive(Component, Debug)]
pub struct OrderPanel;
