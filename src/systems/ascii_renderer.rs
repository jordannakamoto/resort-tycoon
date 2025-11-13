use bevy::prelude::*;
use crate::components::{self, *};

// ASCII characters for different entity types
const PAWN_CHAR: char = '@';
const WALL_CHAR: char = '#';
const DOOR_CHAR: char = '+';
const WINDOW_CHAR: char = '=';
const FLOOR_CHAR: char = '.';
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
            add_ascii_to_floors,
            add_ascii_to_blueprints,
            add_ascii_to_furniture,
            update_door_ascii,
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
    query: Query<(Entity, &Door), Without<AsciiSprite>>,
) {
    for (entity, door) in &query {
        let character = match door.state {
            DoorState::Open => '/',    // Open door shows as '/'
            DoorState::Closed => '+',  // Closed door shows as '+'
        };

        commands.entity(entity).insert(AsciiSprite {
            character,
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

fn add_ascii_to_floors(
    mut commands: Commands,
    query: Query<(Entity, &Floor), Without<AsciiSprite>>,
) {
    for (entity, floor) in &query {
        commands.entity(entity).insert(AsciiSprite {
            character: FLOOR_CHAR,
            color: floor.floor_type.color(),
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
            BlueprintType::Door(_) => Color::srgba(0.4, 0.3, 0.2, 0.5),
            BlueprintType::Window => Color::srgba(0.6, 0.8, 1.0, 0.5),
            BlueprintType::Floor(floor_type) => floor_type.color().with_alpha(0.5),
            BlueprintType::Furniture(furniture_type) => furniture_type.color().with_alpha(0.5),
        };

        commands.entity(entity).insert(AsciiSprite {
            character: BLUEPRINT_CHAR,
            color,
        });
    }
}

// Add ASCII to furniture
fn add_ascii_to_furniture(
    mut commands: Commands,
    bed_query: Query<(Entity, &Bed), Without<AsciiSprite>>,
    desk_query: Query<Entity, (With<Desk>, Without<AsciiSprite>)>,
    chair_query: Query<Entity, (With<Chair>, Without<AsciiSprite>)>,
    dresser_query: Query<Entity, (With<Dresser>, Without<AsciiSprite>)>,
    nightstand_query: Query<Entity, (With<Nightstand>, Without<AsciiSprite>)>,
    console_query: Query<Entity, (With<ReceptionConsole>, Without<AsciiSprite>)>,
) {
    use crate::components::FurnitureType;

    // Use dark color for furniture ASCII so it contrasts with the colored background
    let ascii_color = Color::srgb(0.2, 0.15, 0.1);

    // Beds
    for (entity, bed) in &bed_query {
        let furniture_type = FurnitureType::Bed(bed.bed_type);
        commands.entity(entity).insert(AsciiSprite {
            character: furniture_type.ascii_char(),
            color: ascii_color,
        });
    }

    // Desks
    for entity in &desk_query {
        let furniture_type = FurnitureType::Desk;
        commands.entity(entity).insert(AsciiSprite {
            character: furniture_type.ascii_char(),
            color: ascii_color,
        });
    }

    // Chairs
    for entity in &chair_query {
        let furniture_type = FurnitureType::Chair;
        commands.entity(entity).insert(AsciiSprite {
            character: furniture_type.ascii_char(),
            color: ascii_color,
        });
    }

    // Dressers
    for entity in &dresser_query {
        let furniture_type = FurnitureType::Dresser;
        commands.entity(entity).insert(AsciiSprite {
            character: furniture_type.ascii_char(),
            color: ascii_color,
        });
    }

    // Nightstands
    for entity in &nightstand_query {
        let furniture_type = FurnitureType::Nightstand;
        commands.entity(entity).insert(AsciiSprite {
            character: furniture_type.ascii_char(),
            color: ascii_color,
        });
    }

    // Reception Consoles
    for entity in &console_query {
        let furniture_type = FurnitureType::ReceptionConsole;
        commands.entity(entity).insert(AsciiSprite {
            character: furniture_type.ascii_char(),
            color: ascii_color,
        });
    }
}

// Update door ASCII when state changes
fn update_door_ascii(
    mut query: Query<(&Door, &mut AsciiSprite), Changed<Door>>,
) {
    for (door, mut ascii) in &mut query {
        ascii.character = match door.state {
            DoorState::Open => '/',
            DoorState::Closed => '+',
        };
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
