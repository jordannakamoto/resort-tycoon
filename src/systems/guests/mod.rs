use bevy::prelude::*;
use rand::prelude::*;
use std::collections::HashSet;

use crate::components::*;
use crate::systems::{grid::{grid_to_world, GridSettings}, ResortStatus, PAWN_SIZE};

pub struct GuestPlugin;

impl Plugin for GuestPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GuestGenerator>()
            .init_resource::<GuestArrivalState>()
            .add_systems(
                Update,
                (
                    spawn_guests_on_open,
                    reset_arrivals_on_close,
                    assign_guest_beds,
                    assign_guest_bathroom_breaks,
                    tick_guest_needs,
                ),
            );
    }
}

#[derive(Resource)]
struct GuestGenerator {
    rng: StdRng,
}

impl Default for GuestGenerator {
    fn default() -> Self {
        Self {
            rng: StdRng::from_entropy(),
        }
    }
}

#[derive(Resource, Default)]
struct GuestArrivalState {
    spawned_on_open: bool,
}

#[derive(Component, Default)]
struct GuestBehavior {
    pub has_bed: bool,
    pub using_fixture: bool,
}

fn spawn_guests_on_open(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    resort_status: Res<ResortStatus>,
    mut arrival_state: ResMut<GuestArrivalState>,
    mut generator: ResMut<GuestGenerator>,
    mut existing_guests: Query<Entity, With<Guest>>,
) {
    // Only spawn once when the resort becomes open
    if !resort_status.is_open || arrival_state.spawned_on_open || !resort_status.is_changed() {
        return;
    }

    // Clear existing guests when reopening
    for entity in &mut existing_guests {
        commands.entity(entity).despawn_recursive();
    }

    arrival_state.spawned_on_open = true;

    let guest_count = 4;
    for i in 0..guest_count {
        let guest = generator.generate_guest();
        let y_offset = (i as f32 - 1.0) * PAWN_SIZE * 1.8;

        commands.spawn((
            Mesh2d(meshes.add(Circle::new(PAWN_SIZE * 0.35))),
            MeshMaterial2d(materials.add(guest.color)),
            Transform::from_xyz(-200.0, y_offset, 10.0),
            Pawn {
                name: guest.name.clone(),
                move_speed: guest.move_speed,
            },
            Guest {
                name: guest.name,
                tier: guest.tier,
            },
            GridPosition::new(0, 0),
            Mood::default(),
            GuestNeeds::default(),
            GuestBehavior::default(),
        ));
    }
}

struct GuestData {
    name: String,
    tier: GuestTier,
    move_speed: f32,
    color: Color,
}

impl GuestGenerator {
    fn generate_guest(&mut self) -> GuestData {
        let first_names = ["Sloan", "Harper", "Luca", "Ren", "Mila", "Eden", "Arlo", "Tess"];
        let last_names = ["Harbor", "Pier", "Cove", "Reef", "Marlin", "Seabrook", "Isle", "Crest"];
        let tiers = [GuestTier::Budget, GuestTier::Standard, GuestTier::Luxury];

        let first = first_names
            .choose(&mut self.rng)
            .copied()
            .unwrap_or("Harper");
        let last = last_names
            .choose(&mut self.rng)
            .copied()
            .unwrap_or("Harbor");
        let tier = *tiers.choose(&mut self.rng).unwrap_or(&GuestTier::Standard);
        let name = format!("{first} {last}");

        let color = match tier {
            GuestTier::Budget => Color::srgb(0.5, 0.7, 0.9),
            GuestTier::Standard => Color::srgb(0.6, 0.8, 0.6),
            GuestTier::Luxury => Color::srgb(0.9, 0.8, 0.4),
        };

        GuestData {
            name,
            tier,
            move_speed: self.rng.gen_range(75.0..110.0),
            color,
        }
    }
}

fn reset_arrivals_on_close(
    resort_status: Res<ResortStatus>,
    mut arrival_state: ResMut<GuestArrivalState>,
    mut guests: Query<Entity, With<Guest>>,
    mut commands: Commands,
) {
    if !resort_status.is_open && arrival_state.spawned_on_open {
        arrival_state.spawned_on_open = false;
        for entity in &mut guests {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Assign a free bed to each guest if available.
fn assign_guest_beds(
    mut commands: Commands,
    mut beds: Query<(Entity, &mut BedClaim, &GridPosition)>,
    mut guests: Query<(Entity, &Transform, &mut GuestBehavior), (With<Guest>, Without<MovementTarget>)>,
    grid_settings: Res<GridSettings>,
) {
    for (guest_entity, transform, mut behavior) in &mut guests {
        if behavior.has_bed {
            continue;
        }

        if let Some((_, mut claim, bed_pos)) = beds
            .iter_mut()
            .find(|(_, claim, _)| claim.claimed_by.is_none())
        {
            claim.claimed_by = Some(guest_entity);
            behavior.has_bed = true;

            let world_target = grid_to_world(
                IVec2::new(bed_pos.x, bed_pos.y),
                grid_settings.tile_size,
                grid_settings.width,
                grid_settings.height,
            );
            commands.entity(guest_entity).insert(MovementTarget {
                target: world_target,
            });
            // Lift them slightly above current z to keep order
            commands
                .entity(guest_entity)
                .insert(Transform::from_xyz(
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z,
                ));

        }
    }
}

/// Simple bladder need: seek any bathroom fixture when too high.
fn assign_guest_bathroom_breaks(
    mut commands: Commands,
    mut fixtures: Query<(Entity, &mut FixtureInUse, &GridPosition), With<FixtureInUse>>,
    mut guests: Query<(Entity, &GuestNeeds, &mut GuestBehavior), With<Guest>>,
    grid_settings: Res<GridSettings>,
) {
    for (guest_entity, needs, mut behavior) in &mut guests {
        if needs.bladder < 70.0 || behavior.using_fixture {
            continue;
        }

        if let Some((fixture_entity, mut use_state, fixture_pos)) = fixtures
            .iter_mut()
            .find(|(_, f, _)| f.used_by.is_none())
        {
            use_state.used_by = Some(guest_entity);
            use_state.timer = 5.0;
            behavior.using_fixture = true;

            let world_target = grid_to_world(
                IVec2::new(fixture_pos.x, fixture_pos.y),
                grid_settings.tile_size,
                grid_settings.width,
                grid_settings.height,
            );
            commands.entity(guest_entity).insert(MovementTarget {
                target: world_target,
            });

            commands.entity(fixture_entity).insert(use_state.clone());
        }
    }
}

fn tick_guest_needs(
    time: Res<Time>,
    mut guests: Query<(Entity, &mut GuestNeeds, &mut Mood, Option<&mut GuestBehavior>), With<Guest>>,
    mut fixtures: Query<&mut FixtureInUse>,
) {
    let delta = time.delta_secs();

    // Update fixtures once per frame and track who is using them
    let mut in_use: HashSet<Entity> = HashSet::new();
    for mut fixture in &mut fixtures {
        if let Some(user) = fixture.used_by {
            fixture.timer -= delta;
            if fixture.timer <= 0.0 {
                fixture.used_by = None;
            } else {
                in_use.insert(user);
            }
        }
    }

    for (guest_entity, mut needs, mut mood, behavior_opt) in &mut guests {
        needs.rest = (needs.rest + delta * 2.0).clamp(0.0, 100.0);
        needs.bladder = (needs.bladder + delta * 3.0).clamp(0.0, 100.0);

        let mut mood_delta = 0.0;
        if needs.rest > 80.0 {
            mood_delta -= 0.2;
        }
        if needs.bladder > 80.0 {
            mood_delta -= 0.3;
        }
        mood.moodlets.push(Moodlet {
            name: "Base comfort".to_string(),
            value: mood_delta,
            remaining_seconds: 5.0,
        });
        mood.recompute();

        if let Some(mut behavior) = behavior_opt {
            if behavior.using_fixture && !in_use.contains(&guest_entity) {
                behavior.using_fixture = false;
                needs.bladder = 10.0;
            }
        }
    }
}
