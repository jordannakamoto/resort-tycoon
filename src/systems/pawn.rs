use crate::components::*;
use crate::systems::grid::*;
use bevy::prelude::*;
use bevy::sprite::*;

const PAWN_SIZE: f32 = TILE_SIZE * 2.0; // Pawns occupy 2x2 tiles

pub struct PawnPlugin;

impl Plugin for PawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_initial_pawns)
            .add_systems(Update, (move_pawns, update_pawn_positions));
    }
}

fn spawn_initial_pawns(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Spawn 3 initial worker pawns
    for i in 0..3 {
        let x_offset = (i as f32 - 1.0) * PAWN_SIZE * 1.5;

        commands.spawn((
            Mesh2d(meshes.add(Circle::new(PAWN_SIZE * 0.4))),
            MeshMaterial2d(materials.add(Color::srgb(0.2, 0.6, 0.8))),
            Transform::from_xyz(x_offset, 0.0, 10.0),
            Pawn {
                name: format!("Worker {}", i + 1),
                move_speed: 100.0,
            },
            GridPosition::new(0, 0),
            CurrentJob::default(),
            WorkAssignments::default(),
        ));
    }
}

fn move_pawns(mut query: Query<(&mut Transform, &Pawn, &MovementTarget)>, time: Res<Time>) {
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

fn update_pawn_positions(
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
