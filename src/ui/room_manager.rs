use bevy::prelude::*;

use crate::systems::room_detection::RoomDirectory;
use crate::components::ZoneType;

#[derive(Resource, Default)]
pub struct RoomManagerPanelState {
    pub visible: bool,
}

#[derive(Component)]
struct RoomManagerPanel;

#[derive(Component)]
struct RoomManagerContent;

pub struct RoomManagerUiPlugin;

impl Plugin for RoomManagerUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoomManagerPanelState>()
            .add_systems(Startup, setup_room_manager_panel)
            .add_systems(
                Update,
                (
                    toggle_room_panel_with_keyboard,
                    apply_panel_visibility,
                    rebuild_room_panel,
                ),
            );
    }
}

fn setup_room_manager_panel(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                top: Val::Px(16.0),
                width: Val::Px(420.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(12.0)),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.07, 0.08, 0.12, 0.92)),
            BorderRadius::all(Val::Px(8.0)),
            BorderColor(Color::srgb(0.3, 0.55, 0.8)),
            RoomManagerPanel,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                RoomManagerContent,
            ));
        });
}

fn toggle_room_panel_with_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut panel_state: ResMut<RoomManagerPanelState>,
) {
    if keyboard.just_pressed(KeyCode::KeyL) {
        panel_state.visible = !panel_state.visible;
    }
}

fn apply_panel_visibility(
    panel_state: Res<RoomManagerPanelState>,
    mut panel_query: Query<&mut Node, With<RoomManagerPanel>>,
) {
    if !panel_state.is_changed() {
        return;
    }

    if let Ok(mut node) = panel_query.get_single_mut() {
        node.display = if panel_state.visible {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn rebuild_room_panel(
    mut commands: Commands,
    panel_state: Res<RoomManagerPanelState>,
    directory: Res<RoomDirectory>,
    content_query: Query<Entity, With<RoomManagerContent>>,
    children_query: Query<&Children>,
) {
    if !panel_state.visible {
        return;
    }
    let Ok(content_entity) = content_query.get_single() else {
        return;
    };
    if !(panel_state.is_changed() || directory.is_changed()) {
        return;
    }

    if let Ok(children) = children_query.get(content_entity) {
        for &child in children.iter() {
            commands.entity(child).despawn_recursive();
        }
    }

    commands.entity(content_entity).with_children(|parent| {
        parent.spawn((
            Text::new("Room Manager"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.95, 0.98)),
        ));

        parent.spawn((
            Text::new(format!("Tracked rooms: {}", directory.rooms.len())),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.75, 0.9, 1.0)),
        ));

        for room in directory.rooms.iter().take(20) {
            let zone_label = room
                .zone_type
                .map(|z| match z {
                    ZoneType::GuestBedroom => "Guest Bedroom",
                    ZoneType::Lobby => "Lobby",
                    _ => "Other Zone",
                })
                .unwrap_or("Unzoned");

            parent.spawn((
                Text::new(format!(
                    "• {} • {} • {} tiles • {} beds • {} furniture • {}⭐",
                    room.name,
                    zone_label,
                    room.tile_count,
                    room.bed_count,
                    room.furniture_count,
                    room.quality.stars()
                )),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.95, 1.0)),
            ));
        }

        parent.spawn((
            Text::new("Tip: L toggles this panel. Data auto-refreshes after builds or loads."),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.85, 1.0)),
        ));
    });
}
