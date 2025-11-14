use bevy::prelude::*;

/// Types of work that pawns can be assigned to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkType {
    Construction,
    Reception,
    Cleaning,
    Cooking,
}

impl WorkType {
    pub fn name(&self) -> &str {
        match self {
            WorkType::Construction => "Construction",
            WorkType::Reception => "Reception",
            WorkType::Cleaning => "Cleaning",
            WorkType::Cooking => "Cooking",
        }
    }

    pub fn all() -> Vec<WorkType> {
        vec![
            WorkType::Construction,
            WorkType::Reception,
            WorkType::Cleaning,
            WorkType::Cooking,
        ]
    }
}

/// Work priority levels (1 = highest priority, 4 = lowest, 0 = disabled)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorkPriority(pub u8);

impl WorkPriority {
    pub const DISABLED: WorkPriority = WorkPriority(0);
    pub const HIGHEST: WorkPriority = WorkPriority(1);
    pub const HIGH: WorkPriority = WorkPriority(2);
    pub const NORMAL: WorkPriority = WorkPriority(3);
    pub const LOW: WorkPriority = WorkPriority(4);

    pub fn is_enabled(&self) -> bool {
        self.0 > 0
    }

    pub fn decrease_towards_highest(&mut self) {
        self.0 = match self.0 {
            0 => 1,
            1 => 1,
            _ => self.0.saturating_sub(1),
        };
    }

    pub fn increase_towards_disabled(&mut self) {
        self.0 = match self.0 {
            0 => 0,
            4 => 0,
            _ => self.0 + 1,
        };
    }

    pub fn display(&self) -> String {
        if self.0 == 0 {
            "-".to_string()
        } else {
            self.0.to_string()
        }
    }
}

/// Component storing a pawn's work priorities
#[derive(Component)]
pub struct WorkAssignments {
    priorities: std::collections::HashMap<WorkType, WorkPriority>,
}

impl Default for WorkAssignments {
    fn default() -> Self {
        let mut priorities = std::collections::HashMap::new();
        // Default: Construction enabled at priority 3, others disabled
        priorities.insert(WorkType::Construction, WorkPriority::NORMAL);
        priorities.insert(WorkType::Reception, WorkPriority::DISABLED);
        priorities.insert(WorkType::Cleaning, WorkPriority::DISABLED);
        priorities.insert(WorkType::Cooking, WorkPriority::DISABLED);

        Self { priorities }
    }
}

impl WorkAssignments {
    pub fn get_priority(&self, work_type: WorkType) -> WorkPriority {
        *self
            .priorities
            .get(&work_type)
            .unwrap_or(&WorkPriority::DISABLED)
    }

    pub fn set_priority(&mut self, work_type: WorkType, priority: WorkPriority) {
        self.priorities.insert(work_type, priority);
    }

    pub fn decrease_priority(&mut self, work_type: WorkType) {
        let mut priority = self.get_priority(work_type);
        priority.decrease_towards_highest();
        self.set_priority(work_type, priority);
    }

    pub fn increase_priority(&mut self, work_type: WorkType) {
        let mut priority = self.get_priority(work_type);
        priority.increase_towards_disabled();
        self.set_priority(work_type, priority);
    }

    /// Returns true if the pawn can do this type of work
    pub fn can_do_work(&self, work_type: WorkType) -> bool {
        self.get_priority(work_type).is_enabled()
    }

    /// Get the highest priority work type the pawn can do
    pub fn get_highest_priority_work(&self, available_work_types: &[WorkType]) -> Option<WorkType> {
        available_work_types
            .iter()
            .filter(|&&work_type| self.can_do_work(work_type))
            .min_by_key(|&&work_type| self.get_priority(work_type))
            .copied()
    }
}

/// Component marking a pawn currently staffing a reception desk
#[derive(Component)]
pub struct StaffingReception {
    pub desk_entity: Entity,
}
