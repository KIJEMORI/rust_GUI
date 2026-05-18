struct BakePushConstants {
    brick_id: u32,
    start_instance: u32,
    count: u32,
    padding: u32,
}

struct Instance3DData {
    inv_transform: mat4x4<f32>,
    params: vec4<f32>,
    color: u32,
    material_id: u32,
    pad0: u32,
    pad1: u32,
}

@group(0) @binding(0) var<storage, read> bake_info: array<BakePushConstants>;
@group(0) @binding(1) var<storage, read> instances: array<Instance3DData>;
@group(0) @binding(2) var atlas_sdf: texture_2d<f32>;
@group(0) @binding(3) var atlas_color: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(4) var atlas_color_final: texture_storage_2d<rgba8unorm, write>;

fn sd_sphere(p: vec3<f32>, s: f32) -> f32 { return length(p) - s; }
fn sd_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}
fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = max(k - abs(a - b), 0.0) / k;
    return min(a, b) - h * h * k * 0.25;
}
fn calculate_analytic_sdf(p: vec3<f32>, task: BakePushConstants) -> f32 {
    var res = 10.0;
    let k = 0.3;
    for (var i = 0u; i < task.count; i++) {
        let inst = instances[task.start_instance + i];
        let local_p = (inst.inv_transform * vec4<f32>(p, 1.0)).xyz;
        let tag = inst.params.x;
        let size = inst.params.y;
        let uniform_scale = inst.params.w;

        var d: f32;
        if tag < 1.5 { d = sd_sphere(local_p, size); }
        else if tag < 2.5 { d = sd_box(local_p, vec3<f32>(size)); }
        else { d = sd_sphere(local_p, size); }

        res = smin(res, d * uniform_scale, k);
    }
    return res;
}

@compute @workgroup_size(8, 8, 1)
fn cs_lighting(
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>
) {
    // let task = bake_info[group_id.x];
    // let b_id = task.brick_id;

    // // Распаковка координат кирпича в мире (32x32x32)
    // let b_x = b_id % 32u;
    // let b_y = (b_id / 32u) % 32u;
    // let b_z = b_id / 1024u;
    // let world_origin = vec3<f32>(f32(b_x), f32(b_y), f32(b_z)) - 16.0;

    // // Координаты в атласе (плитка 64x8 для 8 слоев по z)
    // let tile_x = (b_id % 64u) * 64u;
    // let tile_y = (b_id / 64u) * 8u;

    // for (var lz = 0u; lz < 8u; lz++) {
    //     let lx = local_id.x;
    //     let ly = local_id.y;

    //     let uv = vec2<i32>(
    //         i32(tile_x + lx + (lz * 8u)),
    //         i32(tile_y + ly)
    //     );

    //     let local_v = vec3<f32>(f32(lx) + 0.5, f32(ly) + 0.5, f32(lz) + 0.5);
    //     let world_p = world_origin + (local_v / 8.0);

    //     let d = calculate_analytic_sdf(world_p, task);
    //     if d > 0.2 { // Если мы далеко от поверхности, просто копируем (или пишем фон)
    //         let raw_color = textureLoad(atlas_color, uv);
    //         textureStore(atlas_color_final, uv, raw_color);
    //         continue;
    //     }

    //     let e = 0.02;
    //     let n = normalize(vec3<f32>(
    //         calculate_analytic_sdf(world_p + vec3(e, 0, 0), task) - d,
    //         calculate_analytic_sdf(world_p + vec3(0, e, 0), task) - d,
    //         calculate_analytic_sdf(world_p + vec3(0, 0, e), task) - d
    //     ));

    //     let light_dir = normalize(vec3(1.0, 1.2, 0.8));
    //     let diff = max(dot(n, light_dir), 0.0) * 0.7 + 0.3;

    //     // 4. Мягкие тени (Soft Shadows)
    //     var shadow = 1.0;
    //     var t_shad = 0.05; // Начальное смещение, чтобы не задеть себя
    //     let shadow_origin = world_p + n * 0.02; // Смещение точки старта по нормали

    //     for (var s = 0; s < 12; s++) {
    //         let h = calculate_analytic_sdf(shadow_origin + light_dir * t_shad, task);
    //         if h < 0.001 {
    //             shadow = 0.25;
    //             break;
    //         }
    //         shadow = min(shadow, 16.0 * h / t_shad);
    //         t_shad += clamp(h, 0.02, 0.2); // Ускоряем шаг, но не слишком сильно
    //         if t_shad > 2.0 { break; } // Ограничение дистанции тени
    //     }

    //     let raw_color = textureLoad(atlas_color, uv);
    //     let final_rgb = raw_color.rgb * diff * shadow;
    //     textureStore(atlas_color_final, uv, vec4<f32>(final_rgb, raw_color.a));
    // }
}
