// struct ScreenUniform {
//     size: vec2<f32>,
// };
// @group(0) @binding(0) var<uniform> screen: ScreenUniform;

// struct CameraUniform {
//     view_proj: mat4x4<f32>,
//     inv_view_proj: mat4x4<f32>,
//     camera_pos: vec3<f32>,
//     padding: f32,
// };

// @group(1) @binding(0) var<uniform> camera: CameraUniform;
// @group(1) @binding(1) var<storage, read> instances: array<Instance3DData>;

// struct Instance3DData {
//     transform: mat4x4<f32>,
//     params: vec4<f32>,
//     color: u32,
//     entity_id: u32,
//     padding: vec2<u32>,
// };

// // Тот самый универсальный вход, как в 2D
// struct VertexInput {
//     @location(0) position: vec2<f32>,
//     @location(1) color: vec4<f32>,
//     @location(2) p_a: vec2<f32>,       // viewport_min
//     @location(3) p_b: vec2<f32>,       // viewport_max
//     @location(4) params: vec4<f32>,
//     @location(5) border_color: vec4<f32>,
// };

// struct VertexOutput {
//     @builtin(position) clip_position: vec4<f32>,
//     @location(0) canvas_pos: vec2<f32>,
//     @interpolate(flat) @location(1) instance_idx: u32,
//     @location(2) viewport_rect: vec4<f32>,
// };

// fn unpack_color_unorm(c: u32) -> vec4<f32> {
//     return vec4<f32>(
//         f32(c & 0xffu) / 255.0,
//         f32((c >> 8u) & 0xffu) / 255.0,
//         f32((c >> 16u) & 0xffu) / 255.0,
//         f32((c >> 24u) & 0xffu) / 255.0
//     );
// }

// @vertex
// fn vs_main(model: VertexInput, @builtin(instance_index) i_idx: u32) -> VertexOutput {
//     var out: VertexOutput;

//     let x = (model.position.x / screen.size.x) * 2.0 - 1.0;
//     let y = 1.0 - (model.position.y / screen.size.y) * 2.0;

//     out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
//     out.canvas_pos = model.position;
//     out.instance_idx = i_idx;
//     out.viewport_rect = vec4<f32>(model.p_a, model.p_b);
//     return out;
// }

// // --- SDF функции ---
// fn sd_sphere(p: vec3<f32>, s: f32) -> f32 {
//     return length(p) - s;
// }

// fn sd_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
//     let q = abs(p) - b;
//     return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
// }

// fn sd_torus(p: vec3<f32>, t: vec2<f32>) -> f32 {
//     let q = vec2<f32>(length(p.xz) - t.x, p.y);
//     return length(q) - t.y;
// }

// // Цилиндр. h.x - радиус, h.y - половина высоты
// fn sd_cylinder(p: vec3<f32>, h: vec2<f32>) -> f32 {
//     let d = abs(vec2<f32>(length(p.xz), p.y)) - h;
//     return min(max(d.x, d.y), 0.0) + length(max(d, vec2<f32>(0.0)));
// }

// // Капсула. h - половина расстояния между центрами сфер, r - радиус
// fn sd_capsule(p: vec3<f32>, h: f32, r: f32) -> f32 {
//     let pa = p - vec3<f32>(0.0, -h, 0.0);
//     let ba = vec3<f32>(0.0, h * 2.0, 0.0);
//     let h_val = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
//     return length(pa - ba * h_val) - r;
// }

// fn smin(a: f32, b: f32, k: f32) -> f32 {
//     let h = max(k - abs(a - b), 0.0) / k;
//     return min(a, b) - h * h * k * (1.0 / 4.0);
// }

// @group(2) @binding(0) var sdf_textures: texture_3d_array<f32>;
// @group(2) @binding(1) var sdf_sampler: sampler;

// fn scene_sdf(world_p: vec3<f32>, start_idx: u32, bone_count: u32) -> f32 {
//     var final_d = 1000.0;

//     for (var i = 0u; i < bone_count; i++) {
//         let bone = instances[start_idx + i];

//         // Переводим луч в локальное пространство кости
//         let local_p = (bone.transform * vec4<f32>(world_p, 1.0)).xyz;

//         var d = 1000.0;
//         let type_tag = bone.params.x; // 0.0 - капсула, 1.0 - запеченная текстура

//         if type_tag < 0.5 {
//             // Аналитическая капсула (основа тела)
//             d = sd_capsule(local_p, bone.params.y, bone.params.z);
//         } else {
//             // Запеченная деталь (голова, кисть и т.д.)
//             // Используем textureSampleLevel для скорости в Raymarching
//             d = textureSampleLevel(sdf_textures, sdf_sampler, local_p * 0.5 + 0.5, u32(bone.params.z), 0.0).r;
//         }

//         // Слияние! k - это bone.params.w (коэффициент мягкости)
//         final_d = smin(final_d, d, bone.params.w);
//     }

//     return final_d;
// }

// fn get_normal(p: vec3<f32>, start_idx: u32, count: u32) -> vec3<f32> {
//     let e = 0.01; // Подбери значение от 0.01 до 0.05
//     let dx = vec3<f32>(e, 0.0, 0.0);
//     let dy = vec3<f32>(0.0, e, 0.0);
//     let dz = vec3<f32>(0.0, 0.0, e);

//     // Берем разницу с двух сторон от точки - это дает более точный вектор
//     return normalize(vec3<f32>(
//         scene_sdf(p + dx, start_idx, count) - scene_sdf(p - dx, start_idx, count),
//         scene_sdf(p + dy, start_idx, count) - scene_sdf(p - dy, start_idx, count),
//         scene_sdf(p + dz, start_idx, count) - scene_sdf(p - dz, start_idx, count)
//     ));
// }

// struct FragmentOutput {
//     @location(0) color: vec4<f32>,
//     @builtin(frag_depth) depth: f32,
// };

// @fragment
// fn fs_main(in: VertexOutput) -> FragmentOutput {

//     // var exmpl: FragmentOutput;
//     // exmpl.color = vec4<f32>(0.0, 1.0, 0.0, 1.0);
//     // exmpl.depth = 1.0;
//     // return exmpl;

//     // Обрезка по границам панели (Clip Rect)
//     if in.canvas_pos.x < in.viewport_rect.x || in.canvas_pos.y < in.viewport_rect.y ||
//         in.canvas_pos.x > in.viewport_rect.z || in.canvas_pos.y > in.viewport_rect.w {
//         discard;
//     }

//     let first_inst_idx = in.instance_idx;
//     let group_count = u32(instances[first_inst_idx].params.w);
//     let base_color = unpack_color_unorm(instances[first_inst_idx].color);

//     var total_rgb = vec3<f32>(0.0);
//     var total_alpha = 0.0;
//     var final_p = vec3<f32>(0.0);

//     let rect_min = in.viewport_rect.xy;
//     let rect_max = in.viewport_rect.zw;
//     let rect_size = rect_max - rect_min;

//     let rect_center = rect_min + rect_size * 0.5;

//     let samples = 2; // SSAA 2x2
//     for (var i = 0; i < samples; i++) {
//         for (var j = 0; j < samples; j++) {
//             let offset = (vec2<f32>(f32(i), f32(j)) - 0.5) / 2.0;
//             let local_pixel = (in.canvas_pos + offset - rect_center);

//             let ndc = ((in.canvas_pos + offset) / screen.size.x) * 2.0 - 1.0;

//             let global_ndc = ((in.canvas_pos + offset) / screen.size) * 2.0 - 1.0;
//             let ray_ndc = vec2<f32>(global_ndc.x, -global_ndc.y);

//             let target_near = camera.inv_view_proj * vec4<f32>(ray_ndc, 0.0, 1.0);
//             let target_far = camera.inv_view_proj * vec4<f32>(ray_ndc, 1.0, 1.0);
//             let ray_origin = camera.camera_pos;
//             let ray_dir = normalize(target_far.xyz / target_far.w - target_near.xyz / target_near.w);

//             var t = 0.0;
//             var hit = false;
//             var p = ray_origin;

//             for (var step = 0; step < 80; step++) {
//                 p = ray_origin + ray_dir * t;
//                 let d = scene_sdf(p, first_inst_idx, group_count);
//                 if d < 0.001 { hit = true; break; }
//                 t += d;
//                 if t > 1000.0 { break; }
//             }

//             if hit {
//                 let n = get_normal(p, first_inst_idx, group_count);
//                 let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
//                 let view_dir = normalize(camera.camera_pos - p);

//                 let diff = max(dot(n, light_dir), 0.0) * 0.7 + 0.3;
//                 let reflect_dir = reflect(-light_dir, n);
//                 let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);

//                 total_rgb += (base_color.rgb * diff) + vec3<f32>(spec * 0.5);
//                 total_alpha += 1.0;
//                 final_p = p;
//             }
//         }
//     }

//     let final_alpha = total_alpha / 4.0;
//     if final_alpha <= 0.0 { discard; }

//     var out: FragmentOutput;
//     out.color = vec4<f32>(total_rgb / 4.0, base_color.a * final_alpha);

//     let clip_pos = camera.view_proj * vec4<f32>(final_p, 1.0);
//     out.depth = clip_pos.z / clip_pos.w;

//     return out;
// }
