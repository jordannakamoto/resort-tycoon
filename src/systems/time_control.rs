use bevy::prelude::*;

#[derive(Resource, Default, Clone, Copy, PartialEq)]
pub struct TimeSpeed {
    pub multiplier: f32,
}

impl TimeSpeed {
    pub fn normal() -> Self {
        Self { multiplier: 1.0 }
    }

    pub fn fast() -> Self {
        Self { multiplier: 2.0 }
    }

    pub fn very_fast() -> Self {
        Self { multiplier: 3.0 }
    }

    pub fn set_speed(&mut self, speed: SpeedOption) {
        self.multiplier = match speed {
            SpeedOption::Normal => 1.0,
            SpeedOption::Fast => 2.0,
            SpeedOption::VeryFast => 3.0,
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeedOption {
    Normal,
    Fast,
    VeryFast,
}

pub struct TimeControlPlugin;

impl Plugin for TimeControlPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimeSpeed::normal())
            .add_systems(Update, apply_time_speed);
    }
}

fn apply_time_speed(
    time_speed: Res<TimeSpeed>,
    mut time: ResMut<Time<Virtual>>,
) {
    if time_speed.is_changed() {
        time.set_relative_speed(time_speed.multiplier);
    }
}
