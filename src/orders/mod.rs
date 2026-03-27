/// Order system: manages the three active orders that the player can fulfill.
mod components;
mod types;

pub use components::{OrderItemIcon, OrderPanel, OrderSubmitButton};
pub use types::{Order, MAX_ORDERS};

use bevy::prelude::*;
use rand::Rng;

use crate::config::load_orders;
use crate::items::ItemDatabase;

/// Order panel resource.
#[derive(Resource, Debug)]
pub struct Orders {
    pub orders: Vec<Order>,
    pub next_id: u32,
    /// Order templates loaded once from `orders.csv` at construction time.
    templates: Vec<(Vec<String>, u64)>,
}

impl Default for Orders {
    fn default() -> Self {
        Self {
            orders: Vec::new(),
            next_id: 1,
            templates: load_orders(),
        }
    }
}

impl Orders {
    /// Generate a new random order using the item database.
    pub fn generate_order(&mut self, db: &ItemDatabase) -> Option<Order> {
        if self.templates.is_empty() {
            return None;
        }
        let mut rng = rand::thread_rng();
        // Try up to 10 times to pick a valid template
        for _ in 0..10 {
            let (item_ids, coins) = &self.templates[rng.gen_range(0..self.templates.len())];
            // Validate that all items in the template exist in the database
            if item_ids.iter().all(|id| db.get(id).is_some()) {
                let order = Order {
                    id: self.next_id,
                    items: item_ids.clone(),
                    coin_reward: *coins,
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

        // Find one cell per required item; track consumed cells in a HashSet for O(1) lookup.
        let mut used_cells: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut cells_to_remove: Vec<usize> = Vec::new();
        for item_id in &required {
            let found = board_items
                .iter()
                .enumerate()
                .find(|(i, cell)| {
                    cell.as_deref() == Some(item_id.as_str()) && !used_cells.contains(i)
                })
                .map(|(i, _)| i)?;
            used_cells.insert(found);
            cells_to_remove.push(found);
        }

        self.orders.retain(|o| o.id != order_id);
        Some((reward, cells_to_remove))
    }
}
