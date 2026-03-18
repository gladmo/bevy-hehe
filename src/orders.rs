/// Order system: manages the three active orders that the player can fulfill.
use bevy::prelude::*;
use rand::Rng;

use crate::items::ItemDatabase;

/// An active order requiring specific items.
#[derive(Debug, Clone)]
pub struct Order {
    pub id: u32,
    pub item_id: String,
    pub item_name: String,
    pub item_emoji: String,
    pub quantity: u32,
    /// Coin reward for completing this order.
    pub coin_reward: u64,
    /// Remaining time in seconds.
    pub time_remaining_secs: f32,
}

/// Maximum number of simultaneous orders.
pub const MAX_ORDERS: usize = 3;

/// Order panel resource.
#[derive(Resource, Debug)]
pub struct Orders {
    pub orders: Vec<Order>,
    pub next_id: u32,
}

impl Default for Orders {
    fn default() -> Self {
        Self {
            orders: Vec::new(),
            next_id: 1,
        }
    }
}

/// List of possible order targets: (item_id, quantity, coin_reward, duration_secs)
pub const ORDER_TEMPLATES: &[(&str, u32, u64, f32)] = &[
    ("egg_2", 2, 30, 1800.0),
    ("egg_3", 1, 50, 3600.0),
    ("egg_5", 1, 120, 7200.0),
    ("coolTea_1", 3, 20, 1200.0),
    ("coolTea_3", 1, 60, 3600.0),
    ("coolTea_5", 1, 150, 7200.0),
    ("dough_1", 2, 25, 1800.0),
    ("dough_3", 1, 80, 5400.0),
    ("dough_5", 1, 200, 10800.0),
    ("lantern_1", 2, 30, 1800.0),
    ("lantern_3", 1, 90, 5400.0),
    ("lantern_5", 1, 220, 10800.0),
    ("ring_1", 2, 40, 1800.0),
    ("ring_3", 1, 100, 5400.0),
    ("fabric_1", 3, 20, 1800.0),
    ("fabric_3", 1, 80, 5400.0),
];

impl Orders {
    /// Generate a new random order using the item database.
    pub fn generate_order(&mut self, db: &ItemDatabase) -> Option<Order> {
        let mut rng = rand::thread_rng();
        // Try up to 10 times to pick a valid template
        for _ in 0..10 {
            let template = ORDER_TEMPLATES[rng.gen_range(0..ORDER_TEMPLATES.len())];
            let (item_id, qty, coins, duration) = template;
            if let Some(item_def) = db.get(item_id) {
                let order = Order {
                    id: self.next_id,
                    item_id: item_id.to_string(),
                    item_name: item_def.name.to_string(),
                    item_emoji: item_def.emoji.to_string(),
                    quantity: qty,
                    coin_reward: coins,
                    time_remaining_secs: duration,
                };
                self.next_id += 1;
                return Some(order);
            }
        }
        None
    }

    /// Fill up to MAX_ORDERS active orders.
    pub fn fill_orders(&mut self, db: &ItemDatabase) {
        while self.orders.len() < MAX_ORDERS {
            if let Some(order) = self.generate_order(db) {
                self.orders.push(order);
            } else {
                break;
            }
        }
    }

    /// Tick order timers. Returns IDs of expired orders.
    pub fn tick(&mut self, delta_secs: f32) -> Vec<u32> {
        let mut expired = Vec::new();
        for order in &mut self.orders {
            order.time_remaining_secs -= delta_secs;
            if order.time_remaining_secs <= 0.0 {
                expired.push(order.id);
            }
        }
        if !expired.is_empty() {
            self.orders.retain(|o| o.time_remaining_secs > 0.0);
        }
        expired
    }

    /// Try to fulfill an order using items from the board.
    /// Returns Some(coin_reward) if fulfilled, None if cannot fulfill.
    pub fn try_fulfill(
        &mut self,
        order_id: u32,
        board_items: &[Option<String>],
    ) -> Option<(u64, Vec<usize>)> {
        let order = self.orders.iter().find(|o| o.id == order_id)?;
        let target_id = order.item_id.clone();
        let needed = order.quantity as usize;
        let reward = order.coin_reward;

        // Find cells containing the required item
        let matching: Vec<usize> = board_items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.as_deref() == Some(&target_id))
            .map(|(i, _)| i)
            .take(needed)
            .collect();

        if matching.len() < needed {
            return None;
        }

        let reward_cells = matching[..needed].to_vec();
        self.orders.retain(|o| o.id != order_id);
        Some((reward, reward_cells))
    }
}

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
