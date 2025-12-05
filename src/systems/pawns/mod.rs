mod generation;
mod movement;
mod progression;
mod profile;
mod selection;

use bevy::prelude::*;

use crate::systems::grid::TILE_SIZE;

pub use progression::PawnLeveledEvent;
pub use profile::{PawnCoreBundle, PawnTemplate};
pub use selection::SelectedPawn;

pub const PAWN_SIZE: f32 = TILE_SIZE * 2.0;

pub struct PawnPlugin;

impl Plugin for PawnPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<generation::PawnGenerator>()
            .init_resource::<crate::ui::UiInputBlocker>()
            .init_resource::<SelectedPawn>()
            .add_event::<PawnLeveledEvent>()
            .add_systems(Startup, generation::spawn_initial_pawns)
            .add_systems(
                Update,
                (
                    movement::move_pawns,
                    movement::update_pawn_positions,
                    progression::tick_pawn_experience,
                    selection::handle_pawn_selection,
                ),
            );
    }
}
