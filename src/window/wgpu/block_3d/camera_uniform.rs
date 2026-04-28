use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // Матрица проекции * матрицу вида
    pub view_proj: Mat4,
    pub inv_view_proj: Mat4,
    pub camera_pos: [f32; 3], // Позиция камеры (нужна как точка старта луча)
    pub _padding: f32,
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_proj: glam::Mat4::IDENTITY,
            inv_view_proj: glam::Mat4::IDENTITY,
            camera_pos: [0.0, 0.0, 0.0],
            _padding: 0.0,
        }
    }
}

pub struct OrbitCamera {
    pub target: Vec3,  // Точка, вокруг которой крутимся
    pub distance: f32, // Расстояние до цели
    pub yaw: f32,      // Поворот влево-вправо
    pub pitch: f32,    // Поворот вверх-вниз
}

impl OrbitCamera {
    pub fn rotate(&mut self, dx: f32, dy: f32) {
        self.yaw += dx;
        // Ограничиваем Pitch, чтобы камера не перевернулась через зенит
        self.pitch = (self.pitch + dy).clamp(-1.5, 1.5);
    }

    pub fn change_distance(&mut self, dx: f32, dy: f32) {
        self.distance = (self.distance + dy).clamp(0.0, 300.0);
    }

    pub fn update_uniform(&self, aspect: f32) -> CameraUniform {
        // Вычисляем позицию камеры на основе углов (сферические координаты)
        let eye = Vec3::new(
            self.distance * self.yaw.cos() * self.pitch.cos(),
            self.distance * self.pitch.sin(),
            self.distance * self.yaw.sin() * self.pitch.cos(),
        ) + self.target;

        let view = Mat4::look_at_rh(eye, self.target, Vec3::Y);
        let proj = Mat4::perspective_rh(45.0f32.to_radians(), aspect, 0.1, 1000.0);
        let view_proj = proj * view;

        CameraUniform {
            view_proj,
            inv_view_proj: view_proj.inverse(),
            camera_pos: eye.to_array(),
            _padding: 0.0,
        }
    }
}
