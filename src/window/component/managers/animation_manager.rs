use std::{rc::Rc, sync::mpsc::Sender, time::Instant};

use crate::window::component::{
    animation::animation_action::AnimationSequence,
    base::{component_type::SharedDrawable, ui_command::UiCommand},
};

pub struct AnimationManager {
    pub active: Vec<(SharedDrawable, AnimationSequence)>,
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self { active: Vec::new() }
    }
}

impl AnimationManager {
    pub fn start(&mut self, item: SharedDrawable) {
        if let Some((_, seq)) = self
            .active
            .iter_mut()
            .find(|(target, _)| Rc::ptr_eq(target, &item))
        {
            seq.reset();
            return;
        } else {
            let now = Instant::now();

            let animations_to_add = {
                item.borrow_mut()
                    .as_with_animation()
                    .map(|e| e.get_animations().to_vec())
            };

            if let Some(animations) = animations_to_add {
                for mut anim in animations {
                    anim.last_tick = now;
                    anim.is_running = true;
                    anim.current_step = 0;
                    self.active.push((Rc::clone(&item), anim));
                }
            }
        }
    }
    pub fn update(&mut self, tx: &Sender<UiCommand>) -> bool {
        let now = Instant::now();
        let mut changed = false;
        for (target, seq) in &mut self.active {
            if let Some(target_ref) = target.borrow_mut().as_with_animation() {
                let should_stop_loop = seq.is_loop && !target_ref.need_animate_loop();
                let should_stop_all = !target_ref.need_animate();

                if should_stop_loop || should_stop_all {
                    seq.is_running = false;
                }
            }

            if !seq.is_running {
                continue;
            }

            let step = &seq.steps[seq.current_step];
            if now >= seq.last_tick + step.delay {
                // Выполняем действие шага
                //tx.send(step.action.clone()).ok();
                step.action.execute_command();
                changed = true;
                seq.last_tick += step.delay;
                seq.current_step += 1;

                if seq.current_step >= seq.steps.len() {
                    if seq.is_loop {
                        seq.current_step = 0;
                    } else {
                        seq.is_running = false;
                    }
                }
            }
        }
        self.active.retain(|(_, seq)| seq.is_running);
        changed
    }

    pub fn query_next_timeout(&self) -> Option<Instant> {
        self.active
            .iter()
            .filter(|(_, seq)| seq.is_running)
            .map(|(_, seq)| seq.last_tick + seq.steps[seq.current_step].delay)
            .min()
    }
}
