use crate::components::furniture::*;
use bevy::prelude::*;

/// Inserts the appropriate furniture-specific component onto an entity
pub fn insert_furniture_component(
    entity: Entity,
    furniture_type: FurnitureType,
    orientation: FurnitureOrientation,
    commands: &mut Commands,
) {
    // Add type and orientation components (always added)
    commands
        .entity(entity)
        .insert(furniture_type)
        .insert(orientation);

    // Add furniture-specific marker/data components
    match furniture_type {
        FurnitureType::Bed(bed_type) => {
            commands
                .entity(entity)
                .insert(Bed::new(bed_type))
                .insert(BedClaim::default());
        }
        FurnitureType::Desk => {
            commands.entity(entity).insert(Desk);
        }
        FurnitureType::Chair => {
            commands.entity(entity).insert(Chair);
        }
        FurnitureType::Dresser => {
            commands.entity(entity).insert(Dresser);
        }
        FurnitureType::Nightstand => {
            commands.entity(entity).insert(Nightstand);
        }
        FurnitureType::Toilet => {
            commands
                .entity(entity)
                .insert(Toilet)
                .insert(FixtureInUse::default());
        }
        FurnitureType::Sink => {
            commands
                .entity(entity)
                .insert(Sink)
                .insert(FixtureInUse::default());
        }
        FurnitureType::Tub => {
            commands
                .entity(entity)
                .insert(Tub)
                .insert(FixtureInUse::default());
        }
        FurnitureType::ReceptionConsole => {
            commands.entity(entity).insert(ReceptionConsole::new());
        }
    }
}
