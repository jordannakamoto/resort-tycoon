use bevy::prelude::*;

use super::work_assignments::WorkAssignmentsPanelState;

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

#[derive(Component)]
pub struct OrderButton {
    pub order_type: OrderType,
}

#[derive(Component)]
pub struct WorkAssignmentsButton;

#[derive(Component)]
pub struct SaveLoadButton;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstructionTab {
    Orders,
    Structure,
    Furniture,
    Bath,
    Staff,
    Decoration,
    Floors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingType {
    Wall,
    Door,
    Window,
    Floor(crate::components::FloorType),
    Furniture(crate::components::FurnitureType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Deconstruct,
}

impl BuildingType {
    pub fn cost(&self) -> i32 {
        match self {
            BuildingType::Wall => 10,
            BuildingType::Door => 50,
            BuildingType::Window => 30,
            BuildingType::Floor(floor_type) => {
                use crate::components::FloorType;
                match floor_type {
                    FloorType::Wood => 5,
                    FloorType::Stone => 8,
                    FloorType::Carpet => 12,
                    FloorType::Tile => 10,
                }
            }
            BuildingType::Furniture(furniture_type) => {
                use crate::components::{BedType, FurnitureType};
                match furniture_type {
                    FurnitureType::Bed(BedType::Single) => 200,
                    FurnitureType::Bed(BedType::Double) => 350,
                    FurnitureType::Desk => 100,
                    FurnitureType::Chair => 50,
                    FurnitureType::Dresser => 150,
                    FurnitureType::Nightstand => 75,
                    FurnitureType::Toilet => 125,
                    FurnitureType::Sink => 80,
                    FurnitureType::Tub => 275,
                    FurnitureType::ReceptionConsole => 300,
                }
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct ToolbarState {
    pub active_tab: Option<ConstructionTab>,
    pub selected_building: Option<BuildingType>,
    pub selected_order: Option<OrderType>,
}

pub struct ToolbarPlugin;

impl Plugin for ToolbarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolbarState>()
            .add_systems(Startup, setup_toolbar)
            .add_systems(
                Update,
                (
                    handle_tab_clicks,
                    handle_build_button_clicks,
                    handle_order_button_clicks,
                    update_button_colors,
                    update_order_button_colors,
                    handle_work_assignments_button_clicks,
                    update_work_assignments_button_colors,
                    handle_save_load_button_clicks,
                    update_save_load_button_colors,
                ),
            );
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
            spawn_tab_button(parent, ConstructionTab::Orders, "Orders");
            spawn_tab_button(parent, ConstructionTab::Structure, "Structure");
            spawn_tab_button(parent, ConstructionTab::Furniture, "Furniture");
            spawn_tab_button(parent, ConstructionTab::Bath, "Bath");
            spawn_tab_button(parent, ConstructionTab::Staff, "Staff");
            spawn_tab_button(parent, ConstructionTab::Decoration, "Decoration");
            spawn_tab_button(parent, ConstructionTab::Floors, "Floors");

            // Panel shortcuts
            spawn_work_assignments_button(parent);
            spawn_save_load_button(parent);
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

fn spawn_order_button(parent: &mut ChildBuilder, order_type: OrderType, label: &str) {
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
            OrderButton { order_type },
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

fn spawn_work_assignments_button(parent: &mut ChildBuilder) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(140.0),
                height: Val::Px(70.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            WorkAssignmentsButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Assignments"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn spawn_save_load_button(parent: &mut ChildBuilder) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(120.0),
                height: Val::Px(70.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            SaveLoadButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Save/Load"),
                TextFont {
                    font_size: 16.0,
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
    order_button_query: Query<Entity, With<OrderButton>>,
) {
    for (interaction, tab_button, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Toggle tab or select new tab
                if toolbar_state.active_tab == Some(tab_button.tab) {
                    toolbar_state.active_tab = None;
                    toolbar_state.selected_building = None;
                    toolbar_state.selected_order = None;
                    // Remove build buttons
                    for entity in &build_button_query {
                        commands.entity(entity).despawn_recursive();
                    }
                } else {
                    toolbar_state.active_tab = Some(tab_button.tab);
                    toolbar_state.selected_building = None;
                    toolbar_state.selected_order = None;

                    // Remove existing build buttons and order buttons
                    for entity in &build_button_query {
                        commands.entity(entity).despawn_recursive();
                    }
                    for entity in &order_button_query {
                        commands.entity(entity).despawn_recursive();
                    }

                    // Spawn new buttons for this tab
                    if let Ok(toolbar_entity) = toolbar_query.get_single() {
                        commands.entity(toolbar_entity).with_children(|parent| {
                            match tab_button.tab {
                                ConstructionTab::Orders => {
                                    spawn_order_button(
                                        parent,
                                        OrderType::Deconstruct,
                                        "Deconstruct",
                                    );
                                }
                                ConstructionTab::Structure => {
                                    spawn_build_button(parent, BuildingType::Wall, "Wall");
                                    spawn_build_button(parent, BuildingType::Door, "Door");
                                    spawn_build_button(parent, BuildingType::Window, "Window");
                                }
                                ConstructionTab::Furniture => {
                                    use crate::components::{BedType, FurnitureType};
                                    spawn_build_button(
                                        parent,
                                        BuildingType::Furniture(FurnitureType::Bed(
                                            BedType::Single,
                                        )),
                                        "Single Bed",
                                    );
                                    spawn_build_button(
                                        parent,
                                        BuildingType::Furniture(FurnitureType::Bed(
                                            BedType::Double,
                                        )),
                                        "Double Bed",
                                    );
                                    spawn_build_button(
                                        parent,
                                        BuildingType::Furniture(FurnitureType::Desk),
                                        "Desk",
                                    );
                                    spawn_build_button(
                                        parent,
                                        BuildingType::Furniture(FurnitureType::Chair),
                                        "Chair",
                                    );
                                    spawn_build_button(
                                        parent,
                                        BuildingType::Furniture(FurnitureType::Dresser),
                                        "Dresser",
                                    );
                                    spawn_build_button(
                                        parent,
                                        BuildingType::Furniture(FurnitureType::Nightstand),
                                        "Nightstand",
                                    );
                                }
                                ConstructionTab::Bath => {
                                    use crate::components::FurnitureType;
                                    spawn_build_button(
                                        parent,
                                        BuildingType::Furniture(FurnitureType::Tub),
                                        "Tub",
                                    );
                                    spawn_build_button(
                                        parent,
                                        BuildingType::Furniture(FurnitureType::Sink),
                                        "Sink",
                                    );
                                    spawn_build_button(
                                        parent,
                                        BuildingType::Furniture(FurnitureType::Toilet),
                                        "Toilet",
                                    );
                                }
                                ConstructionTab::Staff => {
                                    use crate::components::FurnitureType;
                                    spawn_build_button(
                                        parent,
                                        BuildingType::Furniture(FurnitureType::ReceptionConsole),
                                        "Reception",
                                    );
                                }
                                ConstructionTab::Floors => {
                                    use crate::components::FloorType;
                                    spawn_build_button(
                                        parent,
                                        BuildingType::Floor(FloorType::Wood),
                                        "Wood",
                                    );
                                    spawn_build_button(
                                        parent,
                                        BuildingType::Floor(FloorType::Stone),
                                        "Stone",
                                    );
                                    spawn_build_button(
                                        parent,
                                        BuildingType::Floor(FloorType::Carpet),
                                        "Carpet",
                                    );
                                    spawn_build_button(
                                        parent,
                                        BuildingType::Floor(FloorType::Tile),
                                        "Tile",
                                    );
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
    mut interaction_query: Query<(&Interaction, &BuildButton), Changed<Interaction>>,
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

fn handle_order_button_clicks(
    mut interaction_query: Query<(&Interaction, &OrderButton), Changed<Interaction>>,
    mut toolbar_state: ResMut<ToolbarState>,
) {
    for (interaction, order_button) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            if toolbar_state.selected_order == Some(order_button.order_type) {
                toolbar_state.selected_order = None;
            } else {
                toolbar_state.selected_order = Some(order_button.order_type);
                toolbar_state.selected_building = None; // Clear building selection
            }
        }
    }
}

fn update_order_button_colors(
    mut order_button_query: Query<(&OrderButton, &mut BackgroundColor, &Interaction)>,
    toolbar_state: Res<ToolbarState>,
) {
    for (order_button, mut color, interaction) in &mut order_button_query {
        if toolbar_state.selected_order == Some(order_button.order_type) {
            *color = Color::srgb(0.7, 0.4, 0.4).into(); // Red when selected (destructive action)
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

fn handle_work_assignments_button_clicks(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<WorkAssignmentsButton>)>,
    mut panel_state: ResMut<WorkAssignmentsPanelState>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            panel_state.visible = !panel_state.visible;
        }
    }
}

fn update_work_assignments_button_colors(
    mut button_query: Query<(&mut BackgroundColor, &Interaction), With<WorkAssignmentsButton>>,
    panel_state: Res<WorkAssignmentsPanelState>,
) {
    for (mut color, interaction) in &mut button_query {
        if panel_state.visible {
            *color = Color::srgb(0.4, 0.6, 0.4).into();
        } else {
            match interaction {
                Interaction::Hovered => {
                    *color = Color::srgb(0.35, 0.35, 0.35).into();
                }
                _ => {
                    *color = Color::srgb(0.25, 0.25, 0.25).into();
                }
            }
        }
    }
}

fn handle_save_load_button_clicks(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<SaveLoadButton>)>,
    mut panel_state: ResMut<super::save_load_panel::SaveLoadPanelState>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            panel_state.toggle();
            if panel_state.visible {
                panel_state.refresh_saves_list();
            }
        }
    }
}

fn update_save_load_button_colors(
    mut button_query: Query<(&mut BackgroundColor, &Interaction), With<SaveLoadButton>>,
    panel_state: Res<super::save_load_panel::SaveLoadPanelState>,
) {
    for (mut color, interaction) in &mut button_query {
        if panel_state.visible {
            *color = Color::srgb(0.4, 0.6, 0.4).into();
        } else {
            match interaction {
                Interaction::Hovered => {
                    *color = Color::srgb(0.35, 0.35, 0.35).into();
                }
                _ => {
                    *color = Color::srgb(0.25, 0.25, 0.25).into();
                }
            }
        }
    }
}
