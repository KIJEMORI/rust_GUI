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
}

use crate::window::component::block_3d::model::model::{
    Model, SHAPE_CAPSULE, SHAPE_CILINDER, SHAPE_TORUS,
};
use crate::window::component::{base::area::Rect, block_3d::model::model::SHAPE_BOX};
use crate::window::wgpu::block_3d::camera_uniform::CameraUniform;

pub fn calculate_model_screen_rect(
    model: &Model,
    camera: &CameraUniform,
    screen_size: [f32; 2],
) -> Rect<f32, u16> {
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
        size + extra // Внешний радиус тора
    } else if tag == SHAPE_CAPSULE || tag == SHAPE_CILINDER {
        // Для капсулы и цилиндра: радиус + половина высоты
        (size.powi(2) + extra.powi(2)).sqrt()
    } else {
        size // Для сферы просто радиус
    };

    // Добавляем 10% запаса на сглаживание (AA)
    let r = r * 1.1;
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
        return Rect::default();
    }

    let x1 = min_x.clamp(0.0, width);
    let y1 = min_y.clamp(0.0, height);
    let x2 = max_x.clamp(0.0, width);
    let y2 = max_y.clamp(0.0, height);

    Rect::new(x1, y1, (x2 - x1) as u16, (y2 - y1) as u16)
}
