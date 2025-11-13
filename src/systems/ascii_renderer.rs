use bevy::prelude::*;
use crate::components::{self, *};

// ASCII characters for different entity types
const PAWN_CHAR: char = '@';
const WALL_CHAR: char = '#';
const DOOR_CHAR: char = '+';
const WINDOW_CHAR: char = '=';
const BLUEPRINT_CHAR: char = '▒';

#[derive(Component)]
pub struct AsciiSprite {
    pub character: char,
    pub color: Color,
}

pub struct AsciiRendererPlugin;

impl Plugin for AsciiRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            add_ascii_to_pawns,
            add_ascii_to_walls,
            add_ascii_to_doors,
            add_ascii_to_windows,
            add_ascii_to_blueprints,
            render_ascii_sprites,
        ));
    }
}

// Add ASCII component to pawns that don't have one
fn add_ascii_to_pawns(
    mut commands: Commands,
    query: Query<Entity, (With<Pawn>, Without<AsciiSprite>)>,
) {
    for entity in &query {
        commands.entity(entity).insert(AsciiSprite {
            character: PAWN_CHAR,
            color: Color::srgb(0.2, 0.6, 0.8),
        });
    }
}

fn add_ascii_to_walls(
    mut commands: Commands,
    query: Query<Entity, (With<Wall>, Without<AsciiSprite>)>,
) {
    for entity in &query {
        commands.entity(entity).insert(AsciiSprite {
            character: WALL_CHAR,
            color: Color::srgb(0.5, 0.5, 0.5),
        });
    }
}

fn add_ascii_to_doors(
    mut commands: Commands,
    query: Query<Entity, (With<Door>, Without<AsciiSprite>)>,
) {
    for entity in &query {
        commands.entity(entity).insert(AsciiSprite {
            character: DOOR_CHAR,
            color: Color::srgb(0.4, 0.3, 0.2),
        });
    }
}

fn add_ascii_to_windows(
    mut commands: Commands,
    query: Query<Entity, (With<components::Window>, Without<AsciiSprite>)>,
) {
    for entity in &query {
        commands.entity(entity).insert(AsciiSprite {
            character: WINDOW_CHAR,
            color: Color::srgb(0.6, 0.8, 1.0),
        });
    }
}

fn add_ascii_to_blueprints(
    mut commands: Commands,
    query: Query<(Entity, &Blueprint), Without<AsciiSprite>>,
) {
    for (entity, blueprint) in &query {
        let color = match blueprint.building_type {
            BlueprintType::Wall => Color::srgba(0.5, 0.5, 0.5, 0.5),
            BlueprintType::Door => Color::srgba(0.4, 0.3, 0.2, 0.5),
            BlueprintType::Window => Color::srgba(0.6, 0.8, 1.0, 0.5),
        };

        commands.entity(entity).insert(AsciiSprite {
            character: BLUEPRINT_CHAR,
            color,
        });
    }
}

// Render ASCII sprites as text (this is a simplified version)
// In a real implementation, you might want to use a custom shader or sprite sheet
fn render_ascii_sprites(
    mut commands: Commands,
    query: Query<(Entity, &AsciiSprite), (Changed<AsciiSprite>, Without<Text2d>)>,
) {
    for (entity, ascii) in &query {
        // Remove old text if it exists
        commands.entity(entity).despawn_descendants();

        // Spawn text as a child
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Text2d::new(ascii.character.to_string()),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(ascii.color),
                Transform::from_xyz(0.0, 0.0, 1.0),
            ));
        });
    }
}
