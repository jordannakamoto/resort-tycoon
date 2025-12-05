use bevy::prelude::*;
use rand::prelude::*;

use crate::components::*;

use super::{PawnTemplate, PAWN_SIZE};

const FIRST_NAMES: &[&str] = &[
    "Avery", "Noor", "Mira", "Elias", "Kai", "Rowan", "Jules", "Isla", "Sage", "Niko",
];

const LAST_NAMES: &[&str] = &[
    "Mariner", "Sol", "Wayfarer", "Kite", "Azure", "Harbor", "Vale", "Grove", "Star", "Juniper",
];

const SHIFT_CHOICES: &[&str] = &[
    "Sunrise Prep",
    "Sunset Gala",
    "Evening Breeze",
    "Night Owl Concierge",
];

struct BackgroundTemplate {
    title: &'static str,
    description: &'static str,
    signature_skill: HospitalitySkill,
    secondary_skill: HospitalitySkill,
    taglines: &'static [&'static str],
}

const BACKGROUNDS: &[BackgroundTemplate] = &[
    BackgroundTemplate {
        title: "Wellness Maven",
        description: "Built a reputation hosting sunrise yoga retreats along tropical coasts.",
        signature_skill: HospitalitySkill::Wellness,
        secondary_skill: HospitalitySkill::Concierge,
        taglines: &[
            "Breath in, check in",
            "Calm is contagious",
            "Every lobby needs a sanctuary",
        ],
    },
    BackgroundTemplate {
        title: "Event Virtuoso",
        description: "Can turn any empty hall into a gala experience guests rave about.",
        signature_skill: HospitalitySkill::Entertainment,
        secondary_skill: HospitalitySkill::Service,
        taglines: &[
            "No drama, just ambiance",
            "Every guest deserves a spotlight",
            "Planning beats panic",
        ],
    },
    BackgroundTemplate {
        title: "Harbor Quartermaster",
        description: "Managed boutique cruise vessels and knows how to keep supplies humming.",
        signature_skill: HospitalitySkill::Logistics,
        secondary_skill: HospitalitySkill::Service,
        taglines: &[
            "Inventory is hospitality",
            "Quiet docks, happy guests",
            "Smooth docks make smooth stays",
        ],
    },
    BackgroundTemplate {
        title: "Concierge Whisperer",
        description: "Thrives on remembering names and making impossible reservations happen.",
        signature_skill: HospitalitySkill::Concierge,
        secondary_skill: HospitalitySkill::Wellness,
        taglines: &[
            "Delight hides in details",
            "Everything is solvable",
            "First hello, last goodbye",
        ],
    },
];

pub(super) fn spawn_initial_pawns(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut generator: ResMut<PawnGenerator>,
) {
    for i in 0..3 {
        let template = generator.generate_pawn();
        let (bundle, color) = template.clone().into_bundle();
        let x_offset = (i as f32 - 1.0) * PAWN_SIZE * 1.5;

        commands
            .spawn((
                Mesh2d(meshes.add(Circle::new(PAWN_SIZE * 0.4))),
                MeshMaterial2d(materials.add(color)),
                Transform::from_xyz(x_offset, 0.0, 10.0),
                bundle,
            ))
            .insert(Name::new(format!("Pawn {}", template.name)));
    }
}

#[derive(Resource)]
pub struct PawnGenerator {
    rng: StdRng,
}

impl Default for PawnGenerator {
    fn default() -> Self {
        Self {
            rng: StdRng::from_entropy(),
        }
    }
}

impl PawnGenerator {
    fn generate_pawn(&mut self) -> PawnTemplate {
        let first = FIRST_NAMES.choose(&mut self.rng).unwrap_or(&"Kai");
        let last = LAST_NAMES.choose(&mut self.rng).unwrap_or(&"Azure");
        let name = format!("{first} {last}");

        let background = BACKGROUNDS.choose(&mut self.rng).unwrap();
        let attributes =
            self.roll_attributes(background.signature_skill, background.secondary_skill);
        let profile = PawnProfile {
            background: PawnBackground {
                title: background.title.to_string(),
                description: background.description.to_string(),
                signature_skill: background.signature_skill,
            },
            tagline: background
                .taglines
                .choose(&mut self.rng)
                .unwrap_or(&"Delight every guest")
                .to_string(),
            preferred_shift: SHIFT_CHOICES
                .choose(&mut self.rng)
                .unwrap_or(&"Evening Breeze")
                .to_string(),
        };

        let hue = match profile.background.signature_skill {
            HospitalitySkill::Concierge => Color::srgb(0.2, 0.6, 0.8),
            HospitalitySkill::Service => Color::srgb(0.3, 0.8, 0.6),
            HospitalitySkill::Wellness => Color::srgb(0.5, 0.8, 0.4),
            HospitalitySkill::Entertainment => Color::srgb(0.8, 0.4, 0.7),
            HospitalitySkill::Logistics => Color::srgb(0.8, 0.6, 0.2),
        };

        PawnTemplate {
            name,
            move_speed: self.rng.gen_range(85.0..120.0),
            color: hue,
            attributes,
            profile,
        }
    }

    fn roll_attributes(
        &mut self,
        focus: HospitalitySkill,
        secondary: HospitalitySkill,
    ) -> PawnAttributes {
        PawnAttributes {
            concierge: self.roll_skill(HospitalitySkill::Concierge, focus, secondary),
            service: self.roll_skill(HospitalitySkill::Service, focus, secondary),
            wellness: self.roll_skill(HospitalitySkill::Wellness, focus, secondary),
            entertainment: self.roll_skill(HospitalitySkill::Entertainment, focus, secondary),
            logistics: self.roll_skill(HospitalitySkill::Logistics, focus, secondary),
        }
    }

    fn roll_skill(
        &mut self,
        skill: HospitalitySkill,
        focus: HospitalitySkill,
        secondary: HospitalitySkill,
    ) -> SkillRating {
        let base = if skill == focus {
            self.rng.gen_range(10..=14)
        } else if skill == secondary {
            self.rng.gen_range(7..=11)
        } else {
            self.rng.gen_range(3..=9)
        } as u8;

        let passion = if skill == focus {
            Passion::Devoted
        } else if skill == secondary {
            Passion::Curious
        } else if self.rng.gen_bool(0.15) {
            Passion::Curious
        } else {
            Passion::None
        };

        SkillRating::new(base, passion)
    }
}
