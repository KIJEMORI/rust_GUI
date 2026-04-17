use std::time::{Duration, Instant};

use crate::window::component::base::ui_command::UiCommand;
#[derive(Clone)]
pub struct AnimationStep {
    pub delay: Duration,
    pub action: UiCommand,
}
#[derive(Clone)]
pub struct AnimationSequence {
    pub steps: Vec<AnimationStep>,
    pub current_step: usize,
    pub is_loop: bool,
    pub is_running: bool,
    pub last_tick: Instant,
}
impl AnimationSequence {
    pub fn reset(&mut self) {
        self.current_step = 0;
        self.last_tick = Instant::now();
        self.is_running = true;
    }
}
