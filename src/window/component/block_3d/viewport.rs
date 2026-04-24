use glam::Mat4;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    add_drawable_control,
    window::{
        component::{
            base::{
                area::Rect, base::Base, component_type::SharedDrawable,
                gpu_render_context::GpuRenderContext, settings::Settings,
            },
            block_3d::{model::model::Model, transform::calculate_model_screen_rect},
            interface::{
                component_control::{ComponentControl, ComponentControlExt, PanelControl},
                drawable::{
                    AnimationDrawable, ClickableDrawable, DragableDrawable, Drawable,
                    HoverableDrawable, InternalAccess, LayoutDrawable, ScrollableDrawable,
                },
                drawable_3d::ViewportControl,
                layout::Layout,
            },
            layout::layout_context::LayoutContext,
            managers::{
                atlas_manager::AtlasManager, button_manager::ButtonManager,
                drag_manager::DragManager, hover_manager::HoverManager, id_manager::IDManager,
                scroll_manager::ScrollManager, select_manager::SelectManager,
            },
            panel::Panel,
        },
        wgpu::block_3d::camera_uniform::{CameraUniform, OrbitCamera},
    },
};

pub struct Viewport3D {
    pub panel: Panel,
    pub models: Vec<Model>,
    pub camera: CameraUniform,
    pub orbit_controller: OrbitCamera,
}

impl Viewport3D {
    pub fn new() -> Self {
        // Создаем нормальную камеру вместо дефолтной "пустышки"
        let eye = glam::Vec3::new(0.0, 0.0, 10.0); // Отходим назад
        let target = glam::Vec3::ZERO; // Смотрим в центр
        let up = glam::Vec3::Y;

        let view = glam::Mat4::look_at_rh(eye, target, up);
        // Пока ставим 1.0 как aspect, он обновится в resize
        let proj = glam::Mat4::perspective_rh(45.0f32.to_radians(), 1.0, 0.1, 1000.0);
        let vp = proj * view;

        let camera = CameraUniform {
            view_proj: vp,
            inv_view_proj: vp.inverse(),
            camera_pos: eye.to_array(),
            _padding: 0.0,
        };

        let orbit = OrbitCamera {
            target,
            distance: 10.0,
            yaw: 0.0,
            pitch: 0.0,
        };

        Viewport3D {
            panel: Panel::default(),
            models: Vec::with_capacity(1024),
            camera,
            orbit_controller: orbit,
        }
    }

    pub fn add_model(&mut self, model: Model) {
        self.models.push(model);
    }

    fn update_camera(&mut self, width: f32, height: f32) {
        let aspect = width / height;

        let uniform = self.orbit_controller.update_uniform(aspect);

        self.camera = uniform;
    }
}

impl Drawable for Viewport3D {
    fn print(
        &mut self,
        ctx: &mut GpuRenderContext,
        area: &Rect<f32, u16>,
        level: u32,
        id_parent: u32,
        atlas: &mut AtlasManager,
    ) {
        self.panel.base.id_parent = id_parent;
        if self.panel.base.visible && self.panel.base.visible_on_this_frame {
            self.panel.base.set_parent_rect(area.clone());
            let mut rect = self.panel.base.rect.clone();

            let x1 = rect.x1 + area.x1;
            let y1 = rect.y1 + area.y1;

            rect.set_position(x1, y1);

            self.update_camera(rect.min.get_width() as f32, rect.min.get_height() as f32);

            let background_color = self.panel.base.settings.background_color;
            let border = &self.panel.border;

            ctx.push_rect_sdf(&rect, background_color, border, level, true, false);
            let current_content_level = level + 1;
            let transient = ((background_color >> 24) & 0xff) as f32;
            if transient > 0.0 {
                ctx.push_rect_sdf(
                    &rect,
                    background_color,
                    border,
                    current_content_level,
                    false,
                    false,
                );
            }

            let next_level = level + 1;

            for child in self.panel.childs.iter() {
                child
                    .borrow_mut()
                    .print(ctx, &rect, next_level, self.panel.base.id, atlas);
            }
            ctx.camera_data = self.camera;

            // for model in &self.models {
            //     let tight_rect = calculate_model_screen_rect(
            //         model,
            //         &self.camera,
            //         [area.max.get_width() as f32, area.max.get_height() as f32],
            //     );

            //     ctx.push_3d_viewport(&tight_rect, &[model], level);
            // }
            for model in &self.models {
                // Считаем область на экране для конкретного поворота/позиции этой модели
                let tight_rect = calculate_model_screen_rect(
                    model,
                    &self.camera,
                    [area.max.get_width() as f32, area.max.get_height() as f32],
                );

                // Если модель попадает в экран — пушим её
                if tight_rect.min.get_width() > 0 && tight_rect.min.get_height() > 0 {
                    ctx.push_model_instance(model, &tight_rect, level);
                }
            }

            ctx.push_rect_sdf(&rect, background_color, border, level, true, true);
        }
    }

    fn resize(
        &mut self,
        area: &Rect<f32, u16>,
        ctx: &LayoutContext,
        auto_size: bool,
    ) -> Rect<f32, u16> {
        let rect = self.panel.resize(area, ctx, auto_size);

        return rect;
    }

    fn get_managers<'a>(
        &'a self,
        button_manager: &mut ButtonManager,
        hover_manager: &mut HoverManager,
        select_manager: &mut SelectManager,
        scroll_manager: &mut ScrollManager,
        drag_manager: &mut DragManager,
        id_manager: &mut IDManager,
        token: &InternalAccess,
    ) {
        self.panel.get_managers(
            button_manager,
            hover_manager,
            select_manager,
            scroll_manager,
            drag_manager,
            id_manager,
            token,
        );
    }

    fn hover(&self, mx: u16, my: u16, area: &Rect<f32, u16>) -> bool {
        self.panel.hover(mx, my, area)
    }

    add_drawable_control!();

    fn as_layout_control(&self) -> &dyn LayoutDrawable {
        self.panel.as_layout_control()
    }
    fn as_layout_control_mut(&mut self) -> &mut dyn LayoutDrawable {
        self.panel.as_layout_control_mut()
    }

    fn set_default_settings(&mut self, settings: &Settings) -> &mut dyn Drawable {
        self.panel.set_default_settings(settings);
        self
    }

    fn as_base(&self) -> &Base {
        self.panel.as_base()
    }
    fn as_base_mut(&mut self) -> &mut Base {
        self.panel.as_base_mut()
    }

    fn as_panel_control(&self) -> &dyn PanelControl {
        self.panel.as_panel_control()
    }
    fn as_panel_control_mut(&mut self) -> &mut dyn PanelControl {
        self.panel.as_panel_control_mut()
    }

    fn as_component_control_mut(&mut self) -> Option<&mut dyn ComponentControl> {
        Some(self)
    }

    fn as_clickable(&self) -> Option<&dyn ClickableDrawable> {
        self.panel.as_clickable()
    }
    fn as_clickable_mut(&mut self) -> Option<&mut dyn ClickableDrawable> {
        self.panel.as_clickable_mut()
    }

    fn as_hoverable(&self) -> Option<&dyn HoverableDrawable> {
        self.panel.as_hoverable()
    }
    fn as_hoverable_mut(&mut self) -> Option<&mut dyn HoverableDrawable> {
        self.panel.as_hoverable_mut()
    }

    fn as_with_animation(&self) -> Option<&dyn AnimationDrawable> {
        self.panel.as_with_animation()
    }
    fn as_with_animation_mut(&mut self) -> Option<&mut dyn AnimationDrawable> {
        self.panel.as_with_animation_mut()
    }

    fn as_scrollable(&self) -> Option<&dyn ScrollableDrawable> {
        self.panel.as_scrollable()
    }
    fn as_scrollable_mut(&mut self) -> Option<&mut dyn ScrollableDrawable> {
        self.panel.as_scrollable_mut()
    }

    fn as_dragable(&self) -> Option<&dyn DragableDrawable> {
        self.panel.as_dragable()
    }
    fn as_dragable_mut(&mut self) -> Option<&mut dyn DragableDrawable> {
        self.panel.as_dragable_mut()
    }

    fn as_viewport_control(&self) -> Option<&dyn ViewportControl> {
        Some(self)
    }
    fn as_viewport_control_mut(&mut self) -> Option<&mut dyn ViewportControl> {
        Some(self)
    }
}

impl ComponentControl for Viewport3D {
    fn add_drawable(&mut self, item: SharedDrawable) -> SharedDrawable {
        self.panel.add_drawable(item)
    }

    fn remove_by_index(&mut self, index: u32) -> Result<(), &'static str> {
        self.panel.remove_by_index(index)
    }

    fn remove_item(&mut self, item: SharedDrawable) {
        self.panel.remove_item(item);
    }

    fn set_layout(&mut self, layout: Box<dyn Layout>) {
        self.panel.set_layout(layout);
    }
}

impl ComponentControlExt for Viewport3D {
    fn add<T: Drawable + 'static>(&mut self, item: T) -> SharedDrawable {
        self.panel.add(item)
    }
}

impl ViewportControl for Viewport3D {
    fn rotate_camera(&mut self, mx_offset: f32, my_offset: f32) {
        // Чувствительность мыши
        let sensitivity = 0.005;

        self.orbit_controller
            .rotate(mx_offset * sensitivity, my_offset * sensitivity);

        let rect = &self.panel.base.rect;
        let aspect = rect.min.get_width() as f32 / rect.min.get_height() as f32;

        self.camera = self.orbit_controller.update_uniform(aspect);

        // Не забываем пометить данные для загрузки на GPU
        //self.is_dirty = true;
    }
}
