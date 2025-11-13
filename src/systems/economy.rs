use bevy::prelude::*;

#[derive(Resource)]
pub struct Money {
    pub amount: i32,
}

impl Default for Money {
    fn default() -> Self {
        Self {
            amount: 10000, // Starting money
        }
    }
}

impl Money {
    pub fn can_afford(&self, cost: i32) -> bool {
        self.amount >= cost
    }

    pub fn deduct(&mut self, cost: i32) -> bool {
        if self.can_afford(cost) {
            self.amount -= cost;
            true
        } else {
            false
        }
    }

    pub fn add(&mut self, amount: i32) {
        self.amount += amount;
    }
}

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Money>();
    }
}
