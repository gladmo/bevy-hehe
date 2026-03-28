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

#[cfg(test)]
mod tests {
    use super::*;

    fn economy_with_stamina(stamina: i32) -> Economy {
        let mut e = Economy::default();
        e.stamina = stamina;
        e
    }

    #[test]
    fn recovery_adds_one_per_interval() {
        let mut e = economy_with_stamina(0);
        // One full interval → stamina goes from 0 to 1.
        e.tick(STAMINA_RECOVERY_INTERVAL_SECS);
        assert_eq!(e.stamina, 1);
    }

    #[test]
    fn recovery_caps_at_max_stamina() {
        let mut e = economy_with_stamina(MAX_STAMINA - 1);
        // Two intervals — second tick should not exceed max.
        e.tick(STAMINA_RECOVERY_INTERVAL_SECS * 2.0);
        assert_eq!(e.stamina, MAX_STAMINA);
    }

    #[test]
    fn recovery_does_not_tick_when_at_max() {
        let mut e = economy_with_stamina(MAX_STAMINA);
        e.tick(STAMINA_RECOVERY_INTERVAL_SECS);
        // Already at cap — must not change.
        assert_eq!(e.stamina, MAX_STAMINA);
        // Timer must have been reset (not accumulated).
        assert_eq!(e.stamina_timer, 0.0);
    }

    #[test]
    fn uncapped_add_can_exceed_max_stamina() {
        let mut e = economy_with_stamina(MAX_STAMINA);
        e.add_stamina_uncapped(50);
        assert_eq!(e.stamina, MAX_STAMINA + 50);
    }

    #[test]
    fn recovery_does_not_tick_when_above_max() {
        let mut e = economy_with_stamina(MAX_STAMINA + 50);
        e.tick(STAMINA_RECOVERY_INTERVAL_SECS);
        // Above cap — must stay unchanged and timer stays reset.
        assert_eq!(e.stamina, MAX_STAMINA + 50);
        assert_eq!(e.stamina_timer, 0.0);
    }

    #[test]
    fn spend_stamina_succeeds_when_enough() {
        let mut e = economy_with_stamina(5);
        assert!(e.spend_stamina(3));
        assert_eq!(e.stamina, 2);
    }

    #[test]
    fn spend_stamina_fails_when_insufficient() {
        let mut e = economy_with_stamina(2);
        assert!(!e.spend_stamina(3));
        assert_eq!(e.stamina, 2);
    }

    #[test]
    fn refund_above_max_is_preserved() {
        // Player has 150 stamina (from Gourd), spends 1, then gets refunded.
        let mut e = economy_with_stamina(150);
        assert!(e.spend_stamina(1));
        assert_eq!(e.stamina, 149);
        // Refund should restore to 150, not clamp to 100.
        e.stamina += 1;
        assert_eq!(e.stamina, 150);
    }
}

/// Tag component for the coins label.
#[derive(Component)]
pub struct CoinsLabel;

/// Tag component for the gems label.
#[derive(Component)]
pub struct GemsLabel;

/// Tag component for the level label.
#[derive(Component)]
pub struct LevelLabel;
