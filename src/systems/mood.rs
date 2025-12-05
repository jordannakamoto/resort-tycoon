use bevy::prelude::*;

use crate::components::Mood;

pub struct MoodPlugin;

impl Plugin for MoodPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, tick_moods);
    }
}

/// Decays moodlets over time and recomputes mood for pawns/guests.
fn tick_moods(time: Res<Time>, mut query: Query<&mut Mood>) {
    for mut mood in &mut query {
        let delta = time.delta_secs();
        for moodlet in mood.moodlets.iter_mut() {
            moodlet.remaining_seconds -= delta;
        }
        mood.moodlets.retain(|m| m.remaining_seconds > 0.0);
        mood.recompute();
    }
}
