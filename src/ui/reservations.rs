use bevy::prelude::*;

use crate::components::{Guest, Mood, Zone, ZoneType};

#[derive(Resource, Default)]
pub struct ReservationsPanelState {
    pub visible: bool,
}

#[derive(Resource)]
pub struct ReservationsData {
    pub reservations: Vec<ReservationEntry>,
}

impl Default for ReservationsData {
    fn default() -> Self {
        Self {
            reservations: vec![
                ReservationEntry::new("Sample Booking", None, ReservationStatus::Pending),
                ReservationEntry::new(
                    "VIP Suite Hold",
                    Some("Luxury Guest"),
                    ReservationStatus::AwaitingCheckIn,
                ),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReservationEntry {
    pub room_label: String,
    pub guest_label: Option<String>,
    pub status: ReservationStatus,
}

impl ReservationEntry {
    pub fn new(room_label: impl Into<String>, guest: Option<&str>, status: ReservationStatus) -> Self {
        Self {
            room_label: room_label.into(),
            guest_label: guest.map(|s| s.to_string()),
            status,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ReservationStatus {
    Pending,
    AwaitingCheckIn,
    CheckedIn,
    Completed,
    Cancelled,
}

#[derive(Component)]
struct ReservationsPanel;

#[derive(Component)]
struct ReservationsContent;

pub struct ReservationsUiPlugin;

impl Plugin for ReservationsUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ReservationsPanelState>()
            .init_resource::<ReservationsData>()
            .add_systems(Startup, setup_reservations_panel)
            .add_systems(
                Update,
                (
                    toggle_panel_with_keyboard,
                    apply_panel_visibility,
                    rebuild_reservations_panel,
                ),
            );
    }
}

fn setup_reservations_panel(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(16.0),
                top: Val::Px(16.0),
                width: Val::Px(420.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                padding: UiRect::all(Val::Px(12.0)),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.07, 0.1, 0.9)),
            BorderRadius::all(Val::Px(8.0)),
            BorderColor(Color::srgb(0.25, 0.55, 0.8)),
            ReservationsPanel,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                ReservationsContent,
            ));
        });
}

fn toggle_panel_with_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut panel_state: ResMut<ReservationsPanelState>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        panel_state.visible = !panel_state.visible;
    }
}

fn apply_panel_visibility(
    panel_state: Res<ReservationsPanelState>,
    mut panel_query: Query<&mut Node, With<ReservationsPanel>>,
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

fn rebuild_reservations_panel(
    mut commands: Commands,
    panel_state: Res<ReservationsPanelState>,
    reservations: Res<ReservationsData>,
    zones: Query<&Zone>,
    guests: Query<(&Guest, Option<&Mood>)>,
    content_query: Query<Entity, With<ReservationsContent>>,
    children_query: Query<&Children>,
) {
    if !panel_state.visible {
        return;
    }

    let Ok(content_entity) = content_query.get_single() else {
        return;
    };

    if !(panel_state.is_changed() || reservations.is_changed()) {
        return;
    }

    if let Ok(children) = children_query.get(content_entity) {
        for &child in children.iter() {
            commands.entity(child).despawn_recursive();
        }
    }

    // Collect simple room stats
    let mut guest_rooms = Vec::new();
    for zone in zones.iter().filter(|z| z.zone_type == ZoneType::GuestBedroom) {
        guest_rooms.push(format!(
            "{} ({} tiles, {}⭐)",
            zone.name,
            zone.tile_count(),
            zone.quality.stars()
        ));
    }

    // Collect guest summaries
    let mut guest_summaries = Vec::new();
    for (guest, mood) in &guests {
        let mood_str = mood
            .map(|m| format!("Mood {:.0}", m.current))
            .unwrap_or_else(|| "Mood —".to_string());
        guest_summaries.push(format!("{} ({:?}) • {}", guest.name, guest.tier, mood_str));
    }

    commands.entity(content_entity).with_children(|parent| {
        parent.spawn((
            Text::new("Reservations & Occupancy"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.95, 0.98)),
        ));

        // Rooms
        parent.spawn((
            Text::new(format!("Guest Rooms: {}", guest_rooms.len())),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.75, 0.9, 1.0)),
        ));
        for room in guest_rooms.iter().take(12) {
            parent.spawn((
                Text::new(format!("• {room}")),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.9, 1.0)),
            ));
        }

        // Guests
        parent.spawn((
            Text::new(format!("Guests on site: {}", guest_summaries.len())),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.95, 0.8)),
        ));
        for guest in guest_summaries.iter().take(12) {
            parent.spawn((
                Text::new(format!("• {guest}")),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.95, 0.85)),
            ));
        }

        // Reservations list
        parent.spawn((
            Text::new(format!("Reservations: {}", reservations.reservations.len())),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.9, 0.75)),
        ));

        for res in reservations.reservations.iter().take(12) {
            let guest_label = res
                .guest_label
                .as_ref()
                .map(|g| format!(" • Guest: {g}"))
                .unwrap_or_default();
            parent.spawn((
                Text::new(format!(
                    "• {}{} [{:?}]",
                    res.room_label, guest_label, res.status
                )),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.95, 0.85)),
            ));
        }

        parent.spawn((
            Text::new("Tip: Build rooms + reception, press O to open; reservations will flow in future updates."),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.8, 1.0)),
        ));
    });
}
