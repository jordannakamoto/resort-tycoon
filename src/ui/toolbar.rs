use bevy::prelude::*;
use bevy::ui::*;

const TOOLBAR_HEIGHT: f32 = 80.0;
const TAB_WIDTH: f32 = 100.0;
const BUTTON_SIZE: f32 = 60.0;

#[derive(Component)]
pub struct Toolbar;

#[derive(Component)]
pub struct TabButton {
    pub tab: ConstructionTab,
}

#[derive(Component)]
pub struct BuildButton {
    pub build_type: BuildingType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstructionTab {
    Structure,
    Furniture,
    Decoration,
    Floors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingType {
    Wall,
    Door,
    Window,
}

#[derive(Resource, Default)]
pub struct ToolbarState {
    pub active_tab: Option<ConstructionTab>,
    pub selected_building: Option<BuildingType>,
}

pub struct ToolbarPlugin;

impl Plugin for ToolbarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolbarState>()
            .add_systems(Startup, setup_toolbar)
            .add_systems(Update, (
                handle_tab_clicks,
                handle_build_button_clicks,
                update_button_colors,
            ));
    }
}

fn setup_toolbar(mut commands: Commands) {
    // Root toolbar container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(TOOLBAR_HEIGHT),
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                flex_direction: FlexDirection::Row,
                padding: UiRect::all(Val::Px(5.0)),
                column_gap: Val::Px(5.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            Toolbar,
        ))
        .with_children(|parent| {
            // Tab buttons
            spawn_tab_button(parent, ConstructionTab::Structure, "Structure");
            spawn_tab_button(parent, ConstructionTab::Furniture, "Furniture");
            spawn_tab_button(parent, ConstructionTab::Decoration, "Decoration");
            spawn_tab_button(parent, ConstructionTab::Floors, "Floors");
        });
}

fn spawn_tab_button(parent: &mut ChildBuilder, tab: ConstructionTab, label: &str) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(TAB_WIDTH),
                height: Val::Px(70.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            TabButton { tab },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn spawn_build_button(parent: &mut ChildBuilder, build_type: BuildingType, label: &str) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(BUTTON_SIZE),
                height: Val::Px(BUTTON_SIZE),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
            BuildButton { build_type },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(label),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn handle_tab_clicks(
    mut interaction_query: Query<
        (&Interaction, &TabButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut toolbar_state: ResMut<ToolbarState>,
    mut commands: Commands,
    toolbar_query: Query<Entity, With<Toolbar>>,
    build_button_query: Query<Entity, With<BuildButton>>,
) {
    for (interaction, tab_button, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Toggle tab or select new tab
                if toolbar_state.active_tab == Some(tab_button.tab) {
                    toolbar_state.active_tab = None;
                    toolbar_state.selected_building = None;
                    // Remove build buttons
                    for entity in &build_button_query {
                        commands.entity(entity).despawn_recursive();
                    }
                } else {
                    toolbar_state.active_tab = Some(tab_button.tab);
                    toolbar_state.selected_building = None;

                    // Remove existing build buttons
                    for entity in &build_button_query {
                        commands.entity(entity).despawn_recursive();
                    }

                    // Spawn new build buttons for this tab
                    if let Ok(toolbar_entity) = toolbar_query.get_single() {
                        commands.entity(toolbar_entity).with_children(|parent| {
                            match tab_button.tab {
                                ConstructionTab::Structure => {
                                    spawn_build_button(parent, BuildingType::Wall, "Wall");
                                    spawn_build_button(parent, BuildingType::Door, "Door");
                                    spawn_build_button(parent, BuildingType::Window, "Window");
                                }
                                _ => {
                                    // TODO: Add other categories
                                }
                            }
                        });
                    }
                }
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.35, 0.35, 0.35).into();
            }
            Interaction::None => {
                if toolbar_state.active_tab == Some(tab_button.tab) {
                    *color = Color::srgb(0.4, 0.4, 0.4).into();
                } else {
                    *color = Color::srgb(0.25, 0.25, 0.25).into();
                }
            }
        }
    }
}

fn handle_build_button_clicks(
    mut interaction_query: Query<
        (&Interaction, &BuildButton),
        Changed<Interaction>,
    >,
    mut toolbar_state: ResMut<ToolbarState>,
) {
    for (interaction, build_button) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            if toolbar_state.selected_building == Some(build_button.build_type) {
                toolbar_state.selected_building = None;
            } else {
                toolbar_state.selected_building = Some(build_button.build_type);
            }
        }
    }
}

fn update_button_colors(
    mut build_button_query: Query<(&BuildButton, &mut BackgroundColor, &Interaction)>,
    toolbar_state: Res<ToolbarState>,
) {
    for (build_button, mut color, interaction) in &mut build_button_query {
        if toolbar_state.selected_building == Some(build_button.build_type) {
            *color = Color::srgb(0.5, 0.7, 0.5).into(); // Green when selected
        } else {
            match interaction {
                Interaction::Hovered => {
                    *color = Color::srgb(0.4, 0.4, 0.4).into();
                }
                _ => {
                    *color = Color::srgb(0.3, 0.3, 0.3).into();
                }
            }
        }
    }
}
