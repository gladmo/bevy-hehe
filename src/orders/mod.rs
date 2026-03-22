/// Order system: manages the three active orders that the player can fulfill.
mod components;
mod types;

pub use components::{OrderItemIcon, OrderPanel, OrderSubmitButton};
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
        // Try up to 20 times to pick a valid template (all items must exist in DB)
        for _ in 0..20 {
            let template = ORDER_TEMPLATES[rng.gen_range(0..ORDER_TEMPLATES.len())];
            let (item_ids, coins) = template;
            if item_ids.iter().all(|id| db.get(id).is_some()) {
                let order = Order {
                    id: self.next_id,
                    items: item_ids.iter().map(|s| s.to_string()).collect(),
                    coin_reward: coins,
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

    /// Check whether the board currently contains all items required by `order`.
    pub fn can_fulfill(order: &Order, board_items: &[Option<String>]) -> bool {
        let mut remaining: Vec<Option<&str>> =
            board_items.iter().map(|c| c.as_deref()).collect();
        for item_id in &order.items {
            if let Some(pos) = remaining
                .iter()
                .position(|c| *c == Some(item_id.as_str()))
            {
                remaining[pos] = None;
            } else {
                return false;
            }
        }
        true
    }

    /// Try to fulfill an order using items from the board.
    /// Returns `Some((coin_reward, cells_to_clear))` on success, `None` otherwise.
    pub fn try_fulfill(
        &mut self,
        order_id: u32,
        board_items: &[Option<String>],
    ) -> Option<(u64, Vec<usize>)> {
        let order = self.orders.iter().find(|o| o.id == order_id)?;
        let needed_items = order.items.clone();
        let reward = order.coin_reward;

        // Find one cell for each required item (each consumed once).
        let mut remaining: Vec<Option<(usize, &str)>> = board_items
            .iter()
            .enumerate()
            .map(|(i, c)| c.as_deref().map(|s| (i, s)))
            .collect();

        let mut cells_to_clear = Vec::new();
        for item_id in &needed_items {
            let pos = remaining
                .iter()
                .position(|entry| entry.map(|(_, id)| id) == Some(item_id.as_str()))?;
            let (cell_idx, _) = remaining[pos].take().unwrap();
            cells_to_clear.push(cell_idx);
        }

        self.orders.retain(|o| o.id != order_id);
        Some((reward, cells_to_clear))
    }
}
