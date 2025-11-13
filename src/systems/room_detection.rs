use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};
use crate::components::*;
use crate::systems::grid::*;
use crate::systems::building::BuildingMap;

pub struct RoomDetectionPlugin;

impl Plugin for RoomDetectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            detect_rooms,
            auto_assign_bedroom_zones,
            auto_assign_lobby_zones,
        ).chain());
    }
}

/// Detects enclosed rooms by finding connected open spaces surrounded by walls
fn detect_rooms(
    mut commands: Commands,
    building_map: Res<BuildingMap>,
    grid_settings: Res<GridSettings>,
    // Only re-detect when buildings change
    wall_query: Query<&GridPosition, (With<Wall>, Changed<GridPosition>)>,
    existing_rooms: Query<Entity, With<Room>>,
) {
    // Only run detection if walls have changed
    if wall_query.is_empty() {
        return;
    }

    // Clear existing rooms
    for room_entity in &existing_rooms {
        commands.entity(room_entity).despawn();
    }

    // Find all enclosed rooms
    let rooms = find_enclosed_rooms(&building_map, &grid_settings);

    // Spawn room entities
    for room_tiles in rooms {
        commands.spawn(Room::new(room_tiles));
    }
}

/// Flood-fill algorithm to find enclosed rooms
fn find_enclosed_rooms(
    building_map: &BuildingMap,
    grid_settings: &GridSettings,
) -> Vec<HashSet<IVec2>> {
    let mut visited = HashSet::new();
    let mut rooms = Vec::new();

    // Check every tile in the grid
    for y in 0..grid_settings.height {
        for x in 0..grid_settings.width {
            let pos = IVec2::new(x, y);

            // Skip if already visited, occupied by a wall, or has a door
            if visited.contains(&pos) || building_map.is_occupied(pos) || building_map.doors.contains_key(&pos) {
                continue;
            }

            // Flood fill from this position
            if let Some(room_tiles) = flood_fill_room(pos, building_map, grid_settings, &mut visited) {
                if room_tiles.len() >= 4 {  // Minimum room size
                    rooms.push(room_tiles);
                }
            }
        }
    }

    rooms
}

/// Flood fill from a position to find all connected open tiles
/// Returns None if the area is not enclosed (reaches map edge)
fn flood_fill_room(
    start_pos: IVec2,
    building_map: &BuildingMap,
    grid_settings: &GridSettings,
    visited: &mut HashSet<IVec2>,
) -> Option<HashSet<IVec2>> {
    let mut room_tiles = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(start_pos);

    let mut is_enclosed = true;

    while let Some(pos) = queue.pop_front() {
        if visited.contains(&pos) {
            continue;
        }

        // Check if we're at the edge of the map (not enclosed)
        if pos.x <= 0 || pos.x >= grid_settings.width - 1 ||
           pos.y <= 0 || pos.y >= grid_settings.height - 1 {
            is_enclosed = false;
            // Continue anyway to mark all tiles as visited
        }

        visited.insert(pos);
        room_tiles.insert(pos);

        // Check all four neighbors
        let neighbors = [
            pos + IVec2::new(1, 0),
            pos + IVec2::new(-1, 0),
            pos + IVec2::new(0, 1),
            pos + IVec2::new(0, -1),
        ];

        for neighbor in neighbors {
            // Skip if out of bounds
            if neighbor.x < 0 || neighbor.x >= grid_settings.width ||
               neighbor.y < 0 || neighbor.y >= grid_settings.height {
                continue;
            }

            // Skip if already visited, occupied by a wall, or has a door (doors divide rooms)
            if visited.contains(&neighbor) || building_map.is_occupied(neighbor) || building_map.doors.contains_key(&neighbor) {
                continue;
            }

            queue.push_back(neighbor);
        }
    }

    // Only return the room if it's properly enclosed
    if is_enclosed {
        Some(room_tiles)
    } else {
        None
    }
}

/// Automatically assigns bedroom zones to rooms that contain beds
fn auto_assign_bedroom_zones(
    mut commands: Commands,
    room_query: Query<(Entity, &Room), Without<Zone>>,
    bed_query: Query<&GridPosition, With<Bed>>,
    furniture_query: Query<(&GridPosition, &Furniture)>,
    mut existing_zones: Query<(Entity, &mut Zone)>,
) {
    for (room_entity, room) in &room_query {
        // Check if this room contains a bed
        let has_bed = bed_query.iter().any(|bed_pos| {
            room.contains_tile(bed_pos.to_ivec2())
        });

        if !has_bed {
            continue;
        }

        // Count furniture in this room for quality calculation
        let furniture_in_room: Vec<_> = furniture_query
            .iter()
            .filter(|(pos, _)| room.contains_tile(pos.to_ivec2()))
            .collect();

        // Calculate zone quality based on room size and furniture
        let quality = calculate_bedroom_quality(room.tile_count(), furniture_in_room.len());

        // Check if a zone already exists for this room
        let mut zone_exists = false;
        for (_, mut zone) in &mut existing_zones {
            if zone.zone_type == ZoneType::GuestBedroom &&
               zone.tiles.iter().any(|tile| room.contains_tile(*tile)) {
                // Update existing zone
                zone.tiles = room.tiles.clone();
                zone.quality = quality;
                zone_exists = true;
                break;
            }
        }

        if !zone_exists {
            // Create new bedroom zone
            let mut zone = Zone::new(
                ZoneType::GuestBedroom,
                format!("Guest Bedroom {}", room_entity.index()),
            );
            zone.tiles = room.tiles.clone();
            zone.quality = quality;

            commands.spawn(zone);
        }
    }
}

/// Calculate bedroom quality based on size and furniture count
fn calculate_bedroom_quality(tile_count: usize, furniture_count: usize) -> ZoneQuality {
    // Basic: Has a bed and minimum size
    if tile_count < 12 {
        return ZoneQuality::None;
    }

    // Quality based on furniture
    match furniture_count {
        0..=1 => ZoneQuality::Basic,      // Just a bed
        2..=3 => ZoneQuality::Good,       // Bed + nightstand/dresser
        4..=5 => ZoneQuality::Excellent,  // Bed + multiple furniture
        _ => ZoneQuality::Luxury,         // Fully furnished
    }
}

/// Automatically assigns lobby zones to rooms that contain reception consoles
fn auto_assign_lobby_zones(
    mut commands: Commands,
    room_query: Query<(Entity, &Room), Without<Zone>>,
    console_query: Query<&GridPosition, With<ReceptionConsole>>,
    furniture_query: Query<(&GridPosition, &Furniture)>,
    mut existing_zones: Query<(Entity, &mut Zone)>,
) {
    for (room_entity, room) in &room_query {
        // Check if this room contains a reception console
        let has_console = console_query.iter().any(|console_pos| {
            room.contains_tile(console_pos.to_ivec2())
        });

        if !has_console {
            continue;
        }

        // Count furniture in this room for quality calculation
        let furniture_in_room: Vec<_> = furniture_query
            .iter()
            .filter(|(pos, _)| room.contains_tile(pos.to_ivec2()))
            .collect();

        // Calculate zone quality based on room size and furniture
        let quality = calculate_lobby_quality(room.tile_count(), furniture_in_room.len());

        // Check if a zone already exists for this room
        let mut zone_exists = false;
        for (_, mut zone) in &mut existing_zones {
            if zone.zone_type == ZoneType::Lobby &&
               zone.tiles.iter().any(|tile| room.contains_tile(*tile)) {
                // Update existing zone
                zone.tiles = room.tiles.clone();
                zone.quality = quality;
                zone_exists = true;
                break;
            }
        }

        if !zone_exists {
            // Create new lobby zone
            let mut zone = Zone::new(
                ZoneType::Lobby,
                format!("Lobby {}", room_entity.index()),
            );
            zone.tiles = room.tiles.clone();
            zone.quality = quality;

            commands.spawn(zone);
        }
    }
}

/// Calculate lobby quality based on size and furniture count
fn calculate_lobby_quality(tile_count: usize, furniture_count: usize) -> ZoneQuality {
    // Basic: Has a reception console and minimum size
    if tile_count < 15 {
        return ZoneQuality::None;
    }

    // Quality based on size and furniture
    if tile_count >= 40 && furniture_count >= 5 {
        ZoneQuality::Luxury
    } else if tile_count >= 30 && furniture_count >= 4 {
        ZoneQuality::Excellent
    } else if tile_count >= 20 && furniture_count >= 2 {
        ZoneQuality::Good
    } else {
        ZoneQuality::Basic
    }
}
