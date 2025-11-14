use crate::components::{self, *};
use bevy::prelude::*;

const BLUEPRINT_CHAR: char = 'B';

#[derive(Component)]
pub struct AsciiSprite {
    pub character: char,
    pub color: Color,
}

pub struct AsciiRendererPlugin;

impl Plugin for AsciiRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Blueprints are now rendered as translucent white meshes, no ASCII needed
                render_ascii_sprites,
                render_wall_projections,
            ),
        );
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

fn render_ascii_sprites(
    mut commands: Commands,
    query: Query<(Entity, &AsciiSprite), (Changed<AsciiSprite>, Without<Text2d>)>,
) {
    for (entity, ascii) in &query {
        commands.entity(entity).despawn_descendants();
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

// Marker components for projection visuals
#[derive(Component)]
struct WallProjectionVisual;

#[derive(Component)]
struct WallProjectionVisualized;

// Render wall projections for RimWorld-style depth using shaded mesh elements
fn render_wall_projections(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut wall_sets: ParamSet<(
        Query<
            (Entity, &WallProjection, Option<&Children>),
            (
                Or<(With<Wall>, With<components::Window>)>,
                Changed<WallProjection>,
            ),
        >,
        Query<
            (Entity, &WallProjection, Option<&Children>),
            (
                Or<(With<Wall>, With<components::Window>)>,
                Without<WallProjectionVisualized>,
            ),
        >,
    )>,
    projection_visual_query: Query<Entity, With<WallProjectionVisual>>,
) {
    use bevy::sprite::*;

    const TILE_SIZE: f32 = 16.0;
    const SHADE_THICKNESS: f32 = TILE_SIZE * 0.25; // Top shadow thickness
    const SIDE_SHADE_WIDTH: f32 = TILE_SIZE * 0.25; // Side shadow width
    const SIDE_SHADE_HEIGHT: f32 = TILE_SIZE;
    const HALF_TILE: f32 = TILE_SIZE / 2.0;
    // RimWorld-style consistent shadow colors - fully opaque to prevent stacking/blending
    const NORTH_SHADE_COLOR: Color = Color::srgb(0.1, 0.08, 0.06);  // Top shadow
    const SIDE_SHADE_COLOR: Color = Color::srgb(0.12, 0.10, 0.08);  // Consistent side shadows
    const FORCE_ALL_PROJECTIONS: bool = false;

    let mut rebuild = |entity: Entity,
                       projection: &WallProjection,
                       children: Option<&Children>,
                       commands: &mut Commands| {
        if let Some(children) = children {
            for child in children.iter().copied() {
                if projection_visual_query.get(child).is_ok() {
                    commands.entity(child).despawn_recursive();
                }
            }
        }

        let active_projection = if FORCE_ALL_PROJECTIONS {
            WallProjection {
                north: true,
                east: true,
                west: true,
            }
        } else {
            *projection
        };

        commands.entity(entity).insert(WallProjectionVisualized);

        commands.entity(entity).with_children(|parent| {
            if active_projection.north {
                parent.spawn((
                    Mesh2d(meshes.add(Rectangle::new(TILE_SIZE, SHADE_THICKNESS))),
                    MeshMaterial2d(materials.add(NORTH_SHADE_COLOR)),
                    Transform::from_xyz(0.0, HALF_TILE - SHADE_THICKNESS / 2.0, 0.15),
                    WallProjectionVisual,
                ));
            }

            // Side shadows are always full height and centered for consistency
            if active_projection.east {
                parent.spawn((
                    Mesh2d(meshes.add(Rectangle::new(SIDE_SHADE_WIDTH, SIDE_SHADE_HEIGHT))),
                    MeshMaterial2d(materials.add(SIDE_SHADE_COLOR)),
                    Transform::from_xyz(
                        HALF_TILE - SIDE_SHADE_WIDTH / 2.0,
                        0.0,
                        0.1,
                    ),
                    WallProjectionVisual,
                ));
            }

            if active_projection.west {
                parent.spawn((
                    Mesh2d(meshes.add(Rectangle::new(SIDE_SHADE_WIDTH, SIDE_SHADE_HEIGHT))),
                    MeshMaterial2d(materials.add(SIDE_SHADE_COLOR)),
                    Transform::from_xyz(
                        -HALF_TILE + SIDE_SHADE_WIDTH / 2.0,
                        0.0,
                        0.1,
                    ),
                    WallProjectionVisual,
                ));
            }
        });
    };

    for (entity, projection, children) in wall_sets.p0().iter() {
        rebuild(entity, projection, children, &mut commands);
    }

    for (entity, projection, children) in wall_sets.p1().iter() {
        rebuild(entity, projection, children, &mut commands);
    }
}
