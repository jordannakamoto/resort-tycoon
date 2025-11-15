use bevy::prelude::*;

mod components;
mod systems;
mod ui;

use systems::{
    AsciiRendererPlugin, BuildingPlugin, CameraPlugin, EconomyPlugin, GridPlugin, PawnPlugin,
    RoomDetectionPlugin, SaveLoadPlugin, TimeControlPlugin, WorkPlugin, ZoneVisualizationPlugin,
};
use ui::{MoneyDisplayPlugin, SaveLoadPanelPlugin, SpeedControlPlugin, ToolbarPlugin, WorkAssignmentsPlugin};

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
            CameraPlugin,
            ToolbarPlugin,
            SpeedControlPlugin,
            MoneyDisplayPlugin,
            WorkAssignmentsPlugin,
            SaveLoadPanelPlugin,
            BuildingPlugin,
            SaveLoadPlugin,
            PawnPlugin,
            WorkPlugin,
            AsciiRendererPlugin,
            TimeControlPlugin,
            EconomyPlugin,
            RoomDetectionPlugin,
            ZoneVisualizationPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn camera with pan/zoom capability
    use systems::CameraController;
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, 0.0, 999.9),
        CameraController::default(),
    ));
}
