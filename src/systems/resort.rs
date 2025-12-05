use bevy::prelude::*;

use crate::components::{ReceptionConsole, Zone, ZoneType};

#[derive(Resource, Debug, Default)]
pub struct ResortStatus {
    pub is_open: bool,
    pub requirements_met: bool,
    pub last_status: String,
}

pub struct ResortPlugin;

impl Plugin for ResortPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ResortStatus>()
            .add_systems(Update, (evaluate_resort_readiness, handle_open_toggle));
    }
}

fn evaluate_resort_readiness(
    mut status: ResMut<ResortStatus>,
    zone_query: Query<&Zone>,
    reception_query: Query<&ReceptionConsole>,
) {
    let has_guest_room = zone_query
        .iter()
        .any(|zone| zone.zone_type == ZoneType::GuestBedroom);
    let has_reception = !reception_query.is_empty();
    let ready = has_guest_room && has_reception;

    let mut status_changed = false;

    if status.requirements_met != ready {
        status.requirements_met = ready;
        status_changed = true;
    }

    if status.is_open && !ready {
        status.is_open = false;
        status_changed = true;
        status.last_status = "Closed: missing guest rooms or reception desk".to_string();
    } else if ready && !status.is_open {
        status.last_status = "Ready: press O to open resort".to_string();
    }

    if status_changed {
        info!(
            "Resort status -> open: {}, ready: {} ({})",
            status.is_open, status.requirements_met, status.last_status
        );
    }
}

fn handle_open_toggle(keyboard: Res<ButtonInput<KeyCode>>, mut status: ResMut<ResortStatus>) {
    if keyboard.just_pressed(KeyCode::KeyO) {
        if status.requirements_met {
            status.is_open = !status.is_open;
            status.last_status = if status.is_open {
                "Resort opened for guests".to_string()
            } else {
                "Resort closed".to_string()
            };
            info!("Resort is now {}.", if status.is_open { "open" } else { "closed" });
        } else {
            status.last_status =
                "Cannot open: add a guest room and a reception computer".to_string();
            info!("{}", status.last_status);
        }
    }
}
