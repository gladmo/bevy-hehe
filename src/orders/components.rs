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

/// Tag component for order time text.
#[derive(Component, Debug, Clone)]
pub struct OrderTimeText {
    pub order_id: u32,
}

/// Tag component for order item text.
#[derive(Component, Debug, Clone)]
pub struct OrderItemText {
    pub order_id: u32,
}

/// Tag component for the order panel container.
#[derive(Component, Debug)]
pub struct OrderPanel;

/// Format seconds as "Xh Xm Xs" or "Xm Xs" or "Xs".
pub fn format_time(secs: f32) -> String {
    let total = secs.max(0.0) as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}h{m:02}m")
    } else if m > 0 {
        format!("{m}m{s:02}s")
    } else {
        format!("{s}s")
    }
}
