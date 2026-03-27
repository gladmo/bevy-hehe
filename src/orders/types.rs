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
