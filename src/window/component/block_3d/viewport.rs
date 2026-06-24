use glam::{Vec2, Vec3};

use crate::{
    add_drawable_control,
    window::{
        component::{
            base::{
                area::Area, base::Base, component_type::SharedDrawable,
                gpu_render_context::GpuRenderContext, settings::Settings, ui_command::CommandTrait,
            },
            block_3d::{
                model::{
                    cube::Cube,
                    math::{cast_ray, get_mouse_ray},
                    model::Model,
                    sdf_command::SDFCommandExt,
                    sphere::Sphere,
                },
                pencil_3d::Pencil3D,
            },
            interface::{
                component_control::{ComponentControl, ComponentControlExt, PanelControl},
                drawable::{
                    AnimationDrawable, ClickableDrawable, DragableDrawable, Drawable,
                    HoverableDrawable, InternalAccess, LayoutDrawable, PaintfulDrawable3D,
                    ScrollableDrawable,
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
    pub model: Model,
    pub camera: CameraUniform,
    pub orbit_controller: OrbitCamera,
    pub scrollable: bool,
    pub pencil: Pencil3D,
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
            yaw: 3.14159,
            pitch: 0.0,
        };

        Viewport3D {
            panel: Panel::default(),
            model: Model::default(),
            camera,
            orbit_controller: orbit,
            scrollable: false,
            pencil: Pencil3D::default(),
        }
    }

    pub fn add_model(&mut self, model: SDFCommandExt) {
        self.model.push(model);
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
        area: &Area,
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

            // let step = 4;
            // for i in (0..self.models.len()).step_by(step / 2) {
            //     let models = &self.models[i..(i + step).min(self.models.len())];

            //     let group_rect = calculate_group_screen_rect(
            //         models,
            //         &self.camera,
            //         [area.max.get_width() as f32, area.max.get_height() as f32],
            //     );

            //     if group_rect.min.get_width() > 0 {
            //         ctx.push_3d_viewport(&group_rect, models, level);
            //     }
            // }

            // for model in &self.models {
            //     // Считаем область на экране для конкретного поворота/позиции этой модели
            //     let tight_rect = calculate_sdf_command_screen_rect(
            //         model,
            //         &self.camera,
            //         [area.max.get_width() as f32, area.max.get_height() as f32],
            //     );

            //     // Если модель попадает в экран — пушим её
            //     if tight_rect.min.get_width() > 0 && tight_rect.min.get_height() > 0 {
            //         ctx.push_model_instance(model, &tight_rect, level);
            //     }
            // }

            let (bake_cmds, instance_cmds, need_render) = self.model.render();

            if need_render {
                self.redraw();
            }

            ctx.push_bake_commands(
                &rect,
                bake_cmds,
                instance_cmds,
                level,
                self.panel.base.settings.background_color,
            );

            ctx.push_rect_sdf(&rect, background_color, border, level, true, true);
        }
    }

    fn resize(&mut self, area: &Area, ctx: &LayoutContext, auto_size: bool) -> Area {
        let rect = self.panel.resize(area, ctx, auto_size);

        return rect;
    }

    fn redraw(&self) {
        self.panel.redraw();
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

    fn hover(&self, mx: u16, my: u16, area: &Area) -> bool {
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
        Some(self)
    }
    fn as_scrollable_mut(&mut self) -> Option<&mut dyn ScrollableDrawable> {
        Some(self)
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

    fn as_paintful(&self) -> Option<&dyn PaintfulDrawable3D> {
        Some(self)
    }
    fn as_paintful_mut(&mut self) -> Option<&mut dyn PaintfulDrawable3D> {
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

        // let rect = &self.panel.base.rect;
        // let aspect = rect.min.get_width() as f32 / rect.min.get_height() as f32;

        // self.camera = self.orbit_controller.update_uniform(aspect);

        // Не забываем пометить данные для загрузки на GPU
        //self.is_dirty = true;
    }
    fn change_distance_camera(&mut self, x_offset: f32, y_offset: f32) {
        let sensitivity = -0.05;

        self.orbit_controller
            .change_distance(x_offset, y_offset * sensitivity);
    }
    fn add_sphere(&mut self, mx: f32, my: f32) {
        let panel_rect = self.panel.base.rect.clone();
        let parent_rect = &self.panel.base.parent_rect;

        let global_x = parent_rect.x1 + panel_rect.x1;
        let global_y = parent_rect.y1 + panel_rect.y1;

        let x = mx - global_x;
        let y = my - global_y;
        let size = Vec2::new(
            panel_rect.min.get_width() as f32,
            panel_rect.min.get_height() as f32,
        );

        let (ray_origin, ray_dir) = get_mouse_ray(
            Vec2::new(x, y),
            size,
            self.camera.inv_view_proj,
            Vec3::from_array(self.camera.camera_pos),
        );

        if let Some((hit_pos, normal)) = cast_ray(ray_origin, ray_dir, &self.model.history_sdf_cmds)
        {
            let sphere_radius = 0.5;
            //let spawn_pos = hit_pos + (normal * sphere_radius);

            let spawn_pos = hit_pos;

            let mut new_command = Sphere::new(sphere_radius, spawn_pos.to_array());

            new_command.color = self.pencil.color;
            new_command.type_union = self.pencil.type_union;

            self.add_model(new_command);
        }
    }
}

impl ScrollableDrawable for Viewport3D {
    fn is_scrollable(&self) -> bool {
        self.scrollable
    }
    fn set_on_scroll(
        &mut self,
        cmd: crate::window::component::base::ui_command::UiCommand,
    ) -> &mut dyn ScrollableDrawable {
        self.panel.set_on_scroll(cmd)
    }
    fn set_scrolable(&mut self, tumbler: bool) -> &mut dyn ScrollableDrawable {
        self.scrollable = tumbler;
        self
    }
    fn set_offset(&mut self, x: f32, y: f32, area: &Area) {}

    fn scroll(&mut self, x: f32, y: f32) -> bool {
        if let Some(cmd) = &self.panel.on_scrol {
            let command_to_send = cmd.clone();

            command_to_send.fill_ref(&self.panel.base.id);

            command_to_send.fill_coord(x, y);

            if let Some(tx) = &self.panel.base.settings.command_tx {
                let _ = tx.send(command_to_send);
            }
        }
        true
    }
}

impl PaintfulDrawable3D for Viewport3D {
    fn change_union_pencil(&mut self, type_union: f32) -> &mut dyn PaintfulDrawable3D {
        self.pencil.type_union = type_union;
        self
    }
    fn change_color_pencil(&mut self, color: u32) -> &mut dyn PaintfulDrawable3D {
        self.pencil.color = color;
        self
    }
}
