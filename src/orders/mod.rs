/// Order system: manages the three active orders that the player can fulfill.
mod components;
mod types;

pub use components::{
    format_time, OrderItemText, OrderPanel, OrderRewardText, OrderSubmitButton, OrderTimeText,
};
pub use types::{Order, MAX_ORDERS, ORDER_TEMPLATES};

use bevy::prelude::*;
use rand::Rng;

use crate::items::ItemDatabase;

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
