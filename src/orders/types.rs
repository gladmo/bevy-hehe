/// Order type definitions for 合合游戏 (HeHe Game).

/// An active order requiring 1–3 specific items (each with quantity 1).
#[derive(Debug, Clone)]
pub struct Order {
    pub id: u32,
    /// Required item IDs (1–3 entries, each needed exactly once).
    pub items: Vec<String>,
    /// Coin reward for completing this order.
    pub coin_reward: u64,
}

/// Maximum number of simultaneous orders.
pub const MAX_ORDERS: usize = 3;

/// List of possible order templates: (&[item_id], coin_reward)
/// Each entry specifies 1–3 items (each needed once) and the coin reward.
pub const ORDER_TEMPLATES: &[(&[&str], u64)] = &[
    // Single-item orders
    (&["egg_2"], 30),
    (&["egg_3"], 50),
    (&["egg_5"], 120),
    (&["coolTea_1"], 20),
    (&["coolTea_3"], 60),
    (&["coolTea_5"], 150),
    (&["dough_1"], 25),
    (&["dough_3"], 80),
    (&["dough_5"], 200),
    (&["lantern_1"], 30),
    (&["lantern_3"], 90),
    (&["ring_1"], 40),
    (&["ring_3"], 100),
    (&["fabric_1"], 20),
    (&["fabric_3"], 80),
    // Two-item orders
    (&["egg_2", "coolTea_1"], 60),
    (&["dough_1", "fabric_1"], 55),
    (&["lantern_1", "ring_1"], 80),
    (&["egg_3", "coolTea_3"], 120),
    (&["dough_3", "lantern_3"], 180),
    (&["fabric_3", "ring_3"], 190),
    // Three-item orders
    (&["egg_2", "coolTea_1", "fabric_1"], 90),
    (&["dough_1", "ring_1", "lantern_1"], 110),
    (&["egg_3", "coolTea_3", "dough_3"], 240),
];
