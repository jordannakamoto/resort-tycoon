use bevy::prelude::*;

use crate::components::{
    CurrentJob, GridPosition, Mood, Pawn, PawnAttributes, PawnProfile, PawnProgression,
    WorkAssignments,
};

/// Template describing how a pawn should look and behave.
/// Keeps generation separate from spawning logic so we can extend data later
/// (traits, needs, schedules, relationships, etc.).
#[derive(Clone, Debug)]
pub struct PawnTemplate {
    pub name: String,
    pub move_speed: f32,
    pub color: Color,
    pub attributes: PawnAttributes,
    pub profile: PawnProfile,
}

impl PawnTemplate {
    pub fn into_bundle(self) -> (PawnCoreBundle, Color) {
        let bundle = PawnCoreBundle {
            pawn: Pawn {
                name: self.name,
                move_speed: self.move_speed,
            },
            attributes: self.attributes,
            profile: self.profile,
            progression: PawnProgression::default(),
            grid_position: GridPosition::new(0, 0),
            current_job: CurrentJob::default(),
            work_assignments: WorkAssignments::default(),
            mood: Mood::default(),
        };

        (bundle, self.color)
    }
}

/// Core pawn components used by simulation and UI.
/// Visuals and positioning are intentionally left to call sites (e.g. generation).
#[derive(Bundle, Clone)]
pub struct PawnCoreBundle {
    pub pawn: Pawn,
    pub attributes: PawnAttributes,
    pub profile: PawnProfile,
    pub progression: PawnProgression,
    pub grid_position: GridPosition,
    pub current_job: CurrentJob,
    pub work_assignments: WorkAssignments,
    pub mood: Mood,
}
