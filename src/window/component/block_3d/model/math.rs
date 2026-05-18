use glam::{Mat4, Vec2, Vec3};

use crate::window::component::block_3d::model::sdf_command::{
    SDFCommandExt, sd_box, sd_capsule, sd_cylinder, sd_sphere, smin,
};

fn map_world_sdf(world_p: Vec3, history: &[SDFCommandExt]) -> f32 {
    let mut res = 100.0; // Начальная большая дистанция
    let k = 0.3; // k из твоего шейдера

    for (i, cmd) in history.iter().enumerate() {
        // Получаем инвертированную матрицу (как в твоем BrickManager)
        let model_matrix = cmd.transform.to_matrix();
        let inv_matrix = model_matrix.inverse();

        // Переводим точку в локальное пространство объекта
        let local_p = inv_matrix.transform_point3(world_p);

        let tag = cmd.params[0];
        let size = cmd.params[1];
        let extra = cmd.params[2];
        let uniform_scale = (model_matrix.col(0).truncate().length()
            + model_matrix.col(1).truncate().length()
            + model_matrix.col(2).truncate().length())
            / 3.0;

        let mut d = match tag as u32 {
            1 => sd_sphere(local_p, size),
            2 => sd_box(local_p, Vec3::splat(size)),
            4 => sd_cylinder(local_p, Vec2::new(size, extra)),
            5 => sd_capsule(local_p, extra, size),
            _ => sd_sphere(local_p, size),
        };

        d *= uniform_scale;

        if i == 0 {
            res = d;
        } else {
            res = smin(res, d, k);
        }
    }
    res
}

fn get_normal(p: Vec3, history: &[SDFCommandExt]) -> Vec3 {
    let e = 0.001;
    let n = Vec3::new(
        map_world_sdf(p + Vec3::X * e, history) - map_world_sdf(p - Vec3::X * e, history),
        map_world_sdf(p + Vec3::Y * e, history) - map_world_sdf(p - Vec3::Y * e, history),
        map_world_sdf(p + Vec3::Z * e, history) - map_world_sdf(p - Vec3::Z * e, history),
    );
    n.normalize()
}

/// Возвращает (t_near, t_far), если луч пересекает бокс
pub fn intersect_aabb(origin: Vec3, dir: Vec3, min_p: Vec3, max_p: Vec3) -> Option<(f32, f32)> {
    // Вычисляем инверсию направления, чтобы избежать деления в цикле
    // И обрабатываем деление на ноль (когда луч параллелен оси)
    let inv_dir = Vec3::new(1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z);

    let t1 = (min_p - origin) * inv_dir;
    let t2 = (max_p - origin) * inv_dir;

    let tmin = t1.min(t2);
    let tmax = t1.max(t2);

    let t_near = tmin.x.max(tmin.y).max(tmin.z);
    let t_far = tmax.x.min(tmax.y).min(tmax.z);

    // Если t_far < 0, бокс позади нас.
    // Если t_near > t_far, луч пролетел мимо.
    if t_far >= t_near && t_far > 0.0 {
        Some((t_near, t_far))
    } else {
        None
    }
}

pub fn cast_ray(
    ray_origin: Vec3,
    ray_dir: Vec3,
    history: &[SDFCommandExt],
) -> Option<(Vec3, Vec3)> {
    let mut t = 0.0;

    // Проверка входа в основной бокс сцены [-16, 16]
    // (функцию intersect_aabb можно взять стандартную для луча и куба)
    if let Some((t_near, t_far)) =
        intersect_aabb(ray_origin, ray_dir, Vec3::splat(-16.0), Vec3::splat(16.0))
    {
        t = t_near.max(0.0);

        for _ in 0..80 {
            // 80 шагов достаточно для CPU
            let p = ray_origin + ray_dir * t;
            let d = map_world_sdf(p, history);

            // Динамический порог точно как в fs_main
            let hit_threshold = 0.015 + (t * 0.0005);

            if d < hit_threshold {
                let normal = get_normal(p, history);
                return Some((p, normal));
            }

            t += d.max(0.01); // Безопасный шаг
            if t > t_far {
                break;
            }
        }
    }
    None
}

pub fn get_mouse_ray(
    mouse_pos: Vec2, // (0..width, 0..height)
    screen_size: Vec2,
    inv_view_proj: Mat4, // camera.inv_view_proj
    camera_pos: Vec3,
) -> (Vec3, Vec3) {
    let nx = ((mouse_pos.x + 0.5) / screen_size.x) * 2.0 - 1.0;
    let ny = 1.0 - ((mouse_pos.y + 0.5) / screen_size.y) * 2.0;

    let target = inv_view_proj.project_point3(Vec3::new(nx, ny, 1.0));
    let dir = (target - camera_pos).normalize();

    (camera_pos, dir)
}
