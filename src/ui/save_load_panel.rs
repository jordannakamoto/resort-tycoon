use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use std::fs;
use std::path::Path;

use crate::components::*;
use crate::systems::grid::GridSettings;
use crate::systems::{save_load::SaveLoadConfig, BuildingMap};

#[derive(SystemParam)]
struct ClearQueries<'w, 's> {
    walls: Query<'w, 's, Entity, With<Wall>>,
    floors: Query<'w, 's, Entity, With<Floor>>,
    doors: Query<'w, 's, Entity, With<Door>>,
    furniture: Query<'w, 's, Entity, With<Furniture>>,
    blueprints: Query<'w, 's, Entity, With<Blueprint>>,
    construction_jobs: Query<'w, 's, Entity, With<ConstructionJob>>,
    deconstruction_jobs: Query<'w, 's, Entity, With<DeconstructionJob>>,
    markers: Query<'w, 's, Entity, With<DeconstructionMarker>>,
}

#[derive(Component)]
pub struct SaveLoadPanel;

#[derive(Component)]
pub struct SaveButton;

#[derive(Component)]
pub struct LoadButton {
    pub filename: String,
}

#[derive(Component)]
pub struct RenameButton {
    pub old_filename: String,
}

#[derive(Component)]
pub struct DeleteButton {
    pub filename: String,
}

#[derive(Component)]
pub struct NewSaveButton;

#[derive(Component)]
pub struct SaveNameInput;

#[derive(Component)]
pub struct SaveListContainer;

#[derive(Component)]
pub struct SaveNameText;

#[derive(Resource, Default)]
pub struct SaveLoadPanelState {
    pub visible: bool,
    pub current_save_name: String,
    pub saves_list: Vec<String>,
    pub editing_mode: bool,
}

impl SaveLoadPanelState {
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn refresh_saves_list(&mut self) {
        self.saves_list.clear();
        if let Ok(entries) = fs::read_dir("assets/saves") {
            for entry in entries.flatten() {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.ends_with(".json") {
                        // Only add if the name without .json extension is not empty
                        let name_without_ext = filename.trim_end_matches(".json");
                        if !name_without_ext.is_empty() {
                            self.saves_list.push(filename.to_string());
                            info!("Added save file: {}", filename);
                        } else {
                            warn!("Skipping save file with empty name: {}", filename);
                        }
                    }
                }
            }
        }
        self.saves_list.sort();
        info!("Total saves in list: {}", self.saves_list.len());
    }
}

pub struct SaveLoadPanelPlugin;

impl Plugin for SaveLoadPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveLoadPanelState>()
            .add_systems(Startup, setup_save_load_panel)
            .add_systems(
                Update,
                (
                    update_panel_visibility,
                    handle_save_button,
                    handle_load_button,
                    handle_rename_button,
                ),
            )
            .add_systems(
                Update,
                (
                    handle_delete_button,
                    handle_keyboard_input,
                    update_save_name_display,
                    update_save_list,
                ),
            );
    }
}

fn setup_save_load_panel(mut commands: Commands, mut state: ResMut<SaveLoadPanelState>) {
    // Refresh saves list on startup
    state.refresh_saves_list();
    state.current_save_name = "my_resort".to_string();

    // Create the save/load panel (hidden by default)
    commands
        .spawn((
            Node {
                width: Val::Px(400.0),
                height: Val::Px(500.0),
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                top: Val::Px(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(15.0)),
                row_gap: Val::Px(10.0),
                display: Display::None, // Hidden by default
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.95)),
            SaveLoadPanel,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Save / Load"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Save name input section
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Save name:"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Input field (we'll display the name here)
                    parent.spawn((
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(30.0),
                            padding: UiRect::all(Val::Px(5.0)),
                            justify_content: JustifyContent::Start,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                        SaveNameInput,
                    )).with_children(|parent| {
                        parent.spawn((
                            Text::new("my_resort"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            SaveNameText,
                        ));
                    });
                });

            // Save button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
                    SaveButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Save Game"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Separator
            parent.spawn((
                Text::new("Saved Games:"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Saves list container (scrollable)
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(300.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(5.0),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    SaveListContainer,
                ))
                .with_children(|parent| {
                    // List will be populated dynamically
                });
        });
}

fn update_panel_visibility(
    state: Res<SaveLoadPanelState>,
    mut panel_query: Query<&mut Node, With<SaveLoadPanel>>,
    mut last_visible: Local<bool>,
) {
    // Check if visibility actually changed
    if state.visible != *last_visible {
        *last_visible = state.visible;

        for mut style in &mut panel_query {
            style.display = if state.visible {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
}

fn update_save_list(
    mut commands: Commands,
    state: Res<SaveLoadPanelState>,
    list_container_query: Query<Entity, With<SaveListContainer>>,
    mut last_saves_list: Local<Option<Vec<String>>>,
) {
    // Only rebuild if saves list actually changed (different files, not just refreshed)
    let saves_changed = last_saves_list.as_ref()
        .map_or(true, |last| last != &state.saves_list);

    if !saves_changed {
        return;
    }

    *last_saves_list = Some(state.saves_list.clone());

    // Clear all existing children of the container
    let Ok(container) = list_container_query.get_single() else {
        return;
    };

    commands.entity(container).despawn_descendants();

    commands.entity(container).with_children(|parent| {
        info!("Rebuilding save list UI with {} entries", state.saves_list.len());
        for save_name in &state.saves_list {
            let display_name = save_name.trim_end_matches(".json");
            info!("Creating UI entry for: '{}' (display: '{}')", save_name, display_name);
            // Container for each save item
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(40.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(5.0),
                    margin: UiRect::all(Val::Px(2.0)),
                    ..default()
                })
                .with_children(|parent| {
                    // Load button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(60.0),
                                height: Val::Px(35.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
                            LoadButton {
                                filename: save_name.clone(),
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(display_name),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // Rename button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(20.0),
                                height: Val::Px(35.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.4, 0.4, 0.2)),
                            RenameButton {
                                old_filename: save_name.clone(),
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Rename"),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // Delete button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(20.0),
                                height: Val::Px(35.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                            DeleteButton {
                                filename: save_name.clone(),
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Delete"),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        }
    });
}

fn handle_save_button(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<SaveButton>)>,
    state: Res<SaveLoadPanelState>,
    mut config: ResMut<SaveLoadConfig>,
    wall_query: Query<&GridPosition, With<Wall>>,
    floor_query: Query<(&GridPosition, &Floor)>,
    door_query: Query<(&GridPosition, &Door)>,
    furniture_query: Query<(&GridPosition, &Furniture, &FurnitureType, &FurnitureOrientation)>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.15, 0.5, 0.15));

                // Save the game
                let filename = if state.current_save_name.is_empty() {
                    "unnamed_save.json".to_string()
                } else {
                    format!("{}.json", state.current_save_name.trim_end_matches(".json"))
                };

                let path = format!("assets/saves/{}", filename);

                // Use the existing save logic
                use crate::systems::save_load::{collect_save_data, write_save_file, sort_save_data};
                let mut data = collect_save_data(&wall_query, &floor_query, &door_query, &furniture_query);
                sort_save_data(&mut data);

                if let Err(err) = write_save_file(&path, &data) {
                    error!("Failed to save to {}: {}", path, err);
                } else {
                    info!("Saved game to {}", path);
                    config.path = path;
                }
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.25, 0.7, 0.25));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.2, 0.6, 0.2));
            }
        }
    }
}

fn handle_load_button(
    mut interaction_query: Query<
        (&Interaction, &LoadButton, &mut BackgroundColor),
        (Changed<Interaction>, With<LoadButton>),
    >,
    mut config: ResMut<SaveLoadConfig>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    grid_settings: Res<GridSettings>,
    mut building_map: ResMut<BuildingMap>,
    clear_queries: ClearQueries,
    mut state: ResMut<SaveLoadPanelState>,
) {
    for (interaction, load_btn, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.15, 0.15, 0.15));

                // Load the game
                let path = format!("assets/saves/{}", load_btn.filename);
                config.path = path.clone();

                use crate::systems::save_load::{read_or_create_save_file, clear_structures, apply_save_data};

                let (data, source) = read_or_create_save_file(&path);
                clear_structures(
                    &mut commands,
                    &clear_queries.walls,
                    &clear_queries.floors,
                    &clear_queries.doors,
                    &clear_queries.furniture,
                    &clear_queries.blueprints,
                    &clear_queries.construction_jobs,
                    &clear_queries.deconstruction_jobs,
                    &clear_queries.markers,
                );
                apply_save_data(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &asset_server,
                    &grid_settings,
                    &mut building_map,
                    &data,
                );

                info!("Loaded game from {}", source);

                // Update current save name
                state.current_save_name = load_btn.filename.trim_end_matches(".json").to_string();

                // Close panel after loading
                state.visible = false;
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.35, 0.35, 0.35));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
            }
        }
    }
}

fn handle_keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<SaveLoadPanelState>,
) {
    if !state.visible {
        return;
    }

    // Handle character input
    for key in keys.get_just_pressed() {
        match key {
            KeyCode::Backspace => {
                state.current_save_name.pop();
            }
            KeyCode::Space => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('_');
                }
            }
            KeyCode::KeyA => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('a');
                }
            }
            KeyCode::KeyB => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('b');
                }
            }
            KeyCode::KeyC => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('c');
                }
            }
            KeyCode::KeyD => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('d');
                }
            }
            KeyCode::KeyE => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('e');
                }
            }
            KeyCode::KeyF => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('f');
                }
            }
            KeyCode::KeyG => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('g');
                }
            }
            KeyCode::KeyH => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('h');
                }
            }
            KeyCode::KeyI => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('i');
                }
            }
            KeyCode::KeyJ => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('j');
                }
            }
            KeyCode::KeyK => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('k');
                }
            }
            KeyCode::KeyL => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('l');
                }
            }
            KeyCode::KeyM => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('m');
                }
            }
            KeyCode::KeyN => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('n');
                }
            }
            KeyCode::KeyO => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('o');
                }
            }
            KeyCode::KeyP => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('p');
                }
            }
            KeyCode::KeyQ => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('q');
                }
            }
            KeyCode::KeyR => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('r');
                }
            }
            KeyCode::KeyS => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('s');
                }
            }
            KeyCode::KeyT => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('t');
                }
            }
            KeyCode::KeyU => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('u');
                }
            }
            KeyCode::KeyV => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('v');
                }
            }
            KeyCode::KeyW => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('w');
                }
            }
            KeyCode::KeyX => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('x');
                }
            }
            KeyCode::KeyY => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('y');
                }
            }
            KeyCode::KeyZ => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('z');
                }
            }
            KeyCode::Digit0 => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('0');
                }
            }
            KeyCode::Digit1 => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('1');
                }
            }
            KeyCode::Digit2 => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('2');
                }
            }
            KeyCode::Digit3 => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('3');
                }
            }
            KeyCode::Digit4 => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('4');
                }
            }
            KeyCode::Digit5 => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('5');
                }
            }
            KeyCode::Digit6 => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('6');
                }
            }
            KeyCode::Digit7 => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('7');
                }
            }
            KeyCode::Digit8 => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('8');
                }
            }
            KeyCode::Digit9 => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('9');
                }
            }
            KeyCode::Minus => {
                if state.current_save_name.len() < 30 {
                    state.current_save_name.push('-');
                }
            }
            _ => {}
        }
    }
}

fn update_save_name_display(
    state: Res<SaveLoadPanelState>,
    mut text_query: Query<&mut Text, With<SaveNameText>>,
) {
    if state.is_changed() {
        for mut text in &mut text_query {
            **text = state.current_save_name.clone();
        }
    }
}

fn handle_rename_button(
    mut interaction_query: Query<
        (&Interaction, &RenameButton, &mut BackgroundColor),
        (Changed<Interaction>, With<RenameButton>),
    >,
    mut state: ResMut<SaveLoadPanelState>,
) {
    for (interaction, rename_btn, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.15));

                // Set the current name to the old name (without .json)
                state.current_save_name = rename_btn.old_filename.trim_end_matches(".json").to_string();
                info!("Set save name to {} for renaming", state.current_save_name);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.5, 0.5, 0.25));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.4, 0.4, 0.2));
            }
        }
    }
}

fn handle_delete_button(
    mut interaction_query: Query<
        (&Interaction, &DeleteButton, &mut BackgroundColor),
        (Changed<Interaction>, With<DeleteButton>),
    >,
    mut state: ResMut<SaveLoadPanelState>,
) {
    for (interaction, delete_btn, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.5, 0.15, 0.15));

                // Delete the save file
                let path = format!("assets/saves/{}", delete_btn.filename);
                if let Err(err) = fs::remove_file(&path) {
                    error!("Failed to delete {}: {}", path, err);
                } else {
                    info!("Deleted save: {}", path);
                    state.refresh_saves_list();
                }
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.7, 0.25, 0.25));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.6, 0.2, 0.2));
            }
        }
    }
}
