use super::UiInputBlocker;
use crate::systems::time_control::{SpeedOption, TimeSpeed};
use bevy::prelude::*;

#[derive(Component)]
pub struct SpeedControlPanel;

#[derive(Component)]
pub struct SpeedButton {
    pub speed: SpeedOption,
}

pub struct SpeedControlPlugin;

impl Plugin for SpeedControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiInputBlocker>()
            .add_systems(Startup, setup_speed_control)
            .add_systems(
                Update,
                (
                    handle_speed_button_clicks,
                    update_speed_button_colors,
                    block_map_input_over_speed_controls,
                ),
            );
    }
}

fn setup_speed_control(mut commands: Commands) {
    // Speed control panel in bottom-right corner
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(90.0), // Above the toolbar
                right: Val::Px(10.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(5.0),
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            SpeedControlPanel,
        ))
        .with_children(|parent| {
            spawn_speed_button(parent, SpeedOption::Normal, "1x");
            spawn_speed_button(parent, SpeedOption::Fast, "2x");
            spawn_speed_button(parent, SpeedOption::VeryFast, "3x");
        });
}

fn spawn_speed_button(parent: &mut ChildBuilder, speed: SpeedOption, label: &str) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(50.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            SpeedButton { speed },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(label),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn handle_speed_button_clicks(
    mut interaction_query: Query<(&Interaction, &SpeedButton), Changed<Interaction>>,
    mut time_speed: ResMut<TimeSpeed>,
) {
    for (interaction, speed_button) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            time_speed.set_speed(speed_button.speed);
        }
    }
}

fn update_speed_button_colors(
    mut button_query: Query<(&SpeedButton, &mut BackgroundColor, &Interaction)>,
    time_speed: Res<TimeSpeed>,
) {
    for (speed_button, mut bg_color, interaction) in &mut button_query {
        let is_active = match speed_button.speed {
            SpeedOption::Normal => time_speed.multiplier == 1.0,
            SpeedOption::Fast => time_speed.multiplier == 2.0,
            SpeedOption::VeryFast => time_speed.multiplier == 3.0,
        };

        if is_active {
            *bg_color = BackgroundColor(Color::srgb(0.3, 0.6, 0.3)); // Green when active
        } else {
            match interaction {
                Interaction::Hovered => {
                    *bg_color = BackgroundColor(Color::srgb(0.35, 0.35, 0.35));
                }
                _ => {
                    *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
                }
            }
        }
    }
}

fn block_map_input_over_speed_controls(
    mut ui_blocker: ResMut<UiInputBlocker>,
    interaction_query: Query<&Interaction, With<SpeedButton>>,
) {
    let should_block = interaction_query
        .iter()
        .any(|interaction| matches!(*interaction, Interaction::Hovered | Interaction::Pressed));

    ui_blocker.speed_controls_blocking = should_block;
    ui_blocker.recompute();
}
