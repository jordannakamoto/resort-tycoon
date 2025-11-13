use bevy::prelude::*;

#[derive(Component)]
pub struct Blueprint {
    pub building_type: BlueprintType,
    pub work_required: f32,
    pub work_done: f32,
}

impl Blueprint {
    pub fn new(building_type: BlueprintType) -> Self {
        let work_required = match building_type {
            BlueprintType::Wall => 100.0,
            BlueprintType::Door => 150.0,
            BlueprintType::Window => 120.0,
        };

        Self {
            building_type,
            work_required,
            work_done: 0.0,
        }
    }

    pub fn progress(&self) -> f32 {
        self.work_done / self.work_required
    }

    pub fn is_complete(&self) -> bool {
        self.work_done >= self.work_required
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlueprintType {
    Wall,
    Door,
    Window,
}

#[derive(Component)]
pub struct ConstructionJob {
    pub blueprint: Entity,
    pub assigned_pawn: Option<Entity>,
    pub priority: i32,
}

impl ConstructionJob {
    pub fn new(blueprint: Entity) -> Self {
        Self {
            blueprint,
            assigned_pawn: None,
            priority: 5,
        }
    }
}

#[derive(Component)]
pub struct WorkInProgress {
    pub work_speed: f32, // work units per second
}

impl Default for WorkInProgress {
    fn default() -> Self {
        Self { work_speed: 10.0 }
    }
}
