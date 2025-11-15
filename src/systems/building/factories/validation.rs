use bevy::prelude::*;
use crate::components::furniture::*;
use crate::components::building::GridPosition;
use crate::systems::building::BuildingMap;

/// Validates if furniture can be placed at the given position
pub fn validate_furniture_placement(
    furniture_type: FurnitureType,
    grid_pos: IVec2,
    orientation: FurnitureOrientation,
    building_map: &BuildingMap,
    desk_query: Option<&Query<&GridPosition, With<Desk>>>,
) -> bool {
    let furniture_tiles = furniture_type.tiles_occupied(grid_pos, orientation);

    // Special case: Reception console requires a desk underneath
    if furniture_type == FurnitureType::ReceptionConsole {
        let has_desk = desk_query
            .map(|query| {
                query.iter().any(|desk_pos| {
                    let desk_tiles = vec![
                        desk_pos.to_ivec2(),
                        desk_pos.to_ivec2() + IVec2::new(1, 0),
                        desk_pos.to_ivec2() + IVec2::new(0, 1),
                        desk_pos.to_ivec2() + IVec2::new(1, 1),
                    ];
                    desk_tiles.contains(&grid_pos)
                })
            })
            .unwrap_or(false);

        if !has_desk {
            return false;
        }
        // Reception console doesn't occupy tiles (desk handles that)
        return true;
    }

    // Standard validation: all tiles must have floors and be unoccupied
    furniture_tiles.iter().all(|pos| {
        building_map.floors.contains(pos)
            && !building_map.occupied.contains(pos)
            && !building_map.doors.contains_key(pos)
    })
}
