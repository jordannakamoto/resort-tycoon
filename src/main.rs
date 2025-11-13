use bevy::prelude::*;

mod ui;
mod systems;
mod components;

use ui::ToolbarPlugin;
use systems::{GridPlugin, BuildingPlugin, PawnPlugin, WorkPlugin, AsciiRendererPlugin};

// Tile system constants
// In RimWorld, a pawn occupies 1 tile. In our game, a pawn will occupy 2x2 tiles (4 tiles)
// This gives us finer granularity for smaller objects and installations
const PAWN_TILE_SIZE: usize = 2; // A pawn occupies 2x2 tiles

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Resort Tycoon".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            GridPlugin,
            ToolbarPlugin,
            BuildingPlugin,
            PawnPlugin,
            WorkPlugin,
            AsciiRendererPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn camera with pan/zoom capability
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, 0.0, 999.9),
    ));
}
