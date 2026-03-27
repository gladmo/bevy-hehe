/// Economy system: stamina, coins, and gems.
use bevy::prelude::*;

/// Maximum stamina value.
pub const MAX_STAMINA: i32 = 100;
/// Stamina recovery interval in seconds (2 minutes per point).
pub const STAMINA_RECOVERY_INTERVAL_SECS: f32 = 120.0;

/// Economy resource: tracks player's stamina, coins, and gems.
#[derive(Resource, Debug)]
pub struct Economy {
    pub stamina: i32,
    pub max_stamina: i32,
    pub coins: u64,
    pub gems: u32,
    /// Accumulated time since last stamina recovery.
    pub stamina_timer: f32,
    /// Player level.
    pub level: u32,
    /// Current experience points.
    pub exp: u64,
    /// Experience needed for next level.
    pub exp_to_next: u64,
}

impl Default for Economy {
    fn default() -> Self {
        Self {
            stamina: MAX_STAMINA,
            max_stamina: MAX_STAMINA,
            coins: 0,
            gems: 0,
            stamina_timer: 0.0,
            level: 1,
            exp: 0,
            exp_to_next: 100,
        }
    }
}

impl Economy {
    /// Try to spend `amount` stamina. Returns true if successful.
    pub fn spend_stamina(&mut self, amount: i32) -> bool {
        if self.stamina >= amount {
            self.stamina -= amount;
            true
        } else {
            false
        }
    }

    /// Add stamina without enforcing the max cap (used by gourd tools).
    pub fn add_stamina_uncapped(&mut self, amount: i32) {
        self.stamina += amount;
    }

    /// Add coins.
    pub fn add_coins(&mut self, amount: u64) {
        self.coins += amount;
    }

    /// Add gems.
    #[allow(dead_code)]
    pub fn add_gems(&mut self, amount: u32) {
        self.gems += amount;
    }

    /// Add experience. Returns true if leveled up.
    pub fn add_exp(&mut self, amount: u64) -> bool {
        self.exp += amount;
        if self.exp >= self.exp_to_next {
            self.exp -= self.exp_to_next;
            self.level += 1;
            // Each level requires 1.3× more exp
            self.exp_to_next = (self.exp_to_next as f32 * 1.3) as u64;
            // Level up: refill stamina
            self.stamina = self.max_stamina;
            true
        } else {
            false
        }
    }

    /// Tick stamina recovery. Call with delta_seconds.
    pub fn tick(&mut self, delta_secs: f32) {
        if self.stamina < self.max_stamina {
            self.stamina_timer += delta_secs;
            while self.stamina_timer >= STAMINA_RECOVERY_INTERVAL_SECS {
                self.stamina_timer -= STAMINA_RECOVERY_INTERVAL_SECS;
                self.stamina = (self.stamina + 1).min(self.max_stamina);
            }
        } else {
            self.stamina_timer = 0.0;
        }
    }
}

/// Tag component for the stamina label.
#[derive(Component)]
pub struct StaminaLabel;

/// Tag component for the coins label.
#[derive(Component)]
pub struct CoinsLabel;

/// Tag component for the gems label.
#[derive(Component)]
pub struct GemsLabel;

/// Tag component for the level label.
#[derive(Component)]
pub struct LevelLabel;
