use glam::{EulerRot, Mat4, Quat, Vec3, Vec4Swizzles};

pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3, // Pitch, Yaw, Roll в радианах
    pub scale: Vec3,
}

impl Transform {
    pub fn to_inv_matrix(&self) -> Mat4 {
        let matrix = Mat4::from_scale_rotation_translation(
            self.scale,
            Quat::from_euler(
                EulerRot::XYZ,
                self.rotation.x,
                self.rotation.y,
                self.rotation.z,
            ),
            self.position,
        );

        matrix.inverse()
    }

    pub fn to_inv_matrix_voxel(&self) -> Mat4 {
        let matrix = Mat4::from_scale_rotation_translation(
            self.scale,
            Quat::from_euler(
                EulerRot::XYZ,
                self.rotation.x,
                self.rotation.y,
                self.rotation.z,
            ),
            self.position,
        );

        let inv = matrix.inverse();

        // Смещаем и масштабируем ([-1, 1] -> [0, 16])
        // Сначала переносим из [-1, 1] в [0, 2], затем множим на 8 (чтобы стало 16)
        let offset = Mat4::from_translation(Vec3::ONE); // Сдвиг на 1.0
        let scale_to_res = Mat4::from_scale(Vec3::splat(8.0)); // 2.0 * 8.0 = 16.0

        scale_to_res * offset * inv
    }
}

use crate::window::component::base::area::Area;
use crate::window::component::block_3d::model::model::SHAPE_BOX;
use crate::window::component::block_3d::model::model::{
    Model, SHAPE_CAPSULE, SHAPE_CILINDER, SHAPE_TORUS,
};
use crate::window::wgpu::block_3d::camera_uniform::CameraUniform;

pub fn calculate_model_screen_rect(
    model: &Model,
    camera: &CameraUniform,
    screen_size: [f32; 2],
) -> Area {
    let view_proj = camera.view_proj;
    let [width, height] = screen_size;

    // Определяем "эффективный" радиус для охвата фигуры
    let tag = model.params[0];
    let size = model.params[1];
    let extra = model.params[2];

    let r = if tag == SHAPE_BOX {
        // Для куба радиус описанной сферы — это корень из суммы квадратов сторон
        (Vec3::new(size, size, size)).length()
    } else if tag == SHAPE_TORUS {
        // Увеличиваем "эффективный" радиус для тора
        // size - радиус кольца, extra - радиус трубки
        (size + extra) * 1.2 // Внешний радиус тора
    } else if tag == SHAPE_CAPSULE || tag == SHAPE_CILINDER {
        // Для капсулы и цилиндра: радиус + половина высоты
        (size.powi(2) + extra.powi(2)).sqrt()
    } else {
        size // Для сферы просто радиус
    };

    // Добавляем 10% запаса на сглаживание (AA)
    let r = r * 1.5;
    let center = model.transform.position; // Предполагаем, что в Transform есть position: Vec3

    // 2. Проецируем 8 точек AABB
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    let mut visible = false;

    for i in 0..8 {
        let offset = Vec3::new(
            if i & 1 == 0 { -r } else { r },
            if i & 2 == 0 { -r } else { r },
            if i & 4 == 0 { -r } else { r },
        );

        let clip_p = view_proj * (center + offset).extend(1.0);

        if clip_p.w > 0.0 {
            let ndc = clip_p.xyz() / clip_p.w;
            let px = (ndc.x + 1.0) * 0.5 * width;
            let py = (1.0 - ndc.y) * 0.5 * height;

            min_x = min_x.min(px);
            max_x = max_x.max(px);
            min_y = min_y.min(py);
            max_y = max_y.max(py);
            visible = true;
        }
    }

    if !visible {
        return Area::default();
    }

    let x1 = min_x;
    let y1 = min_y;
    let x2 = max_x;
    let y2 = max_y;

    Area::new(x1, y1, (x2 - x1) as u16, (y2 - y1) as u16)
}

pub fn calculate_group_screen_rect(
    models: &[Model],
    camera: &CameraUniform,
    screen_size: [f32; 2],
) -> Area {
    let view_proj = camera.view_proj;
    let [width, height] = screen_size;

    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    let mut any_visible = false;

    for model in models {
        // Определяем радиус охвата конкретной модели
        let tag = model.params[0];
        let size = model.params[1];
        let extra = model.params[2];

        let r = match tag {
            SHAPE_BOX => (Vec3::new(size, size, size)).length(),
            SHAPE_TORUS => (size + extra) * 1.2,
            SHAPE_CAPSULE | SHAPE_CILINDER => (size.powi(2) + extra.powi(2)).sqrt(),
            _ => size,
        } * 1.5; // Твой запас на сглаживание

        let center = model.transform.position;

        // Проецируем 8 углов описанного куба модели
        for i in 0..8 {
            let offset = Vec3::new(
                if i & 1 == 0 { -r } else { r },
                if i & 2 == 0 { -r } else { r },
                if i & 4 == 0 { -r } else { r },
            );

            let clip_p = view_proj * (center + offset).extend(1.0);

            // Проверка нахождения перед камерой (Near Plane Culling)
            if clip_p.w > 0.001 {
                let ndc = clip_p.xyz() / clip_p.w;
                let px = (ndc.x + 1.0) * 0.5 * width;
                let py = (1.0 - ndc.y) * 0.5 * height;

                min_x = min_x.min(px);
                max_x = max_x.max(px);
                min_y = min_y.min(py);
                max_y = max_y.max(py);
                any_visible = true;
            }
        }
    }

    if !any_visible {
        return Area::default();
    }

    // Безопасное создание области (Area) с учетом границ экрана
    let x1 = min_x.max(-width).min(width * 2.0);
    let y1 = min_y.max(-height).min(height * 2.0);
    let x2 = max_x.max(-width).min(width * 2.0);
    let y2 = max_y.max(-height).min(height * 2.0);

    let w = (x2 - x1).max(0.0) as u16;
    let h = (y2 - y1).max(0.0) as u16;

    Area::new(x1, y1, w, h)
}
