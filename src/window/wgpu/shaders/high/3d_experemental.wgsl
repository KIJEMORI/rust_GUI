struct ScreenUniform {
    size: vec2<f32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    padding: f32,
};

struct Instance3DData {
    inv_transform: mat4x4<f32>,
    params: vec4<f32>, // x: tag, y: brick_id/size, z: smin_k, w: unused
    color: u32,
    entity_id: u32,
    padding: vec2<u32>,
};

@group(0) @binding(0) var<uniform> screen: ScreenUniform;
@group(1) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(1) var<storage, read> instances: array<Instance3DData>;

// Группа 2: Воксельный атлас
@group(2) @binding(0) var atlas_texture: texture_2d<f32>;
@group(2) @binding(1) var atlas_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: u32,
    @location(2) p_a: vec2<f32>,      // UV_min в атласе
    @location(3) p_b: vec2<f32>,      // UV_max в атласе
    @location(4) params: vec4<f32>,   // [radius, tag, smooth, instance_idx]
    @location(5) border_color: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) canvas_pos: vec2<f32>,
    @location(1) @interpolate(flat) instance_idx: u32,
    @location(2) atlas_uv_min: vec2<f32>,
    @location(3) atlas_uv_max: vec2<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let x = (model.position.x / screen.size.x) * 2.0 - 1.0;
    let y = 1.0 - (model.position.y / screen.size.y) * 2.0;

    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.canvas_pos = model.position;
    out.instance_idx = u32(model.params.w); // Передаем индекс инстанса из буфера
    out.atlas_uv_min = model.p_a;
    out.atlas_uv_max = model.p_b;
    return out;
}

// --- SDF Функции ---
fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = max(k - abs(a - b), 0.0) / k;
    return min(a, b) - h * h * k * 0.25;
}

fn sd_sphere(p: vec3<f32>, s: f32) -> f32 { return length(p) - s; }
fn sd_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// ВЫБОРКА ИЗ АТЛАСА
fn sample_brick(uv_min: vec2<f32>, uv_max: vec2<f32>, local_p: vec3<f32>) -> f32 {
    let res = 16.0;
    // local_p ожидается в 0..16
    let p = clamp(local_p, vec3(0.0), vec3(res - 0.05));

    let z_layer = floor(p.z);

    // Перевод локальных координат в UV атласа
    // Z-слои лежат вертикально: 16 слоев по 16 пикселей = 256 пикселей в высоту на кирпич
    let norm_x = p.x / res;
    let norm_y = (z_layer * res + p.y) / (res * res);

    let uv = vec2<f32>(
        uv_min.x + norm_x * (uv_max.x - uv_min.x),
        uv_min.y + norm_y * (uv_max.y - uv_min.y)
    );

    return textureSampleLevel(atlas_texture, atlas_sampler, uv, 0.0).r;
}

fn scene_sdf(world_p: vec3<f32>, target_inst: u32, uv_min: vec2<f32>, uv_max: vec2<f32>) -> f32 {
    let inst = instances[target_inst];

    // Перенос луча в локальное пространство объекта (0..16 для вокселей)
    let local_p = (inst.inv_transform * vec4<f32>(world_p, 1.0)).xyz;

    let tag = inst.params.x;
    let size = inst.params.y;
    let k = inst.params.z;

    if tag > 5.5 { // Тип 6.0: Воксельный атлас
        return sample_brick(uv_min, uv_max, local_p);
    }

    // Стандартные примитивы
    if tag < 1.5 { return sd_sphere(local_p, size); }
    return sd_box(local_p, vec3(size));
}

fn get_normal(p: vec3<f32>, inst_idx: u32, uv_min: vec2<f32>, uv_max: vec2<f32>) -> vec3<f32> {
    let e = 0.01;
    let dx = vec3(e, 0.0, 0.0);
    let dy = vec3(0.0, e, 0.0);
    let dz = vec3(0.0, 0.0, e);
    return normalize(vec3<f32>(
        scene_sdf(p + dx, inst_idx, uv_min, uv_max) - scene_sdf(p - dx, inst_idx, uv_min, uv_max),
        scene_sdf(p + dy, inst_idx, uv_min, uv_max) - scene_sdf(p - dy, inst_idx, uv_min, uv_max),
        scene_sdf(p + dz, inst_idx, uv_min, uv_max) - scene_sdf(p - dz, inst_idx, uv_min, uv_max)
    ));
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
};

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let global_ndc = (in.canvas_pos / screen.size) * 2.0 - 1.0;
    let ray_ndc = vec2<f32>(global_ndc.x, -global_ndc.y);

    let target_near = camera.inv_view_proj * vec4<f32>(ray_ndc, 0.0, 1.0);
    let target_far = camera.inv_view_proj * vec4<f32>(ray_ndc, 1.0, 1.0);
    let ray_origin = camera.camera_pos;
    let ray_dir = normalize(target_far.xyz / target_far.w - target_near.xyz / target_near.w);

    var t = 0.0;
    var hit = false;
    var p = ray_origin;

    // RAY MARCHING LOOP
    for (var step = 0; step < 100; step++) {
        p = ray_origin + ray_dir * t;
        let d = scene_sdf(p, in.instance_idx, in.atlas_uv_min, in.atlas_uv_max);

        if d < 0.005 {
            hit = true;
            break;
        }
        t += d * 0.9; // Коэффициент 0.9 для стабильности
        if t > 1000.0 { break; }
    }

    if !hit { discard; }

    let n = get_normal(p, in.instance_idx, in.atlas_uv_min, in.atlas_uv_max);
    let light_dir = normalize(vec3(1.0, 1.0, 1.0));
    let diff = max(dot(n, light_dir), 0.0) * 0.7 + 0.3;

    var out: FragmentOutput;
    let base_color = vec3(0.7, 0.7, 0.8); // Можно брать из instances[in.instance_idx].color
    out.color = vec4<f32>(base_color * diff, 1.0);

    let clip_pos = camera.view_proj * vec4<f32>(p, 1.0);
    out.depth = clip_pos.z / clip_pos.w;

    return out;
}
