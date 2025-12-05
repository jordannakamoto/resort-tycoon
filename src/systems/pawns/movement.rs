use bevy::prelude::*;

use crate::components::*;
use crate::systems::grid::{world_to_grid, GridSettings};

pub(super) fn move_pawns(
    mut query: Query<(&mut Transform, &Pawn, &MovementTarget)>,
    time: Res<Time>,
) {
    for (mut transform, pawn, target) in &mut query {
        let current_pos = transform.translation.truncate();
        let direction = target.target - current_pos;
        let distance = direction.length();

        if distance > 1.0 {
            let movement = direction.normalize() * pawn.move_speed * time.delta_secs();
            if movement.length() < distance {
                transform.translation += movement.extend(0.0);
            } else {
                transform.translation = target.target.extend(transform.translation.z);
            }
        }
    }
}

pub(super) fn update_pawn_positions(
    mut query: Query<(&Transform, &mut GridPosition), (With<Pawn>, Changed<Transform>)>,
    grid_settings: Res<GridSettings>,
) {
    for (transform, mut grid_pos) in &mut query {
        let pos = transform.translation.truncate();
        if let Some(new_grid_pos) = world_to_grid(
            pos,
            grid_settings.tile_size,
            grid_settings.width,
            grid_settings.height,
        ) {
            grid_pos.x = new_grid_pos.x;
            grid_pos.y = new_grid_pos.y;
        }
    }
}
