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
    pad0: u32,
    pad1: u32,
    pad2: u32,
}

@group(0) @binding(0) var<storage, read> bake_info: array<BakePushConstants>;
@group(0) @binding(1) var<storage, read> instances: array<Instance3DData>;

fn unpack_color(c: u32) -> vec3<f32> {
    return vec3<f32>(f32(c & 0xffu), f32((c >> 8u) & 0xffu), f32((c >> 16u) & 0xffu)) / 255.0;
}

fn sd_sphere(p: vec3<f32>, s: f32) -> f32 {
    return length(p) - s;
}

fn sd_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

fn sd_torus(p: vec3<f32>, t: vec2<f32>) -> f32 {
    let q = vec2<f32>(length(p.xz) - t.x, p.y);
    return length(q) - t.y;
}

// Цилиндр. h.x - радиус, h.y - половина высоты
fn sd_cylinder(p: vec3<f32>, h: vec2<f32>) -> f32 {
    let d = abs(vec2<f32>(length(p.xz), p.y)) - h;
    return min(max(d.x, d.y), 0.0) + length(max(d, vec2<f32>(0.0)));
}

// Капсула. h - половина расстояния между центрами сфер, r - радиус
fn sd_capsule(p: vec3<f32>, h: f32, r: f32) -> f32 {
    let pa = p - vec3<f32>(0.0, -h, 0.0);
    let ba = vec3<f32>(0.0, h * 2.0, 0.0);
    let h_val = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h_val) - r;
}

fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = max(k - abs(a - b), 0.0) / k;
    return min(a, b) - h * h * k * (1.0 / 4.0);
}

fn unpack_color_unorm(c: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(c & 0xffu) / 255.0,
        f32((c >> 8u) & 0xffu) / 255.0,
        f32((c >> 16u) & 0xffu) / 255.0,
        f32((c >> 24u) & 0xffu) / 255.0
    );
}

// @compute @workgroup_size(8, 8, 1)
// fn cs_main(
//     @builtin(local_invocation_id) local_id: vec3<u32>,
//     @builtin(workgroup_id) group_id: vec3<u32>
// ) {
//     let task = bake_info[group_id.x];
//     let b_id = task.brick_id; // ID от Rust (x + y*32 + z*1024)

//     // Распаковка b_id (строго по формуле из Rust)
//     let b_z = b_id / 1024u;
//     let b_y = (b_id % 1024u) / 32u;
//     let b_x = b_id % 32u;
//     let world_origin = vec3<f32>(f32(b_x), f32(b_y), f32(b_z)) - 16.0;

//     let col = b_id % 64u;
//     let row = b_id / 64u;

//     let k = 0.3;

//     for (var lz = 0u; lz < 8u; lz++) {

//         let local_v = vec3<f32>(f32(local_id.x) + 0.5, f32(local_id.y) + 0.5, f32(lz) + 0.5);

//         // ВАЖНО: local_v / 8.0 переводит 0..8 в 0..1 метр
//         let world_p = world_origin + (local_v / 8.0);

//         var res = 10.0;
//         var final_color = vec4(0.0);

//         for (var i = 0u; i < task.count; i++) {
//             let inst = instances[task.start_instance + i];
//             let local_p = (inst.inv_transform * vec4<f32>(world_p, 1.0)).xyz;

//             let tag = inst.params.x;
//             let size = inst.params.y;

//             var d: f32;

//             if tag < 1.5 { // 1.0 Сфера

//                 let sphere_pos = inst.inv_transform[3].xyz;

//                 d = sd_sphere(world_p - sphere_pos, size);
//             } else if tag < 2.5 { // 2.0 Куб
//                 d = sd_box(local_p, vec3<f32>(size));
//             } else if tag < 3.5 { // 3.0 Тор
//                 // params.z используется как толщина кольца
//                 d = sd_torus(local_p, vec2<f32>(size, inst.params.z));
//             } else if tag < 4.5 { // 4.0 Цилиндр
//                 // params.z используется как высота
//                 d = sd_cylinder(local_p, vec2<f32>(size, inst.params.z));
//             } else { // 5.0 Капсула
//                 d = sd_capsule(local_p, inst.params.z, size);
//             }

//             let base_color = unpack_color_unorm(inst.color);

//             // Считаем SDF по чистой евклидовой дистанции
//             // d = length(world_p - sphere_pos) - size;

//             if i == 0u {
//                 res = d;
//                 final_color = base_color;
//             } else {
//                 let h = clamp(0.5 + 0.5 * (res - d) / k, 0.0, 1.0);
//                 res = smin(res, d, k);

//                 final_color = mix(final_color, base_color, h);
//             }
//         }

//         let tx = clamp(i32((b_id % 64u) * 64u + lz * 8u + local_id.x), 0, 4095);
//         let ty = clamp(i32((b_id / 64u) * 8u + local_id.y), 0, 4095);

//         textureStore(atlas_texture, vec2<i32>(tx, ty), vec4<f32>(res, final_color.rgb));
//     }
// }

@group(0) @binding(2) var atlas_texture: texture_storage_3d<rgba16float, write>;

@compute @workgroup_size(8, 8, 1)
fn cs_main(
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>
) {
    let task = bake_info[group_id.x];
    let b_id = task.brick_id; // Это ID кирпича (от 0 до 32767)

    // Распаковываем 1D индекс в 3D координаты КИРПИЧА
    let b_x = b_id % 32u;
    let b_y = (b_id / 32u) % 32u;
    let b_z = b_id / 1024u;

    let world_origin = vec3<f32>(f32(b_x), f32(b_y), f32(b_z)) - 16.0;

    let k = 0.3;

    for (var lz = 0u; lz < 8u; lz++) {
        let local_v = vec3<f32>(f32(local_id.x) + 0.5, f32(local_id.y) + 0.5, f32(lz) + 0.5);
        let world_p = world_origin + (local_v / 8.0);

        var res = 10.0;
        var final_color = vec4<f32>(0.0);

        for (var i = 0u; i < task.count; i++) {
            let inst = instances[task.start_instance + i];
            let local_p = (inst.inv_transform * vec4<f32>(world_p, 1.0)).xyz;

            let tag = inst.params.x;
            let size = inst.params.y;
            let uniform_scale = inst.params.w; // Извлеченный масштаб из params.w

            var d: f32;

            if tag < 1.5 { // Сфера
                // Возвращаем SDF в честные мировые координаты
                d = sd_sphere(local_p, size) * uniform_scale;
            } else if tag < 2.5 { // Куб
                d = sd_box(local_p, vec3<f32>(size)) * uniform_scale;
            } else if tag < 3.5 { // Тор
                d = sd_torus(local_p, vec2<f32>(size, inst.params.z)) * uniform_scale;
            } else if tag < 4.5 { // Цилиндр
                d = sd_cylinder(local_p, vec2<f32>(size, inst.params.z)) * uniform_scale;
            } else { // Капсула
                d = sd_capsule(local_p, inst.params.z, size) * uniform_scale;
            }

            let base_color = unpack_color_unorm(inst.color);

            if i == 0u {
                res = d;
                final_color = base_color;
            } else {
                let h = clamp(0.5 + 0.5 * (res - d) / k, 0.0, 1.0);
                res = smin(res, d, k);

                final_color = mix(final_color, base_color, h);
            }
        }
        let normalized_sdf = clamp(res / 32.0 + 0.5, 0.0, 1.0);

        let voxel_data = vec4<f32>(normalized_sdf, final_color.rgb);

        let voxel_coords = vec3<i32>(
            i32(b_x * 8u + local_id.x),
            i32(b_y * 8u + local_id.y),
            i32(b_z * 8u + lz)
        );

        textureStore(atlas_texture, voxel_coords, voxel_data);
    }
}
