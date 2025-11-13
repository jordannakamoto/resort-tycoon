use bevy::prelude::*;
use bevy::sprite::*;

pub const TILE_SIZE: f32 = 16.0;
pub const GRID_WIDTH: i32 = 100;
pub const GRID_HEIGHT: i32 = 100;

#[derive(Resource)]
pub struct GridSettings {
    pub tile_size: f32,
    pub width: i32,
    pub height: i32,
    pub show_grid: bool,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self {
            tile_size: TILE_SIZE,
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
            show_grid: true,
        }
    }
}

#[derive(Component)]
pub struct GridLines;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GridSettings>()
            .add_systems(Startup, setup_grid)
            .add_systems(Update, update_grid_visibility);
    }
}

fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    grid_settings: Res<GridSettings>,
) {
    let tile_size = grid_settings.tile_size;
    let width = grid_settings.width as f32 * tile_size;
    let height = grid_settings.height as f32 * tile_size;

    // Create vertical lines
    for x in 0..=grid_settings.width {
        let x_pos = x as f32 * tile_size - width / 2.0;

        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(1.0, height))),
            MeshMaterial2d(materials.add(Color::srgba(0.3, 0.3, 0.3, 0.2))),
            Transform::from_xyz(x_pos, 0.0, 0.0),
            GridLines,
        ));
    }

    // Create horizontal lines
    for y in 0..=grid_settings.height {
        let y_pos = y as f32 * tile_size - height / 2.0;

        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(width, 1.0))),
            MeshMaterial2d(materials.add(Color::srgba(0.3, 0.3, 0.3, 0.2))),
            Transform::from_xyz(0.0, y_pos, 0.0),
            GridLines,
        ));
    }
}

fn update_grid_visibility(
    grid_settings: Res<GridSettings>,
    mut query: Query<&mut Visibility, With<GridLines>>,
) {
    if grid_settings.is_changed() {
        for mut visibility in &mut query {
            *visibility = if grid_settings.show_grid {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

// Helper functions for grid coordinate conversion
pub fn world_to_grid(world_pos: Vec2, tile_size: f32, grid_width: i32, grid_height: i32) -> Option<IVec2> {
    let width = grid_width as f32 * tile_size;
    let height = grid_height as f32 * tile_size;

    let grid_x = ((world_pos.x + width / 2.0) / tile_size).floor() as i32;
    let grid_y = ((world_pos.y + height / 2.0) / tile_size).floor() as i32;

    if grid_x >= 0 && grid_x < grid_width && grid_y >= 0 && grid_y < grid_height {
        Some(IVec2::new(grid_x, grid_y))
    } else {
        None
    }
}

pub fn grid_to_world(grid_pos: IVec2, tile_size: f32, grid_width: i32, grid_height: i32) -> Vec2 {
    let width = grid_width as f32 * tile_size;
    let height = grid_height as f32 * tile_size;

    Vec2::new(
        grid_pos.x as f32 * tile_size - width / 2.0 + tile_size / 2.0,
        grid_pos.y as f32 * tile_size - height / 2.0 + tile_size / 2.0,
    )
}
