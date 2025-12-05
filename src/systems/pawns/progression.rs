use bevy::prelude::*;

use crate::components::{PawnAttributes, PawnProgression};

#[derive(Event)]
pub struct PawnLeveledEvent {
    pub pawn: Entity,
    pub new_level: u32,
}

pub(super) fn tick_pawn_experience(
    time: Res<Time>,
    mut pawns: Query<(Entity, &mut PawnProgression, &PawnAttributes)>,
    mut events: EventWriter<PawnLeveledEvent>,
) {
    for (entity, mut progression, attributes) in &mut pawns {
        let hospitality_bonus = 0.75 + attributes.average_level() / 30.0;
        let xp_gain = hospitality_bonus * time.delta_secs() * 4.0;

        if progression.gain_experience(xp_gain) {
            events.send(PawnLeveledEvent {
                pawn: entity,
                new_level: progression.level,
            });
        }
    }
}
