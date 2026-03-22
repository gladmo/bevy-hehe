/// Order type definitions for 合合游戏 (HeHe Game).

/// An active order requiring specific items (1-3 items, each quantity 1).
#[derive(Debug, Clone)]
pub struct Order {
    pub id: u32,
    /// Item IDs required by this order (1-3 entries, each needed once).
    pub items: Vec<String>,
    /// Coin reward for completing this order.
    pub coin_reward: u64,
}

/// Maximum number of simultaneous orders.
pub const MAX_ORDERS: usize = 3;

/// Order templates: (item_ids, coin_reward).
/// Each order requires 1-3 items, each with quantity 1.
pub const ORDER_TEMPLATES: &[(&[&str], u64)] = &[
    // Single-item orders
    (&["egg_2"], 20),
    (&["egg_3"], 40),
    (&["egg_5"], 100),
    (&["coolTea_1"], 15),
    (&["coolTea_3"], 50),
    (&["coolTea_5"], 120),
    (&["dough_1"], 15),
    (&["dough_3"], 60),
    (&["dough_5"], 160),
    (&["lantern_1"], 20),
    (&["lantern_3"], 70),
    (&["lantern_5"], 180),
    (&["ring_1"], 25),
    (&["ring_3"], 80),
    (&["fabric_1"], 15),
    (&["fabric_3"], 60),
    // Two-item orders
    (&["egg_2", "coolTea_1"], 40),
    (&["dough_1", "fabric_1"], 35),
    (&["lantern_1", "ring_1"], 50),
    (&["egg_3", "dough_3"], 110),
    (&["coolTea_3", "ring_3"], 140),
    // Three-item orders
    (&["egg_2", "coolTea_1", "dough_1"], 55),
    (&["lantern_1", "ring_1", "fabric_1"], 65),
    (&["egg_3", "coolTea_3", "dough_3"], 200),
];
