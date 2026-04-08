use std::{
    rc::Rc,
    time::{Duration, Instant},
};

use crate::{
    add_drawable_control,
    window::component::{
        animation::animation_action::{AnimationSequence, AnimationStep},
        base::{
            area::Rect, base::Base, component_type::SharedDrawable,
            gpu_render_context::GpuRenderContext, ui_command::UiCommand,
        },
        interface::{
            component_control::{FullEditControl, LabelControl, PanelControl},
            const_layout::ConstLayout,
            drawable::{AnimationDrawable, Drawable},
        },
        label::Label,
        layout::{const_base_layout::Direction, layout_context::LayoutContext},
    },
};

#[allow(dead_code)]
pub struct EditLabel {
    label: Label,
}

impl EditLabel {
    pub fn new(text: &str) -> Self {
        let mut label = Label::new(text.to_string());

        label
            .as_clickable()
            .unwrap()
            .set_on_click(UiCommand::EditLabel(None));

        let mut steps = Vec::new();
        let on_cursor = |el: SharedDrawable| {
            if let Some(el) = el.borrow_mut().as_edit_label_control_mut() {
                el.set_cursor();
            }
        };
        let off_cursor = |el: SharedDrawable| {
            if let Some(el) = el.borrow_mut().as_edit_label_control_mut() {
                el.delete_cursor();
            }
        };

        steps.push(AnimationStep {
            delay: Duration::from_millis(500),
            action: UiCommand::Custom(None, Rc::new(on_cursor)),
        });
        steps.push(AnimationStep {
            delay: Duration::from_millis(500),
            action: UiCommand::Custom(None, Rc::new(off_cursor)),
        });

        let animation = AnimationSequence {
            steps: steps,
            current_step: 0,
            is_loop: true,
            is_running: false,
            last_tick: Instant::now(),
        };

        if let Some(with_animation) = label.as_with_animation() {
            with_animation.set_animation(vec![animation]);
        }

        EditLabel { label: label }
    }
}

impl Default for EditLabel {
    fn default() -> Self {
        EditLabel::new("")
    }
}

impl Drawable for EditLabel {
    fn print(&self, ctx: &mut GpuRenderContext, area: &Rect<i16>, offset: (f32, f32)) {
        self.label.print(ctx, area, offset);
    }
    fn resize(&mut self, area: &Rect<i16>, ctx: &LayoutContext, scroll_item: bool) -> Rect<i16> {
        self.label.resize(area, ctx, scroll_item)
    }

    add_drawable_control!();

    fn set_padding(&mut self, direction: Direction) {
        self.label.set_padding(direction);
    }
    fn set_margin(&mut self, direction: Direction) {
        self.label.set_margin(direction);
    }
    fn set_const_layout(&mut self, const_layout: Option<Box<dyn ConstLayout>>) {
        self.label.set_const_layout(const_layout);
    }
    fn get_margin(&self) -> &Direction {
        self.label.get_margin()
    }
    fn get_padding(&self) -> &Direction {
        self.label.get_padding()
    }
    fn set_default_settings(&mut self, settings: &super::base::settings::Settings) {
        self.label.set_default_settings(settings);
    }
    fn hover(&self, mx: u16, my: u16) -> bool {
        self.label.hover(mx, my)
    }
    fn as_panel_control_mut(&mut self) -> Option<&mut dyn PanelControl> {
        self.label.as_panel_control_mut()
    }
    fn as_label_control_mut(&mut self) -> Option<&mut dyn LabelControl> {
        self.label.as_label_control_mut()
    }
    fn as_base(&self) -> &Base {
        self.label.as_base()
    }
    fn as_base_mut(&mut self) -> &mut Base {
        self.label.as_base_mut()
    }
    fn as_clickable(&mut self) -> Option<&mut dyn super::interface::drawable::ClickableDrawable> {
        self.label.as_clickable()
    }
    fn as_hoverable(&mut self) -> Option<&mut dyn super::interface::drawable::HoverableDrawable> {
        self.label.as_hoverable()
    }
    fn as_selectable(&mut self) -> Option<&mut dyn super::interface::drawable::SelectableDrawable> {
        self.label.as_selectable()
    }
    fn as_with_animation(&mut self) -> Option<&mut dyn AnimationDrawable> {
        self.label.as_with_animation()
    }
    fn as_edit_label_control_mut(&mut self) -> Option<&mut dyn FullEditControl> {
        self.label.as_edit_label_control_mut()
    }
    fn as_scrollable(&mut self) -> Option<&mut dyn super::interface::drawable::ScrollableDrawable> {
        self.label.as_scrollable()
    }
}
