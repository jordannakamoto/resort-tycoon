use bevy::prelude::*;

use crate::{components::Pawn, ui::UiInputBlocker};

use super::PAWN_SIZE;

#[derive(Resource, Default)]
pub struct SelectedPawn {
    entity: Option<Entity>,
}

impl SelectedPawn {
    pub fn entity(&self) -> Option<Entity> {
        self.entity
    }

    pub fn set(&mut self, entity: Option<Entity>) {
        if self.entity != entity {
            self.entity = entity;
        }
    }

    pub fn clear(&mut self) {
        self.set(None);
    }
}

pub(super) fn handle_pawn_selection(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera>>,
    pawns: Query<(Entity, &Transform), With<Pawn>>,
    mut selected: ResMut<SelectedPawn>,
    ui_blocker: Res<UiInputBlocker>,
) {
    if ui_blocker.block_world_input {
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape) || mouse_button.just_pressed(MouseButton::Right) {
        selected.clear();
        return;
    }

    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        selected.clear();
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        selected.clear();
        return;
    };

    let mut closest: Option<(Entity, f32)> = None;
    let selection_radius = PAWN_SIZE * 0.6;

    for (entity, transform) in &pawns {
        let distance = transform.translation.truncate().distance(world_position);
        if distance <= selection_radius {
            match closest {
                Some((_, best_distance)) if distance >= best_distance => {}
                _ => closest = Some((entity, distance)),
            }
        }
    }

    if let Some((entity, _)) = closest {
        selected.set(Some(entity));
    } else {
        selected.clear();
    }
}
