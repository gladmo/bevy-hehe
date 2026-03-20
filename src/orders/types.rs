/// Order type definitions for 合合游戏 (HeHe Game).

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
