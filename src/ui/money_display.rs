use bevy::prelude::*;
use crate::systems::Money;

#[derive(Component)]
pub struct MoneyDisplay;

pub struct MoneyDisplayPlugin;

impl Plugin for MoneyDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_money_display)
            .add_systems(Update, update_money_display);
    }
}

fn setup_money_display(mut commands: Commands) {
    // Root money display container (top-left)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            MoneyDisplay,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("$0"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.2, 0.8, 0.2)), // Green for money
            ));
        });
}

fn update_money_display(
    money: Res<Money>,
    query: Query<Entity, With<MoneyDisplay>>,
    mut text_query: Query<&mut Text>,
    children_query: Query<&Children>,
) {
    if !money.is_changed() {
        return;
    }

    for entity in &query {
        if let Ok(children) = children_query.get(entity) {
            for &child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    **text = format!("${}", money.amount);
                }
            }
        }
    }
}
