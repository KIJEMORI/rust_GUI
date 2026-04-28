// @group(0) @binding(0) var scene_texture: texture_2d<f32>;
// @group(0) @binding(1) var scene_sampler: sampler;

// fn get_luma(rgb: vec3<f32>) -> f32 {
//     return dot(rgb, vec3<f32>(0.299, 0.587, 0.114));
// }

// @fragment
// fn fs_post_process(in: VertexOutput) -> @location(0) vec4<f32> {
//     let uv = in.canvas_pos / screen.size;
//     let texel_size = 1.0 / screen.size;

//     // Берем соседние пиксели (крестом)
//     let rgbM = textureSample(scene_texture, scene_sampler, uv).rgb;
//     let rgbNW = textureSample(scene_texture, scene_sampler, uv + vec2(-1.0, -1.0) * texel_size).rgb;
//     let rgbNE = textureSample(scene_texture, scene_sampler, uv + vec2(1.0, -1.0) * texel_size).rgb;
//     let rgbSW = textureSample(scene_texture, scene_sampler, uv + vec2(-1.0, 1.0) * texel_size).rgb;
//     let rgbSE = textureSample(scene_texture, scene_sampler, uv + vec2(1.0, 1.0) * texel_size).rgb;

//     let lumaM = get_luma(rgbM);
//     let lumaNW = get_luma(rgbNW);
//     let lumaNE = get_luma(rgbNE);
//     let lumaSW = get_luma(rgbSW);
//     let lumaSE = get_luma(rgbSE);

//     // Вычисляем направление градиента (где край объекта)
//     var dir = vec2<f32>(
//         -((lumaNW + lumaNE) - (lumaSW + lumaSE)),
//         ((lumaNW + lumaSW) - (lumaNE + lumaSE))
//     );

//     let dir_reduce = max((lumaNW + lumaNE + lumaSW + lumaSE) * 0.03125, 0.0001);
//     let span = 1.0 / (min(abs(dir.x), abs(dir.y)) + dir_reduce);

//     dir = clamp(dir * span, vec2(-8.0), vec2(8.0)) * texel_size;

//     // Смешиваем пиксели вдоль направления края
//     let rgbA = 0.5 * (textureSample(scene_texture, scene_sampler, uv + dir * (1.0 / 3.0 - 0.5)).rgb +
//         textureSample(scene_texture, scene_sampler, uv + dir * (2.0 / 3.0 - 0.5)).rgb);
//     let rgbB = rgbA * 0.5 + 0.25 * (textureSample(scene_texture, scene_sampler, uv + dir * (0.0 / 3.0 - 0.5)).rgb +
//         textureSample(scene_texture, scene_sampler, uv + dir * (3.0 / 3.0 - 0.5)).rgb);

//     let lumaB = get_luma(rgbB);
//     let luma_min = min(lumaM, min(min(lumaNW, lumaNE), min(lumaSW, lumaSE)));
//     let luma_max = max(lumaM, max(max(lumaNW, lumaNE), max(lumaSW, lumaSE)));

//     // Если результат смешивания слишком "вылетает" по яркости, возвращаем исходный вариант A
//     if lumaB < luma_min || lumaB > luma_max {
//         return vec4(rgbA, 1.0);
//     } else {
//         return vec4(rgbB, 1.0);
//     }
// }
