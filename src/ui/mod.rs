use bevy::prelude::Resource;

pub mod money_display;
pub mod save_load_panel;
pub mod speed_control;
pub mod toolbar;
pub mod work_assignments;

pub use money_display::*;
pub use save_load_panel::*;
pub use speed_control::*;
pub use toolbar::*;
pub use work_assignments::*;

#[derive(Resource, Default)]
pub struct UiInputBlocker {
    pub block_world_input: bool,
    pub speed_controls_blocking: bool,
    pub context_menu_blocking: bool,
}

impl UiInputBlocker {
    pub fn recompute(&mut self) {
        self.block_world_input = self.speed_controls_blocking || self.context_menu_blocking;
    }
}
