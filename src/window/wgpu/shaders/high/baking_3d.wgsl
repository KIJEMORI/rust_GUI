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
    type_union: f32,
    pad1: u32,
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

@group(0) @binding(2) var atlas_sdf: texture_storage_2d<r32float, write>;
@group(0) @binding(3) var atlas_color: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(8, 8, 1)
fn cs_main(
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>
) {
    let task = bake_info[group_id.x];
    let b_id = task.brick_id;

    let b_x = b_id % 32u;
    let b_y = (b_id / 32u) % 32u;
    let b_z = b_id / 1024u;

    let world_origin = vec3<f32>(f32(b_x), f32(b_y), f32(b_z)) - 16.0;

    let k = 0.3;

    let tile_x = (b_id % 64u) * 64u;
    let tile_y = (b_id / 64u) * 8u;

    for (var lz = 0u; lz < 8u; lz++) {
        let local_v = vec3<f32>(f32(local_id.x) + 0.5, f32(local_id.y) + 0.5, f32(lz) + 0.5);
        let world_p = world_origin + (local_v / 8.0);

        var res = 10.0; // Общая честная дистанция для всего
        var final_color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        var has_color = false;
        var current_mat_id = u32(0);

        for (var i = 0u; i < task.count; i++) {
            let inst = instances[task.start_instance + i];
            let local_p = (inst.inv_transform * vec4<f32>(world_p, 1.0)).xyz;
            let tag = inst.params.x;
            let size = inst.params.y;
            let uniform_scale = inst.params.w;

            var d: f32;
            if tag < 1.5 { d = sd_sphere(local_p, size) * uniform_scale; }
            else if tag < 2.5 { d = sd_box(local_p, vec3<f32>(size)) * uniform_scale; }
            else if tag < 3.5 { d = sd_torus(local_p, vec2<f32>(size, inst.params.z)) * uniform_scale; }
            else if tag < 4.5 { d = sd_cylinder(local_p, vec2<f32>(size, inst.params.z)) * uniform_scale; }
            else if tag < 5.5 { d = sd_capsule(local_p, inst.params.z, size) * uniform_scale; }
            else { d = sd_sphere(local_p, size) * uniform_scale; }

            let base_color = unpack_color_unorm(inst.color);
            let mat_id = inst.material_id;

            if !has_color {
                res = d;
                current_mat_id = u32(mat_id);
                final_color = base_color;
                has_color = true;
            } else {
                let h = clamp(0.5 + 0.5 * (res - d) / k, 0.0, 1.0);

                let type_union = inst.type_union;

                let old_res = res;

                if type_union < 0.5 { // UNION
                    res = min(res, d);
                    if d < old_res {
                        current_mat_id = mat_id;
                        final_color = mix(final_color.rgba, base_color.rgba, h);
                    }
                }
                else if type_union < 1.5 { // INTERSECTION
                    res = max(res, d);
                    final_color = base_color;
                    if d > old_res { current_mat_id = mat_id; }
                }
                else if type_union < 2.5 { // SUBTRACTION
                    res = max(res, -d);
                    if -d > old_res { current_mat_id = mat_id; }
                }
                else if type_union < 3.5 { // SMOOTH
                    res = smin(res, d, k);
                    final_color = mix(final_color.rgba, base_color.rgba, h);
                    if h > 0.5 { current_mat_id = mat_id; }
                }
                else if type_union < 4.5 { // COLOR_DRAWING
                    final_color = mix(final_color.rgba, base_color.rgba, h);
                    if h > 0.5 { current_mat_id = mat_id; }
                }

                //     if mat_id == 1u {
                //         mix_color = mix(final_color.rgba, base_color.rgba, 0.45);
                //     }
            }
        }

        let voxel_x = tile_x + local_id.x + (lz * 8u);
        let voxel_y = tile_y + local_id.y;
        let uv = vec2<i32>(i32(voxel_x), i32(voxel_y));

        // Пишем чистый f32 в R-канал (никакой упаковки pack2x16 не нужно!)
        textureStore(atlas_sdf, uv, vec4<f32>(res, 0.0, 0.0, 0.0));
        textureStore(atlas_color, uv, vec4<f32>(final_color.rgba));
    }
}
