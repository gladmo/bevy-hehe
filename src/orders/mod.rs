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
        // Try up to 10 times to pick a valid template
        for _ in 0..10 {
            let (item_ids, coins) = ORDER_TEMPLATES[rng.gen_range(0..ORDER_TEMPLATES.len())];
            // Validate that all items in the template exist in the database
            if item_ids.iter().all(|&id| db.get(id).is_some()) {
                let order = Order {
                    id: self.next_id,
                    items: item_ids.iter().map(|&s| s.to_string()).collect(),
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

    /// Try to fulfill an order using items from the board.
    /// Returns Some((coin_reward, cells_to_clear)) if fulfilled, None if cannot fulfill.
    pub fn try_fulfill(
        &mut self,
        order_id: u32,
        board_items: &[Option<String>],
    ) -> Option<(u64, Vec<usize>)> {
        let order = self.orders.iter().find(|o| o.id == order_id)?;
        let reward = order.coin_reward;
        let required = order.items.clone();

        // Find one cell per required item (no cell used twice)
        let mut used_cells: Vec<usize> = Vec::new();
        for item_id in &required {
            let found = board_items
                .iter()
                .enumerate()
                .find(|(i, cell)| {
                    cell.as_deref() == Some(item_id.as_str()) && !used_cells.contains(i)
                })
                .map(|(i, _)| i)?;
            used_cells.push(found);
        }

        self.orders.retain(|o| o.id != order_id);
        Some((reward, used_cells))
    }
}
