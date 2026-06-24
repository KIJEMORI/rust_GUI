struct ScreenUniform {
    size: vec2<f32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    padding: f32,
};

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) p_a: vec2<f32>,       // viewport_min
    @location(3) p_b: vec2<f32>,       // viewport_max
    @location(4) params: vec4<f32>,    // [id_3d_instance, ]
    @location(5) border_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) ray_ndc: vec2<f32>, // Передаем готовые координаты для луча
    @location(1) background_color: vec4<f32>,
};
// --- Структура выхода фрагментного шейдера ---
struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32, // Позволяет корректно взаимодействовать с Z-буфером
};

struct Voxel {
    sdf: f32,
    color: vec3<f32>,
    material_id: u32,
};

@group(0) @binding(0) var<uniform> screen: ScreenUniform;
@group(1) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(1) var atlas_sdf: texture_2d<f32>;
@group(1) @binding(2) var atlas_color: texture_2d<f32>;

fn unpack_direction(packed_f32: f32) -> vec3<f32> {
    let packed_u32 = bitcast<u32>(packed_f32);
    let uv = unpack2x16unorm(packed_u32);

    let phi = uv.x * (2.0 * 3.14159265) - 3.14159265;
    let theta = uv.y * 3.14159265;

    let sin_theta = sin(theta);
    return vec3<f32>(
        cos(phi) * sin_theta,
        sin(phi) * sin_theta,
        cos(theta)
    );
}

// Чтение только SDF (дистанции)
fn fetch_sdf(grid_v: vec3<u32>) -> vec2<f32> {
    let clamped_v = clamp(grid_v, vec3<u32>(0u), vec3<u32>(255u));
    let brick_coord = clamped_v / 8u;
    let b_id = brick_coord.x + (brick_coord.y * 32u) + (brick_coord.z * 1024u);

    let tile_x = (b_id % 64u) * 64u;
    let tile_y = (b_id / 64u) * 8u;
    let local_v = clamped_v % 8u;

    let uv = vec2<i32>(
        i32(tile_x + local_v.x + (local_v.z * 8u)),
        i32(tile_y + local_v.y)
    );

    return textureLoad(atlas_sdf, clamp(uv, vec2<i32>(0), vec2<i32>(4095)), 0).rg;
}

fn fetch_material(grid_v: vec3<u32>) -> u32 {
    return 0;
}

fn fetch_color(grid_v: vec3<u32>) -> vec4<f32> {
    let clamped_v = clamp(grid_v, vec3<u32>(0u), vec3<u32>(255u));
    let brick_coord = clamped_v / 8u;
    let b_id = brick_coord.x + (brick_coord.y * 32u) + (brick_coord.z * 1024u);

    let tile_x = (b_id % 64u) * 64u;
    let tile_y = (b_id / 64u) * 8u;
    let local_v = clamped_v % 8u;

    let uv = vec2<i32>(
        i32(tile_x + local_v.x + (local_v.z * 8u)),
        i32(tile_y + local_v.y)
    );

    return textureLoad(atlas_color, clamp(uv, vec2<i32>(0), vec2<i32>(4095)), 0);
}

fn sample_sdf_trilinear(p: vec3<f32>) -> f32 {
    let grid_p = p + 16.0;
    //if any(grid_p < vec3<f32>(0.0625)) || any(grid_p >= vec3<f32>(31.9375)) { return 10.0; }

    let scaled_p = grid_p * 8.0;
    let v_p = vec3<u32>(floor(scaled_p));
    let f_p = fract(scaled_p);

    let data = fetch_sdf(v_p);
    let base_sdf = data.r;

    var v000 = fetch_sdf(v_p + vec3<u32>(0u, 0u, 0u)).r;
    var v001 = fetch_sdf(v_p + vec3<u32>(0u, 0u, 1u)).r;
    var v010 = fetch_sdf(v_p + vec3<u32>(0u, 1u, 0u)).r;
    var v011 = fetch_sdf(v_p + vec3<u32>(0u, 1u, 1u)).r;
    var v100 = fetch_sdf(v_p + vec3<u32>(1u, 0u, 0u)).r;
    var v101 = fetch_sdf(v_p + vec3<u32>(1u, 0u, 1u)).r;
    var v110 = fetch_sdf(v_p + vec3<u32>(1u, 1u, 0u)).r;
    var v111 = fetch_sdf(v_p + vec3<u32>(1u, 1u, 1u)).r;

    v000 = select(base_sdf, v000, v000 < 10.0);
    v001 = select(base_sdf, v001, v001 < 10.0);
    v010 = select(base_sdf, v010, v010 < 10.0);
    v011 = select(base_sdf, v011, v011 < 10.0);
    v100 = select(base_sdf, v100, v100 < 10.0);
    v101 = select(base_sdf, v101, v110 < 10.0);
    v110 = select(base_sdf, v110, v110 < 10.0);
    v111 = select(base_sdf, v111, v111 < 10.0);

    let mix_z0 = mix(mix(v000, v100, f_p.x), mix(v010, v110, f_p.x), f_p.y);
    let mix_z1 = mix(mix(v001, v101, f_p.x), mix(v011, v111, f_p.x), f_p.y);

    return mix(mix_z0, mix_z1, f_p.z);
}

fn sample_sdf_direction(p: vec3<f32>) -> f32 {
    let grid_p = p + 16.0;
    let scaled_p = grid_p * 8.0;

    let v_p = vec3<u32>(floor(scaled_p));
    let f_p = fract(scaled_p); // Смещение точки внутри вокселя [0.0, 1.0]

    let data = fetch_sdf(v_p);

    let base_sdf = data.r;
    let packed_g = data.g;

    if base_sdf > 4.0 { return base_sdf; }

    let N = unpack_direction(packed_g);

    let local_pos = f_p - vec3<f32>(0.5);

    let voxel_size = 0.125;
    let local_pos_world = local_pos * voxel_size;

    let sharp_sdf = base_sdf + dot(local_pos_world, N);

    return sharp_sdf;
}

fn sample_color_trilinear(p: vec3<f32>) -> vec4<f32> {
    let grid_p = p + 16.0;
    let scaled_p = grid_p * 8.0;
    let v_p = vec3<u32>(floor(scaled_p));
    let f_p = fract(scaled_p);

    // Считываем центральный базовый цвет, чтобы знать, чем заменять пустоту
    let base_color = fetch_color(v_p);

    // Считываем все 8 соседей
    var c000 = fetch_color(v_p + vec3<u32>(0u, 0u, 0u));
    var c100 = fetch_color(v_p + vec3<u32>(1u, 0u, 0u));
    var c010 = fetch_color(v_p + vec3<u32>(0u, 1u, 0u));
    var c110 = fetch_color(v_p + vec3<u32>(1u, 1u, 0u));
    var c001 = fetch_color(v_p + vec3<u32>(0u, 0u, 1u));
    var c101 = fetch_color(v_p + vec3<u32>(1u, 0u, 1u));
    var c011 = fetch_color(v_p + vec3<u32>(0u, 1u, 1u));
    var c111 = fetch_color(v_p + vec3<u32>(1u, 1u, 1u));

    // Если сосед пустой (черный),
    // подменяем его на базовый цвет, чтобы не размывать геометрию в прозрачность
    c000 = select(base_color, c000, length(c000.rgba) > 0.001);
    c100 = select(base_color, c100, length(c100.rgba) > 0.001);
    c010 = select(base_color, c010, length(c010.rgba) > 0.001);
    c110 = select(base_color, c110, length(c110.rgba) > 0.001);
    c001 = select(base_color, c001, length(c001.rgba) > 0.001);
    c101 = select(base_color, c101, length(c101.rgba) > 0.001);
    c011 = select(base_color, c011, length(c011.rgba) > 0.001);
    c111 = select(base_color, c111, length(c111.rgba) > 0.001);

    // Стандартное трилинейное смешивание, но теперь уже очищенных цветов
    let mix_z0 = mix(mix(c000, c100, f_p.x), mix(c010, c110, f_p.x), f_p.y);
    let mix_z1 = mix(mix(c001, c101, f_p.x), mix(c011, c111, f_p.x), f_p.y);

    return mix(mix_z0, mix_z1, f_p.z);
}

fn sample_color_direction(p: vec3<f32>) -> vec4<f32> {
    let grid_p = p + 16.0;
    let scaled_p = grid_p * 8.0;

    let v_p = vec3<u32>(floor(scaled_p));
    let f_p = fract(scaled_p);

    let data = fetch_sdf(v_p);
    let packed_g = data.g;
    let N = unpack_direction(packed_g);

    let local_pos = f_p - vec3<f32>(0.5);
    let step_dir = vec3<i32>(sign(local_pos));

    var target_v = v_p;

    // Если смещение по оси критично и нормаль смотрит туда, берем соседа
    if abs(local_pos.x) > 0.4 && abs(N.x) > 0.5 { target_v.x = u32(i32(target_v.x) + step_dir.x); }
    if abs(local_pos.y) > 0.4 && abs(N.y) > 0.5 { target_v.y = u32(i32(target_v.y) + step_dir.y); }
    if abs(local_pos.z) > 0.4 && abs(N.z) > 0.5 { target_v.z = u32(i32(target_v.z) + step_dir.z); }

    let final_color = fetch_color(target_v);

    // Если вдруг у соседа пустой цвет, возвращаем базовый центр
    return select(fetch_color(v_p), final_color, length(final_color.rgba) > 0.001);
}

fn sample_color_material_safe(p: vec3<f32>) -> vec4<f32> {
    let grid_p = p + 16.0;
    let scaled_p = grid_p * 8.0;
    let v_p = vec3<u32>(floor(scaled_p));
    let f_p = fract(scaled_p);

    let base_color = fetch_color(v_p);

    // Если центральный воксель пустой, просто возвращаем его
    if length(base_color.rgba) < 0.001 { return base_color; }

    var c000 = fetch_color(v_p + vec3<u32>(0u, 0u, 0u));
    var c100 = fetch_color(v_p + vec3<u32>(1u, 0u, 0u));
    var c010 = fetch_color(v_p + vec3<u32>(0u, 1u, 0u));
    var c110 = fetch_color(v_p + vec3<u32>(1u, 1u, 0u));
    var c001 = fetch_color(v_p + vec3<u32>(0u, 0u, 1u));
    var c101 = fetch_color(v_p + vec3<u32>(1u, 0u, 1u));
    var c011 = fetch_color(v_p + vec3<u32>(0u, 1u, 1u));
    var c111 = fetch_color(v_p + vec3<u32>(1u, 1u, 1u));

    let threshold = 0.15; // Чувствительность разделения материалов

    c000 = select(base_color, c000, distance(c000.rgb, base_color.rgb) < threshold);
    c100 = select(base_color, c100, distance(c100.rgb, base_color.rgb) < threshold);
    c010 = select(base_color, c010, distance(c010.rgb, base_color.rgb) < threshold);
    c110 = select(base_color, c110, distance(c110.rgb, base_color.rgb) < threshold);
    c001 = select(base_color, c001, distance(c001.rgb, base_color.rgb) < threshold);
    c101 = select(base_color, c101, distance(c101.rgb, base_color.rgb) < threshold);
    c011 = select(base_color, c011, distance(c011.rgb, base_color.rgb) < threshold);
    c111 = select(base_color, c111, distance(c111.rgb, base_color.rgb) < threshold);

    // Стандартное смешивание, но теперь строго очищенное от "чужих" материалов
    let mix_z0 = mix(mix(c000, c100, f_p.x), mix(c010, c110, f_p.x), f_p.y);
    let mix_z1 = mix(mix(c001, c101, f_p.x), mix(c011, c111, f_p.x), f_p.y);

    return mix(mix_z0, mix_z1, f_p.z);
}

fn sample_sdf_nearest(p: vec3<f32>) -> f32 {
    let grid_p = p + 16.0;
    let scaled_p = grid_p * 8.0;
    let v_p = vec3<u32>(floor(scaled_p));

    return fetch_sdf(v_p).r;
}

fn sample_color_nearest(p: vec3<f32>) -> vec4<f32> {
    let grid_p = p + 16.0;
    let scaled_p = grid_p * 8.0;

    let v_p = vec3<u32>(floor(scaled_p));

    return fetch_color(v_p);
}

fn get_voxel_normal(p: vec3<f32>) -> vec3<f32> {
    let grid_p = p + 16.0;
    let scaled_p = grid_p * 8.0;
    let f = fract(scaled_p) - 0.5; // Смещение к центру вокселя
    let abs_f = abs(f);

    // Определяем, какая сторона вокселя ближе к точке контакта
    if abs_f.x > abs_f.y && abs_f.x > abs_f.z {
        return vec3<f32>(sign(f.x), 0.0, 0.0);
    } else if abs_f.y > abs_f.z {
        return vec3<f32>(0.0, sign(f.y), 0.0);
    } else {
        return vec3<f32>(0.0, 0.0, sign(f.z));
    }
}

fn get_normal(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.17, 0.0);
    return normalize(vec3<f32>(
        sample_sdf_trilinear(p + e.xyy) - sample_sdf_trilinear(p - e.xyy),
        sample_sdf_trilinear(p + e.yxy) - sample_sdf_trilinear(p - e.yxy),
        sample_sdf_trilinear(p + e.yyx) - sample_sdf_trilinear(p - e.yyx)
    ));
}

fn sample_normal_trilinear(p: vec3<f32>) -> vec3<f32> {
    let grid_p = p + 16.0;
    let scaled_p = grid_p * 8.0;
    let v_p = vec3<u32>(floor(scaled_p));
    let f_p = fract(scaled_p);

    // Добавляем микро-сглаживание для интерполяционных весов (убирает изломы на стыках вокселей)
    let s_p = f_p * f_p * (3.0 - 2.0 * f_p); // Аналог smoothstep для весов

    let n000 = unpack_direction(fetch_sdf(v_p + vec3<u32>(0u, 0u, 0u)).g);
    let n100 = unpack_direction(fetch_sdf(v_p + vec3<u32>(1u, 0u, 0u)).g);
    let n010 = unpack_direction(fetch_sdf(v_p + vec3<u32>(0u, 1u, 0u)).g);
    let n110 = unpack_direction(fetch_sdf(v_p + vec3<u32>(1u, 1u, 0u)).g);
    let n001 = unpack_direction(fetch_sdf(v_p + vec3<u32>(0u, 0u, 1u)).g);
    let n101 = unpack_direction(fetch_sdf(v_p + vec3<u32>(1u, 0u, 1u)).g);
    let n011 = unpack_direction(fetch_sdf(v_p + vec3<u32>(0u, 1u, 1u)).g);
    let n111 = unpack_direction(fetch_sdf(v_p + vec3<u32>(1u, 1u, 1u)).g);

    // Смешиваем, используя сглаженные веса s_p вместо f_p
    let mix_z0 = mix(mix(n000, n100, s_p.x), mix(n010, n110, s_p.x), s_p.y);
    let mix_z1 = mix(mix(n001, n101, s_p.x), mix(n011, n111, s_p.x), s_p.y);

    return normalize(mix(mix_z0, mix_z1, s_p.z));
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let x = (model.position.x / screen.size.x) * 2.0 - 1.0;
    let y = 1.0 - (model.position.y / screen.size.y) * 2.0;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);

    let size = model.p_b - model.p_a;
    let local_uv = (model.position - model.p_a) / size;

    out.ray_ndc = vec2<f32>(
        local_uv.x * 2.0 - 1.0,
        1.0 - local_uv.y * 2.0
    );

    out.background_color = model.color;

    return out;
}
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let background_color = in.background_color;
    let ray_ndc = in.ray_ndc;

    let target_far = camera.inv_view_proj * vec4<f32>(ray_ndc, 1.0, 1.0);
    let world_far = target_far.xyz / target_far.w;

    var ray_dir = normalize(world_far - camera.camera_pos);
    let ray_origin = camera.camera_pos;

    let box_min = vec3<f32>(-16.0);
    let box_max = vec3<f32>(16.0);

    var t_min = 0.0;
    var t_max = 1000.0;

    let t1 = (box_min - ray_origin) / ray_dir;
    let t2 = (box_max - ray_origin) / ray_dir;
    let t_near = min(t1, t2);
    let t_far = max(t1, t2);

    t_min = max(t_min, max(t_near.x, max(t_near.y, t_near.z)));
    t_max = min(t_max, min(t_far.x, min(t_far.y, t_far.z)));

    if t_min > t_max {
        return background_color;
    }

    t_max = min(t_max, t_min + 64.1);

    var t = max(0.0, t_min);
    var sample_color = vec3<f32>(0.0);
    var ray_transmission = 1.0;

    // Параметры глобального освещения
    let sun_dir = normalize(vec3<f32>(0.6, 0.8, 0.4));
    let ambient_light = 0.15;

    // Состояния структуры слоев
    var inside_object = false;
    var t_entry = 0.0;
    var current_obj_alpha = 0.0;
    var current_base_color = vec3<f32>(0.0);

    for (var i = 0; i < 80; i++) {
        let p = ray_origin + ray_dir * t;

        if any(p < vec3<f32>(-16.1)) || any(p > vec3<f32>(16.1)) { break; }

        // Луч стабильно и плавно шагает по непрерывному трилинейному SDF
        let d = sample_sdf_trilinear(p);
        let hit_threshold = 0.015 + (t * 0.0005);

        if d < hit_threshold {
            // Для цвета и альфы применяем резкое направленное переключение на гранях
            let interpolated_color = sample_color_trilinear(p);
            let voxel_alpha = interpolated_color.a;

            if voxel_alpha < 0.05 {
                t += 0.02;
                continue;
            }

            if !inside_object {
                inside_object = true;
                t_entry = t;
                current_obj_alpha = voxel_alpha;
                current_base_color = interpolated_color.rgb;
            }

            if inside_object && abs(voxel_alpha - current_obj_alpha) > 0.1 {
                let thickness = t - t_entry;
                let final_alpha = select(1.0 - exp(-current_obj_alpha * thickness * 0.5), 1.0, current_obj_alpha > 0.95);

                let hit_p = ray_origin + ray_dir * t_entry;
                let normal_vector = sample_normal_trilinear(hit_p);
                // Заменяем жесткий max(dot, 0) на плавный smoothstep переход
                let raw_ndotl = dot(normal_vector, sun_dir);
                let diffuse = smoothstep(-0.05, 0.1, raw_ndotl);

                // Расчет теней (Shadow Raymarching)
                var shadow_factor = 1.0;
                var t_shadow = 0.04 / max(dot(normal_vector, sun_dir), 0.001);
                t_shadow = clamp(t_shadow, 0.05, 0.35);

                for (var j = 0; j < 40; j++) {
                    let shadow_p = hit_p + sun_dir * t_shadow;
                    if any(shadow_p < vec3<f32>(-16.0)) || any(shadow_p > vec3<f32>(16.0)) { break; }

                    // Теневой луч также оценивает пространство по гладкому алгоритму
                    let sd = sample_sdf_trilinear(shadow_p);

                    // Безопасный порог против самозатенения на мягких стыках сетки
                    if sd < 0.025 {
                        shadow_factor = 0.0;
                        break;
                    }
                    t_shadow += max(sd, 0.02);
                    if t_shadow > 50.0 { break; }
                }

                let lighting = ambient_light + diffuse * shadow_factor;
                let lit_color = current_base_color * clamp(lighting, 0.0, 1.0);

                sample_color += lit_color * final_alpha * ray_transmission;
                ray_transmission *= (1.0 - final_alpha);

                if ray_transmission < 0.01 { break; }

                t_entry = t;
                current_obj_alpha = voxel_alpha;
                current_base_color = interpolated_color.rgb;
            }

            let inside_step = select(0.02, min(abs(d) * 0.9, 0.5), d < -0.02);
            t += inside_step;
        } else {
            if inside_object {
                inside_object = false;
                let thickness = t - t_entry;
                let final_alpha = select(1.0 - exp(-current_obj_alpha * thickness * 0.5), 1.0, current_obj_alpha > 0.95);

                let hit_p = ray_origin + ray_dir * t_entry;
                let normal_vector = sample_normal_trilinear(hit_p);
                // Заменяем жесткий max(dot, 0) на плавный smoothstep переход
                let raw_ndotl = dot(normal_vector, sun_dir);
                let diffuse = smoothstep(-0.05, 0.1, raw_ndotl);

                var shadow_factor = 1.0;
                var t_shadow = 0.04 / max(dot(normal_vector, sun_dir), 0.001);
                t_shadow = clamp(t_shadow, 0.05, 0.35);

                for (var j = 0; j < 40; j++) {
                    let shadow_p = hit_p + sun_dir * t_shadow;
                    if any(shadow_p < vec3<f32>(-16.0)) || any(shadow_p > vec3<f32>(16.0)) { break; }

                    let sd = sample_sdf_trilinear(shadow_p);
                    if sd < 0.025 {
                        shadow_factor = 0.0;
                        break;
                    }
                    t_shadow += max(sd, 0.02);
                    if t_shadow > 50.0 { break; }
                }

                let lighting = ambient_light + diffuse * shadow_factor;
                let lit_color = current_base_color * clamp(lighting, 0.0, 1.0);

                sample_color += lit_color * final_alpha * ray_transmission;
                ray_transmission *= (1.0 - final_alpha);
            }

            if ray_transmission < 0.01 { break; }

            let safe_step = min(d, 1.5);
            t += max(safe_step, 0.02);
        }

        if t > t_max { break; }
    }

    if inside_object {
        let thickness = t - t_entry;
        let final_alpha = select(1.0 - exp(-current_obj_alpha * thickness * 0.5), 1.0, current_obj_alpha > 0.95);

        let hit_p = ray_origin + ray_dir * t_entry;
        let normal_vector = sample_normal_trilinear(hit_p);
        // Заменяем жесткий max(dot, 0) на плавный smoothstep переход
        let raw_ndotl = dot(normal_vector, sun_dir);
        let diffuse = smoothstep(-0.05, 0.1, raw_ndotl);

        var shadow_factor = 1.0;
        var t_shadow = 0.04 / max(dot(normal_vector, sun_dir), 0.001);
        t_shadow = clamp(t_shadow, 0.05, 0.35);

        for (var j = 0; j < 40; j++) {
            let shadow_p = hit_p + sun_dir * t_shadow;
            if any(shadow_p < vec3<f32>(-16.0)) || any(shadow_p > vec3<f32>(16.0)) { break; }

            let sd = sample_sdf_trilinear(shadow_p);
            if sd < 0.025 {
                shadow_factor = 0.0;
                break;
            }
            t_shadow += max(sd, 0.02);
            if t_shadow > 50.0 { break; }
        }

        let lighting = ambient_light + diffuse * shadow_factor;
        let lit_color = current_base_color * clamp(lighting, 0.0, 1.0);

        sample_color += lit_color * final_alpha * ray_transmission;
        ray_transmission *= (1.0 - final_alpha);
    }

    sample_color += background_color.rgb * ray_transmission;
    return vec4<f32>(sample_color, 1.0);
}
// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     let background_color = in.background_color;
//     let ray_ndc = in.ray_ndc;

//     let target_far = camera.inv_view_proj * vec4<f32>(ray_ndc, 1.0, 1.0);
//     let world_far = target_far.xyz / target_far.w;

//     var ray_dir = normalize(world_far - camera.camera_pos);
//     let ray_origin = camera.camera_pos;

//     let box_min = vec3<f32>(-16.0);
//     let box_max = vec3<f32>(16.0);

//     var t_min = 0.0;
//     var t_max = 1000.0;

//     let t1 = (box_min - ray_origin) / ray_dir;
//     let t2 = (box_max - ray_origin) / ray_dir;
//     let t_near = min(t1, t2);
//     let t_far = max(t1, t2);

//     t_min = max(t_min, max(t_near.x, max(t_near.y, t_near.z)));
//     t_max = min(t_max, min(t_far.x, min(t_far.y, t_far.z)));

//     if t_min > t_max {
//         return background_color;
//     }

//     t_max = min(t_max, t_min + 64.1);

//     var t = max(0.0, t_min);
//     var sample_color = vec3<f32>(0.0);
//     var ray_transmission = 1.0;

//     // Переменные состояния
//     var inside_object = false;
//     var t_entry = 0.0;
//     var current_obj_alpha = 0.0;
//     var current_obj_unlit_color = vec3<f32>(0.0); // Убрали упоминание "lit"

//     for (var i = 0; i < 80; i++) {
//         let p = ray_origin + ray_dir * t;

//         if any(p < vec3<f32>(-16.1)) || any(p > vec3<f32>(16.1)) { break; }

//         let d = sample_sdf_trilinear(p);
//         let hit_threshold = 0.035 + (t * 0.0005);

//         if d < hit_threshold {
//             let interpolated_color = sample_color_trilinear(p);
//             let voxel_alpha = interpolated_color.a;

//             if voxel_alpha < 0.05 {
//                 t += 0.02; // Просто делаем маленький шаг вперед в пустоте
//                 continue;
//             }

//             if !inside_object {
//                 inside_object = true;
//                 t_entry = t;
//                 current_obj_alpha = voxel_alpha;
//                 current_obj_unlit_color = interpolated_color.rgb;
//             }

//             if inside_object && abs(voxel_alpha - current_obj_alpha) > 0.1 {
//                 let thickness = t - t_entry;
//                 let final_alpha = select(1.0 - exp(-current_obj_alpha * thickness * 0.5), 1.0, current_obj_alpha > 0.95);

//                 sample_color += current_obj_unlit_color * final_alpha * ray_transmission;
//                 ray_transmission *= (1.0 - final_alpha);

//                 if ray_transmission < 0.01 { break; }

//                 t_entry = t;
//                 current_obj_alpha = voxel_alpha;
//                 current_obj_unlit_color = interpolated_color.rgb;
//             }

//             let inside_step = select(0.02, min(abs(d) * 0.9, 0.5), d < -0.02);
//             t += inside_step;
//         } else {
//             if inside_object {
//                 inside_object = false;
//                 let thickness = t - t_entry;
//                 let final_alpha = select(1.0 - exp(-current_obj_alpha * thickness * 0.5), 1.0, current_obj_alpha > 0.95);

//                 sample_color += current_obj_unlit_color * final_alpha * ray_transmission;
//                 ray_transmission *= (1.0 - final_alpha);
//             }

//             if ray_transmission < 0.01 { break; }

//             let safe_overstep = 0.85;
//             let step_scale = 1.0 + (t * 0.01);

//             let safe_step = min(d * safe_overstep * step_scale, 1.5);

//             t += max(safe_step, 0.02);
//         }

//         if t > t_max { break; }
//     }

//     // Если луч завершил работу, так и не выйдя из объекта
//     if inside_object {
//         let thickness = t - t_entry;
//         let final_alpha = select(1.0 - exp(-current_obj_alpha * thickness * 0.5), 1.0, current_obj_alpha > 0.95);
//         sample_color += current_obj_unlit_color * final_alpha * ray_transmission;
//         ray_transmission *= (1.0 - final_alpha);
//     }

//     // Добавляем фон
//     sample_color += background_color.rgb * ray_transmission;

//     return vec4<f32>(sample_color, 1.0);
// }


// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     let background_color = in.background_color;
//     let ray_ndc = in.ray_ndc;

//     let target_far = camera.inv_view_proj * vec4<f32>(ray_ndc, 1.0, 1.0);
//     let world_far = target_far.xyz / target_far.w;

//     var ray_dir = normalize(world_far - camera.camera_pos);
//     let ray_origin = camera.camera_pos;

//     let box_min = vec3<f32>(-16.0);
//     let box_max = vec3<f32>(16.0);

//     var t_min = 0.0;
//     var t_max = 1000.0;

//     let t1 = (box_min - ray_origin) / ray_dir;
//     let t2 = (box_max - ray_origin) / ray_dir;
//     let t_near = min(t1, t2);
//     let t_far = max(t1, t2);

//     t_min = max(t_min, max(t_near.x, max(t_near.y, t_near.z)));
//     t_max = min(t_max, min(t_far.x, min(t_far.y, t_far.z)));

//     if t_min > t_max {
//         return background_color;
//     }

//     t_max = min(t_max, t_min + 64.1);

//     var t = max(0.0, t_min);
//     var sample_color = vec3<f32>(0.0);
//     var ray_transmission = 1.0;

//     let sun_dir = normalize(vec3<f32>(0.6, 0.8, 0.4));

//     // Переменные состояния
//     var inside_object = false;
//     var t_entry = 0.0;
//     var obj_normal = vec3<f32>(0.0);
//     var current_obj_alpha = 0.0;
//     var current_obj_lit_color = vec3<f32>(0.0);

//     for (var i = 0; i < 80; i++) {
//         let p = ray_origin + ray_dir * t;

//         if any(p < vec3<f32>(-16.1)) || any(p > vec3<f32>(16.1)) { break; }

//         let d = sample_sdf_trilinear(p);
//         let hit_threshold = 0.015 + (t * 0.0005);

//         if d < hit_threshold {

//             let scaled_p = (p + 16.0) * 8.0;
//             let v_p = vec3<u32>(floor(scaled_p));
//             let color_data = fetch_color(v_p);
//             //let voxel_alpha = color_data.a;

//             let interpolated_color = sample_color_trilinear(p);

//             let voxel_alpha = interpolated_color.a;

//             if !inside_object {
//                 inside_object = true;
//                 t_entry = t;
//                 obj_normal = get_normal(p);
//                 current_obj_alpha = voxel_alpha;

//                 let diffuse = max(dot(obj_normal, sun_dir), 0.0);
//                 current_obj_lit_color = interpolated_color.rgb * clamp(diffuse + 0.2, 0.0, 1.0);
//             }

//             if inside_object && abs(voxel_alpha - current_obj_alpha) > 0.1 {
//                 let thickness = t - t_entry;
//                 let final_alpha = select(1.0 - exp(-current_obj_alpha * thickness * 0.5), 1.0, current_obj_alpha > 0.95);

//                 sample_color += current_obj_lit_color * final_alpha * ray_transmission;
//                 ray_transmission *= (1.0 - final_alpha);

//                 if ray_transmission < 0.01 { break; }

//                 // Переключаемся на новый объект
//                 t_entry = t;
//                 obj_normal = get_normal(p);
//                 current_obj_alpha = voxel_alpha;

//                 let diffuse = max(dot(obj_normal, sun_dir), 0.0);
//                 current_obj_lit_color = interpolated_color.rgb * clamp(diffuse + 0.2, 0.0, 1.0);
//             }

//             let inside_step = select(0.02, min(abs(d) * 0.9, 0.5), d < -0.02);
//             t += inside_step;
//         } else {

//             if inside_object {
//                 inside_object = false;
//                 let thickness = t - t_entry;
//                 let final_alpha = select(1.0 - exp(-current_obj_alpha * thickness * 0.5), 1.0, current_obj_alpha > 0.95);

//                 sample_color += current_obj_lit_color * final_alpha * ray_transmission;
//                 ray_transmission *= (1.0 - final_alpha);
//             }

//             if ray_transmission < 0.01 { break; }

//             let overstep = 1.3;
//             let step_scale = 1.0 + (t * 0.01);
//             let safe_step = min(d * overstep * step_scale, 2.0);
//             t += max(safe_step, 0.02);
//         }

//         if t > t_max { break; }
//     }

//     // Если луч завершил работу, так и не выйдя из объекта
//     if inside_object {
//         let thickness = t - t_entry;
//         let final_alpha = select(1.0 - exp(-current_obj_alpha * thickness * 0.5), 1.0, current_obj_alpha > 0.95);
//         sample_color += current_obj_lit_color * final_alpha * ray_transmission;
//         ray_transmission *= (1.0 - final_alpha);
//     }

//     // Добавляем фон
//     sample_color += background_color.rgb * ray_transmission;

//     return vec4<f32>(sample_color, 1.0);
// }



// // Главный фрагментный шейдер
// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     let background_color = in.background_color;
//     let ray_ndc = in.ray_ndc;

//     let target_far = camera.inv_view_proj * vec4<f32>(ray_ndc, 1.0, 1.0);
//     let world_far = target_far.xyz / target_far.w;

//     var ray_dir = normalize(world_far - camera.camera_pos);

//     let ray_origin = camera.camera_pos;

//     let box_min = vec3<f32>(-16.0);
//     let box_max = vec3<f32>(16.0);

//     var t_min = 0.0;
//     var t_max = 1000.0;

//     let t1 = (box_min - ray_origin) / ray_dir;
//     let t2 = (box_max - ray_origin) / ray_dir;
//     let t_near = min(t1, t2);
//     let t_far = max(t1, t2);

//     t_min = max(t_min, max(t_near.x, max(t_near.y, t_near.z)));
//     t_max = min(t_max, min(t_far.x, min(t_far.y, t_far.z)));

//     if t_min > t_max {
//         return background_color;
//     }

//     t_max = min(t_max, t_min + 64.1);

//     var t = max(0.0, t_min);
//     var hit = false;
//     var sample_color = vec3<f32>(0.0);

//     var ray_transmission = 1.0;
//     var in_glass = false;

//     let voxel_size = 1.0 / 8.0;
//     let light_dir = normalize(vec3<f32>(0.6, 0.8, 0.4));

//     for (var i = 0; i < 100; i++) {
//         let p = ray_origin + ray_dir * t;

//         if any(p < vec3<f32>(-16.1)) || any(p > vec3<f32>(16.1)) { break; }

//         let d = sample_sdf_trilinear(p);

//         let hit_threshold = 0.015 + (t * 0.0005);

//         if d < hit_threshold {
//             let scaled_p = (p + 16.0) * 8.0;
//             let v_p = vec3<u32>(floor(scaled_p));
//             let color_data = fetch_color(v_p);
//             let material_id = u32(color_data.a * 255.0 + 0.5);

//             if material_id == 1u {
//                 if !in_glass {
//                     var glass_rgb = color_data.rgb;
//                     if length(glass_rgb) < 0.01 {
//                         glass_rgb = vec3<f32>(0.7, 0.5, 0.6);
//                     }

//                     let glass_opacity = 0.25;
//                     sample_color += glass_rgb * glass_opacity * ray_transmission;
//                     ray_transmission *= (1.0 - glass_opacity);

//                     in_glass = true;

//                     let n_val = sin(dot(in.clip_position.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453;
//                     let noise_vec = vec3<f32>(
//                         fract(n_val * 1.1) - 0.5,
//                         fract(n_val * 1.2) - 0.5,
//                         fract(n_val * 1.3) - 0.5
//                     );

//                     ray_dir = normalize(ray_dir + noise_vec * 0.02);
//                     t += 0.5;
//                 } else {
//                     ray_transmission *= 0.98;
//                     t += 0.2;
//                 }
//             } else {
//                 let interpolated_color = sample_color_trilinear(p);
//                 let edge_softness = 0.035;
//                 let raw_factor = (hit_threshold - d) / edge_softness + 0.5;
//                 let edge_factor = smoothstep(0.0, 1.0, clamp(raw_factor, 0.0, 1.0));

//                 if edge_factor > 0.01 {
//                     sample_color += interpolated_color.rgb * edge_factor * ray_transmission;
//                     ray_transmission *= (1.0 - edge_factor);
//                 }

//                 if edge_factor > 0.92 {
//                     hit = true;
//                     break;
//                 }

//                 t += 0.015;
//             }
//         } else {
//             if in_glass && d > 0.05 {
//                 in_glass = false;
//             }

//             let overstep = 1.15;
//             let step_scale = 1.0 + (t * 0.01);
//             let safe_step = min(abs(d) * overstep * step_scale, 1.2);
//             t += max(safe_step, 0.015);
//         }

//         if ray_transmission < 0.01 { break; }
//         if t > t_max { break; }
//     }

//     if !hit {
//         sample_color += background_color.rgb * ray_transmission;
//     }

//     return vec4<f32>(sample_color, 1.0);
// }

// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     let uv = in.clip_position.xy / screen.size;

//     let tx = clamp(i32(uv.x * 4095.0), 0, 4095);
//     let ty = clamp(i32(uv.y * 4095.0), 0, 4095);

//     let atlas_data = textureLoad(atlas_sdf, vec2<i32>(tx, ty), 0);
//     let atlas_color = textureLoad(atlas_color, vec2<i32>(tx, ty), 0);
//     let dist = atlas_data.r;
//     let color = atlas_color.rgb;

//     if dist > 9.5 {
//         return vec4<f32>(0.05, 0.05, 0.05, 1.0); // Фон пустоты
//     }

//     if dist < 3.0 {
//         let depth_factor = clamp(1.0 - (dist / 5.0), 0.1, 1.0);
//         return vec4<f32>(color * depth_factor, 1.0); // Цвет запеченного объекта
//     }

//     return vec4<f32>(0.0, 1.0, 0.0, 1.0);
// }


// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     let uv = in.clip_position.xy / screen.size;

//     // Срез 256x256
//     let tx = clamp(i32(uv.x * 255.0), 0, 255);
//     let ty = clamp(i32(uv.y * 255.0), 0, 255);

//     var final_dist = 10.0;
//     var final_color = vec3<f32>(0.05, 0.05, 0.05); // Темно-серый фон по умолчанию

//     // Пробегаем по всей глубине Z, чтобы найти сферу без полос
//     for (var tz = 0; tz < 256; tz++) {
//         let atlas_data = textureLoad(atlas_texture, vec3<i32>(tx, ty, tz), 0);
//         let dist = atlas_data.r;
//         let color = atlas_data.gba;

//         // Если нашли объект и он ближе, чем то, что мы видели раньше
//         if dist < final_dist && dist < 5.0 {
//             final_dist = dist;
//             let depth_factor = clamp(1.0 - (dist / 5.0), 0.1, 1.0);
//             final_color = color * depth_factor;
//         }
//     }

//     return vec4<f32>(final_color, 1.0);
// }
//
//
// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     let uv = in.clip_position.xy / screen.size;

//     // Смотрим ровно в центр 3D куба (Z = 128)
//     let tex_coords = vec3<i32>(i32(uv.x * 255.0), i32(uv.y * 255.0), 128);
//     let data = textureLoad(atlas_texture, tex_coords, 0);

//     if data.r > 9.5 { return vec4(0.1, 0.1, 0.1, 1.0); } // Серый фон
//     return vec4(data.gba, 1.0); // Цвет объекта
// }
