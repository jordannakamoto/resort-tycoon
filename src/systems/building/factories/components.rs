use bevy::prelude::*;
use crate::components::furniture::*;

/// Inserts the appropriate furniture-specific component onto an entity
pub fn insert_furniture_component(
    entity: Entity,
    furniture_type: FurnitureType,
    orientation: FurnitureOrientation,
    commands: &mut Commands,
) {
    // Add type and orientation components (always added)
    commands.entity(entity)
        .insert(furniture_type)
        .insert(orientation);

    // Add furniture-specific marker/data components
    match furniture_type {
        FurnitureType::Bed(bed_type) => {
            commands.entity(entity).insert(Bed::new(bed_type));
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
            commands.entity(entity).insert(Toilet);
        }
        FurnitureType::Sink => {
            commands.entity(entity).insert(Sink);
        }
        FurnitureType::Tub => {
            commands.entity(entity).insert(Tub);
        }
        FurnitureType::ReceptionConsole => {
            commands.entity(entity).insert(ReceptionConsole::new());
        }
    }
}
