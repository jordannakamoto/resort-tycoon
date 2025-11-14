use crate::components::*;
use bevy::prelude::*;

const PANEL_WIDTH: f32 = 600.0;
const CELL_SIZE: f32 = 40.0;
const HEADER_HEIGHT: f32 = 30.0;

#[derive(Component)]
pub struct WorkAssignmentsPanel;

#[derive(Component)]
pub struct WorkAssignmentsContent;

#[derive(Component)]
pub struct WorkAssignmentCell {
    pub pawn_entity: Entity,
    pub work_type: WorkType,
}

#[derive(Resource, Default)]
pub struct WorkAssignmentsPanelState {
    pub visible: bool,
}

pub struct WorkAssignmentsPlugin;

impl Plugin for WorkAssignmentsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorkAssignmentsPanelState>()
            .add_systems(Startup, setup_work_assignments_panel)
            .add_systems(
                Update,
                (
                    handle_keyboard_panel_toggle,
                    apply_panel_visibility,
                    update_work_assignments_panel,
                    handle_cell_clicks,
                ),
            );
    }
}

fn setup_work_assignments_panel(mut commands: Commands) {
    // Initially hidden panel
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                top: Val::Px(50.0),
                width: Val::Px(PANEL_WIDTH),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(5.0),
                display: Display::None, // Hidden by default
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.95)),
            WorkAssignmentsPanel,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Work Assignments"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Container used for rebuilding the table contents
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(5.0),
                    ..default()
                },
                WorkAssignmentsContent,
            ));
        });
}

fn handle_keyboard_panel_toggle(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut panel_state: ResMut<WorkAssignmentsPanelState>,
) {
    if keyboard.just_pressed(KeyCode::KeyW) {
        panel_state.visible = !panel_state.visible;
    }
}

fn apply_panel_visibility(
    panel_state: Res<WorkAssignmentsPanelState>,
    mut panel_query: Query<&mut Node, With<WorkAssignmentsPanel>>,
) {
    if !panel_state.is_changed() {
        return;
    }

    if let Ok(mut style) = panel_query.get_single_mut() {
        style.display = if panel_state.visible {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn update_work_assignments_panel(
    mut commands: Commands,
    content_query: Query<Entity, With<WorkAssignmentsContent>>,
    pawn_query: Query<(Entity, &Pawn, &WorkAssignments)>,
    panel_state: Res<WorkAssignmentsPanelState>,
    children_query: Query<&Children>,
) {
    if !panel_state.visible {
        return;
    }

    let Ok(content_entity) = content_query.get_single() else {
        return;
    };

    // Only rebuild when panel visibility changes
    if !panel_state.is_changed() {
        return;
    }

    // Remove old table rows
    if let Ok(children) = children_query.get(content_entity) {
        for &child in children.iter() {
            commands.entity(child).despawn_recursive();
        }
    }

    // Rebuild table
    commands.entity(content_entity).with_children(|parent| {
        // Header row
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(2.0),
                ..default()
            })
            .with_children(|row| {
                // Pawn name column header
                row.spawn((
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(HEADER_HEIGHT),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                ))
                .with_children(|cell| {
                    cell.spawn((
                        Text::new("Pawn"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

                // Work type column headers
                for work_type in WorkType::all() {
                    row.spawn((
                        Node {
                            width: Val::Px(CELL_SIZE),
                            height: Val::Px(HEADER_HEIGHT),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    ))
                    .with_children(|cell| {
                        cell.spawn((
                            Text::new(work_type.name()),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                }
            });

        // Pawn rows
        for (pawn_entity, pawn, assignments) in &pawn_query {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(2.0),
                    ..default()
                })
                .with_children(|row| {
                    // Pawn name
                    row.spawn((
                        Node {
                            width: Val::Px(150.0),
                            height: Val::Px(CELL_SIZE),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    ))
                    .with_children(|cell| {
                        cell.spawn((
                            Text::new(&pawn.name),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });

                    // Work priority cells
                    for work_type in WorkType::all() {
                        let priority = assignments.get_priority(work_type);
                        let bg_color = if priority.is_enabled() {
                            Color::srgb(0.3, 0.5, 0.3) // Green if enabled
                        } else {
                            Color::srgb(0.2, 0.2, 0.2) // Gray if disabled
                        };

                        row.spawn((
                            Button,
                            Node {
                                width: Val::Px(CELL_SIZE),
                                height: Val::Px(CELL_SIZE),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(bg_color),
                            WorkAssignmentCell {
                                pawn_entity,
                                work_type,
                            },
                        ))
                        .with_children(|cell| {
                            cell.spawn((
                                Text::new(priority.display()),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                    }
                });
        }
    });
}

fn handle_cell_clicks(
    mut interaction_query: Query<
        (
            &Interaction,
            &WorkAssignmentCell,
            &mut BackgroundColor,
            &Children,
        ),
        Changed<Interaction>,
    >,
    mut pawn_query: Query<&mut WorkAssignments>,
    mut text_query: Query<&mut Text>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    for (interaction, cell, mut bg_color, children) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            if let Ok(mut assignments) = pawn_query.get_mut(cell.pawn_entity) {
                let shift_pressed =
                    keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

                if shift_pressed {
                    // Shift-clicks push the priority towards the disabled state
                    assignments.increase_priority(cell.work_type);
                } else {
                    // Regular clicks decrease the priority towards the highest slot
                    assignments.decrease_priority(cell.work_type);
                }

                let priority = assignments.get_priority(cell.work_type);

                // Update background color
                *bg_color = if priority.is_enabled() {
                    Color::srgb(0.3, 0.5, 0.3).into()
                } else {
                    Color::srgb(0.2, 0.2, 0.2).into()
                };

                // Update text
                for &child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(child) {
                        **text = priority.display();
                    }
                }
            }
        }
    }
}
