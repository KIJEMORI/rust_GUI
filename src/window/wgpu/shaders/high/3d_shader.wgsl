struct CameraUniform {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var<uniform> screen_size: vec2<f32>;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) world_pos: vec3<f32>, // Центр объекта в мире
    @location(3) params: vec4<f32>,    // [размер, тип, сглаживание, доп]
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) screen_pos: vec2<f32>,
    @location(1) world_center: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) obj_params: vec4<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Преобразуем экранные пиксели в NDC (-1..1)
    let x = (model.position.x / screen_size.x) * 2.0 - 1.0;
    let y = 1.0 - (model.position.y / screen_size.y) * 2.0;

    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.screen_pos = model.position;
    out.world_center = model.world_pos;
    out.color = model.color;
    out.obj_params = model.params;
    return out;
}

// SDF функции для интерактора
fn sd_sphere(p: vec3<f32>, s: f32) -> f32 {
    return length(p) - s;
}

fn sd_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// Расчет нормали через градиент
fn get_normal(p: vec3<f32>, center: vec3<f32>, size: f32) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        sd_sphere(p + e.xyy - center, size) - sd_sphere(p - e.xyy - center, size),
        sd_sphere(p + e.yxy - center, size) - sd_sphere(p - e.yxy - center, size),
        sd_sphere(p + e.yyx - center, size) - sd_sphere(p - e.yyx - center, size)
    ));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Генерируем луч из камеры через текущий пиксель
    let ndc = vec2<f32>(
        (in.screen_pos.x / screen_size.x) * 2.0 - 1.0,
        1.0 - (in.screen_pos.y / screen_size.y) * 2.0
    );

    // Восстановление направления луча в мировом пространстве
    let target = camera.inv_view_proj * vec4<f32>(ndc, 0.0, 1.0);
    let ray_dir = normalize(target.xyz / target.w - camera.camera_pos);
    let ray_origin = camera.camera_pos;

    // Цикл Raymarching
    var t = 0.0;
    var hit = false;
    var p = ray_origin;
    let max_dist = 100.0;

    for (var i = 0; i < 64; i++) {
        p = ray_origin + ray_dir * t;

        // Пример: рисуем сферу в центре world_center
        let d = sd_sphere(p - in.world_center, in.obj_params.x);

        if (d < 0.0001) { hit = true; break; }
        t += d;
        if (t > max_dist) { break; }
    }

    if (!hit) { discard; } // Если не попали в объект — пиксель прозрачный

    // Освещение (Простой Lambert + Блик)
    let normal = get_normal(p, in.world_center, in.obj_params.x);
    let light_pos = vec3<f32>(10.0, 10.0, 10.0);
    let light_dir = normalize(light_pos - p);

    let diff = max(dot(normal, light_dir), 0.2); // Рассеянный свет
    let reflect_dir = reflect(-light_dir, normal);
    let spec = pow(max(dot(ray_dir, reflect_dir), 0.0), 32.0); // Блик

    let final_color = (in.color.rgb * diff) + vec3<f32>(spec * 0.5);

    return vec4<f32>(final_color, in.color.a);
}
