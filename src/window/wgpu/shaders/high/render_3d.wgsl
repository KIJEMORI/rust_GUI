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
    @location(1) color: vec4<f32>, // Не используется в Raymarching, но оставлено для структуры
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) ray_ndc: vec2<f32>, // Передаем готовые координаты для луча
};
// --- Структура выхода фрагментного шейдера ---
struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32, // Позволяет корректно взаимодействовать с Z-буфером
};

@group(0) @binding(0) var<uniform> screen: ScreenUniform;
@group(1) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(1) var atlas_texture: texture_3d<f32>;
@group(1) @binding(2) var atlas_sampler: sampler;

fn fetch_voxel(coords: vec3<u32>) -> vec4<f32> {
    let clamped_coords = clamp(vec3<i32>(coords), vec3<i32>(0), vec3<i32>(255));
    let data = textureLoad(atlas_texture, clamped_coords, 0);

    // Если r близок к 1.0 (это наш 10.0 из прошлого шейдера), возвращаем пустоту
    if data.r > 0.99 {
        return vec4<f32>(10.0, 0.0, 0.0, 0.0);
    }

    // Восстановление SDF обратно в мировые единицы
    let world_sdf = (data.r - 0.5) * 32.0;

    return vec4<f32>(world_sdf, data.gba);
}

// fn sample_atlas(p: vec3<f32>) -> vec4<f32> {
//     let grid_p = p + 16.0;

//     // Оставляем безопасный зазор в 1 воксель (1.0 / 8.0 = 0.125) для фильтрации
//     if any(grid_p < vec3<f32>(0.125)) || any(grid_p >= vec3<f32>(31.875)) {
//         return vec4<f32>(10.0, 0.0, 0.0, 0.0);
//     }

//     // Вычисляем координаты вокселя и дробную часть положения внутри него
//     let scaled_p = grid_p * 8.0;
//     let v_p = vec3<u32>(floor(scaled_p));
//     let f_p = fract(scaled_p);

//     // Выборка 8 соседних вокселей (вершины куба)
//     let v000 = fetch_voxel(v_p + vec3<u32>(0u, 0u, 0u));
//     let v100 = fetch_voxel(v_p + vec3<u32>(1u, 0u, 0u));
//     let v010 = fetch_voxel(v_p + vec3<u32>(0u, 1u, 0u));
//     let v110 = fetch_voxel(v_p + vec3<u32>(1u, 1u, 0u));
//     let v001 = fetch_voxel(v_p + vec3<u32>(0u, 0u, 1u));
//     let v101 = fetch_voxel(v_p + vec3<u32>(1u, 0u, 1u));
//     let v011 = fetch_voxel(v_p + vec3<u32>(0u, 1u, 1u));
//     let v111 = fetch_voxel(v_p + vec3<u32>(1u, 1u, 1u));

//     // Плавное трилинейное смешивание всех 8 значений
//     let mix_z0 = mix(mix(v000, v100, f_p.x), mix(v010, v110, f_p.x), f_p.y);
//     let mix_z1 = mix(mix(v001, v101, f_p.x), mix(v011, v111, f_p.x), f_p.y);

//     return mix(mix_z0, mix_z1, f_p.z);
// }

fn sample_atlas(p: vec3<f32>) -> vec4<f32> {
    let grid_p = p + 16.0;

    // Оставляем безопасный зазор на краях
    if any(grid_p < vec3<f32>(0.1)) || any(grid_p >= vec3<f32>(31.9)) {
        return vec4<f32>(10.0, 0.0, 0.0, 0.0);
    }

    // Переводим мировые координаты в нормализованные текстурные [0.0, 1.0]
    let uvw = grid_p / 32.0;

    // textureSampleLevel извлекает данные с аппаратной трилинейной фильтрацией
    let data = textureSampleLevel(atlas_texture, atlas_sampler, uvw, 0.0);

    if data.r > 0.99 {
        return vec4<f32>(10.0, 0.0, 0.0, 0.0);
    }

    // Распаковка SDF из диапазона [0.0, 1.0] в мировые метры
    let world_sdf = (data.r - 0.5) * 32.0;

    return vec4<f32>(world_sdf, data.gba);
}

// Расчет нормали через градиент SDF
fn get_normal(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.25, 0.0);
    return normalize(vec3<f32>(
        sample_atlas(p + e.xyy).r - sample_atlas(p - e.xyy).r,
        sample_atlas(p + e.yxy).r - sample_atlas(p - e.yxy).r,
        sample_atlas(p + e.yyx).r - sample_atlas(p - e.yyx).r
    ));
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let x = (model.position.x / screen.size.x) * 2.0 - 1.0;
    let y = 1.0 - (model.position.y / screen.size.y) * 2.0;

    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.ray_ndc = vec2<f32>(x, y); // Инвертируем Y прямо тут для WebGPU
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let ray_ndc = in.ray_ndc;

    let target_far = camera.inv_view_proj * vec4<f32>(ray_ndc, 1.0, 1.0);
    let world_far = target_far.xyz / target_far.w;

    let ray_dir = normalize(world_far - camera.camera_pos);
    let ray_origin = camera.camera_pos;

    // Границы SDF сетки (от -16 до +16)
    let box_min = vec3<f32>(-16.0);
    let box_max = vec3<f32>(16.0);

    // 1. Увеличиваем t_max до максимальной дистанции рендеринга вашей камеры
    var t_min: f32 = 0.0;
    var t_max: f32 = 500.0;

    let t1 = (box_min - ray_origin) / ray_dir;
    let t2 = (box_max - ray_origin) / ray_dir;
    let t_near = min(t1, t2);
    let t_far = max(t1, t2);

    t_min = max(t_min, max(t_near.x, max(t_near.y, t_near.z)));
    t_max = min(t_max, min(t_far.x, min(t_far.y, t_far.z)));

    // Если луч вообще не пересекает куб сцены, пропускаем пиксель
    if t_min > t_max { discard; }

    // Корректно стартуем с t_min (но не меньше 0, если камера внутри куба)
    var t = max(0.0, t_min);
    var hit_data = vec4<f32>(10.0, 0.0, 0.0, 0.0);

    for (var i = 0; i < 200; i++) {
        let p = ray_origin + ray_dir * t;
        hit_data = sample_atlas(p);
        let d = hit_data.r;

        if d < 0.002 {
            let n = get_normal(p);
            let light_dir = normalize(vec3<f32>(0.577, 0.577, 0.577));
            let NdotL = dot(n, light_dir);
            // Мягкое распределение света по поверхности (модель Half-Lambert)
            let diff = pow(NdotL * 0.5 + 0.5, 2.0);

            var out: FragmentOutput;
            out.color = vec4<f32>(hit_data.gba * diff, 1.0);

            let clip_pos = camera.view_proj * vec4<f32>(p, 1.0);
            out.depth = clip_pos.z / clip_pos.w;
            return out;
        }

        // Продвигаем луч вперед
        let safe_step = min(d, 0.5);
        t += max(safe_step, 0.005);

        if t > t_max { break; }
    }

    discard;
}



// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     let uv = in.clip_position.xy / screen.size;

//     let tx = clamp(i32(uv.x * 2047.0), 0, 2047);
//     let ty = clamp(i32(uv.y * 2047.0), 0, 2047);

//     let atlas_data = textureLoad(atlas_texture, vec2<i32>(tx, ty), 0);

//     let dist = atlas_data.r;
//     let color = atlas_data.gba;

//     // --- Дальше твоя логика визуализации ---
//     if dist > 9.5 {
//         return vec4<f32>(0.05, 0.05, 0.05, 1.0); // Фон пустоты
//     }

//     //if dist < 3.0 {
//     let depth_factor = clamp(1.0 - (dist / 5.0), 0.1, 1.0);
//     return vec4<f32>(color * depth_factor, 1.0); // Цвет запеченного объекта
//     //}

//     //return vec4<f32>(0.0, 1.0, 0.0, 1.0);
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
