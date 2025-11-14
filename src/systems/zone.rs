use crate::components::*;
use crate::systems::grid::*;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window as BevyWindow};

pub struct ZoneVisualizationPlugin;

impl Plugin for ZoneVisualizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_room_hover_ui,));
    }
}

#[derive(Component)]
struct RoomStatsPanel;

/// Shows room stats on hover
fn update_room_hover_ui(
    mut commands: Commands,
    window_query: Query<&BevyWindow, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    grid_settings: Res<GridSettings>,
    room_query: Query<&Room>,
    zone_query: Query<&Zone>,
    panel_query: Query<Entity, With<RoomStatsPanel>>,
) {
    // Remove old panel
    for entity in &panel_query {
        commands.entity(entity).despawn_recursive();
    }

    let window = window_query.single();
    let (camera, camera_transform) = camera_query.single();

    // Get cursor position
    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            if let Some(grid_pos) = world_to_grid(
                world_pos,
                grid_settings.tile_size,
                grid_settings.width,
                grid_settings.height,
            ) {
                // Find if cursor is in any room
                for room in &room_query {
                    if room.contains_tile(grid_pos) {
                        // Find the zone for this room
                        let zone = zone_query.iter().find(|z| z.tiles.contains(&grid_pos));

                        // Create stats panel
                        spawn_room_stats_panel(&mut commands, room, zone, cursor_pos);
                        break;
                    }
                }
            }
        }
    }
}

fn spawn_room_stats_panel(
    commands: &mut Commands,
    room: &Room,
    zone: Option<&Zone>,
    cursor_pos: Vec2,
) {
    let panel_text = if let Some(zone) = zone {
        format!(
            "{}\nQuality: {} ({}★)\nSize: {} tiles",
            zone.zone_type.name(),
            zone.quality.name(),
            zone.quality.stars(),
            room.tile_count(),
        )
    } else {
        format!(
            "Unassigned Room\nSize: {} tiles\n\nAdd furniture to create a zone",
            room.tile_count(),
        )
    };

    // Spawn UI panel near cursor
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(cursor_pos.x + 20.0),
                top: Val::Px(cursor_pos.y + 20.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            RoomStatsPanel,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(panel_text),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}
