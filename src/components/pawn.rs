use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct Pawn {
    pub name: String,
    pub move_speed: f32,
}

impl Default for Pawn {
    fn default() -> Self {
        Self {
            name: "Worker".to_string(),
            move_speed: 100.0, // pixels per second
        }
    }
}

#[derive(Component, Clone)]
pub struct MovementTarget {
    pub target: Vec2,
}

#[derive(Component, Default, Clone)]
pub struct CurrentJob {
    pub job_id: Option<Entity>,
}

// A pawn occupies 2x2 tiles
pub const PAWN_GRID_SIZE: i32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Passion {
    None,
    Curious,
    Devoted,
}

impl Passion {
    pub fn display_name(&self) -> &'static str {
        match self {
            Passion::None => "No Passion",
            Passion::Curious => "Curious",
            Passion::Devoted => "Devoted",
        }
    }

    pub fn xp_multiplier(&self) -> f32 {
        match self {
            Passion::None => 1.0,
            Passion::Curious => 1.2,
            Passion::Devoted => 1.5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HospitalitySkill {
    Concierge,
    Service,
    Wellness,
    Entertainment,
    Logistics,
}

impl HospitalitySkill {
    pub const ALL: [HospitalitySkill; 5] = [
        HospitalitySkill::Concierge,
        HospitalitySkill::Service,
        HospitalitySkill::Wellness,
        HospitalitySkill::Entertainment,
        HospitalitySkill::Logistics,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            HospitalitySkill::Concierge => "Concierge",
            HospitalitySkill::Service => "Service",
            HospitalitySkill::Wellness => "Wellness",
            HospitalitySkill::Entertainment => "Entertainment",
            HospitalitySkill::Logistics => "Logistics",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SkillRating {
    pub level: u8,
    pub passion: Passion,
    pub experience: f32,
}

impl SkillRating {
    pub fn new(level: u8, passion: Passion) -> Self {
        Self {
            level,
            passion,
            experience: 0.0,
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct PawnAttributes {
    pub concierge: SkillRating,
    pub service: SkillRating,
    pub wellness: SkillRating,
    pub entertainment: SkillRating,
    pub logistics: SkillRating,
}

impl PawnAttributes {
    pub fn iter(&self) -> impl Iterator<Item = (HospitalitySkill, SkillRating)> + '_ {
        [
            (HospitalitySkill::Concierge, self.concierge),
            (HospitalitySkill::Service, self.service),
            (HospitalitySkill::Wellness, self.wellness),
            (HospitalitySkill::Entertainment, self.entertainment),
            (HospitalitySkill::Logistics, self.logistics),
        ]
        .into_iter()
    }

    pub fn average_level(&self) -> f32 {
        self.iter()
            .map(|(_, rating)| rating.level as f32)
            .sum::<f32>()
            / HospitalitySkill::ALL.len() as f32
    }
}

#[derive(Clone, Debug)]
pub struct PawnBackground {
    pub title: String,
    pub description: String,
    pub signature_skill: HospitalitySkill,
}

#[derive(Component, Clone, Debug)]
pub struct PawnProfile {
    pub background: PawnBackground,
    pub tagline: String,
    pub preferred_shift: String,
}

#[derive(Component, Clone, Debug)]
pub struct PawnProgression {
    pub level: u32,
    pub experience: f32,
    pub next_level_experience: f32,
}

impl Default for PawnProgression {
    fn default() -> Self {
        Self::new(1)
    }
}

impl PawnProgression {
    pub fn new(level: u32) -> Self {
        Self {
            level,
            experience: 0.0,
            next_level_experience: Self::xp_required(level),
        }
    }

    fn xp_required(level: u32) -> f32 {
        75.0 + (level as f32).powf(1.35) * 25.0
    }

    pub fn gain_experience(&mut self, amount: f32) -> bool {
        if amount <= 0.0 {
            return false;
        }

        self.experience += amount;
        let mut leveled = false;

        while self.experience >= self.next_level_experience {
            self.experience -= self.next_level_experience;
            self.level += 1;
            self.next_level_experience = Self::xp_required(self.level);
            leveled = true;
        }

        leveled
    }

    pub fn progress_ratio(&self) -> f32 {
        if self.next_level_experience <= 0.0 {
            return 0.0;
        }

        (self.experience / self.next_level_experience).clamp(0.0, 1.0)
    }
}
