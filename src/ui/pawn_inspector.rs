use bevy::prelude::*;

use crate::components::{Mood, Pawn, PawnAttributes, PawnProfile, PawnProgression};
use crate::systems::SelectedPawn;

#[derive(Component)]
struct PawnInspectorPanel;

#[derive(Component)]
struct PawnInspectorContent;

pub struct PawnInspectorPlugin;

impl Plugin for PawnInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_pawn_inspector).add_systems(
            Update,
            (update_panel_visibility, rebuild_pawn_inspector_content),
        );
    }
}

fn setup_pawn_inspector(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                bottom: Val::Px(20.0),
                width: Val::Px(380.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.06, 0.07, 0.92)),
            BorderRadius::all(Val::Px(8.0)),
            BorderColor(Color::srgb(0.3, 0.6, 0.7)),
            PawnInspectorPanel,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                PawnInspectorContent,
            ));
        });
}

fn update_panel_visibility(
    selected: Res<SelectedPawn>,
    mut panel_query: Query<&mut Node, With<PawnInspectorPanel>>,
) {
    if !selected.is_changed() {
        return;
    }

    if let Ok(mut node) = panel_query.get_single_mut() {
        node.display = if selected.entity().is_some() {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn rebuild_pawn_inspector_content(
    mut commands: Commands,
    selected: Res<SelectedPawn>,
    content_query: Query<Entity, With<PawnInspectorContent>>,
    pawns: Query<(
        Ref<Pawn>,
        Ref<PawnProfile>,
        Ref<PawnAttributes>,
        Ref<PawnProgression>,
        Ref<Mood>,
    )>,
    children_query: Query<&Children>,
) {
    let Some(selected_entity) = selected.entity() else {
        return;
    };

    let Ok(content_entity) = content_query.get_single() else {
        return;
    };

    let Ok((pawn, profile, attributes, progression, mood)) = pawns.get(selected_entity) else {
        return;
    };

    let changed = selected.is_changed()
        || pawn.is_changed()
        || profile.is_changed()
        || attributes.is_changed()
        || progression.is_changed();

    if !changed {
        return;
    }

    if let Ok(children) = children_query.get(content_entity) {
        for &child in children.iter() {
            commands.entity(child).despawn_recursive();
        }
    }

    let xp_percentage = progression.progress_ratio() * 100.0;

    commands.entity(content_entity).with_children(|parent| {
        parent.spawn((
            Text::new(format!("{} • Level {}", pawn.name, progression.level)),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.95, 0.95)),
        ));

        parent.spawn((
            Text::new(format!(
                "XP: {:>5.1}/{:>5.1} ({:.0}%)",
                progression.experience, progression.next_level_experience, xp_percentage
            )),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.9, 0.9)),
        ));

        parent.spawn((
            Text::new(format!("\"{}\"", profile.tagline)),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.65)),
        ));

        parent.spawn((
            Text::new(format!(
                "{} – {}",
                profile.background.title, profile.background.description
            )),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.85, 0.85, 0.85)),
        ));

        parent.spawn((
            Text::new(format!(
                "Mood: {:.0} ({})",
                mood.current,
                if mood.moodlets.is_empty() {
                    "stable".to_string()
                } else {
                    format!("{} moodlets", mood.moodlets.len())
                }
            )),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.75, 0.9, 0.75)),
        ));

        parent.spawn((
            Text::new(format!(
                "Prefers the {} shift • Signature skill: {}",
                profile.preferred_shift,
                profile.background.signature_skill.display_name()
            )),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.8, 0.9)),
        ));

        parent
            .spawn((Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },))
            .with_children(|skills| {
                skills.spawn((
                    Text::new("Hospitality Attributes"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.95, 0.95)),
                ));

                for (skill, rating) in attributes.iter() {
                    skills
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Text::new(skill.display_name().to_string()),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));

                            row.spawn((
                                Text::new(format!(
                                    "Lv {:02} • {}",
                                    rating.level,
                                    rating.passion.display_name()
                                )),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.8, 0.85, 0.95)),
                            ));
                        });
                }
            });
    });
}
