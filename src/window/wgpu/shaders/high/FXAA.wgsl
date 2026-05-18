struct ScreenUniform {
    size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> screen: ScreenUniform;

@group(1) @binding(0) var scene_texture: texture_2d<f32>;
@group(1) @binding(1) var scene_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    // Генерируем один большой треугольник, покрывающий весь экран
    let x = f32(i32(vertex_index == 1u) * 2);
    let y = f32(i32(vertex_index == 2u) * 2);
    out.uv = vec2<f32>(x, y);
    out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;

    // Получаем размер текстуры для вычисления шага пикселя
    let tex_size = vec2<f32>(textureDimensions(scene_texture));
    let texel_size = 1.0 / tex_size;

    // Считываем яркость (Luma) в текущем пикселе и у соседей
    let rgbM = textureSample(scene_texture, scene_sampler, uv).rgb;
    let rgbNW = textureSample(scene_texture, scene_sampler, uv + vec2<f32>(-1.0, -1.0) * texel_size).rgb;
    let rgbNE = textureSample(scene_texture, scene_sampler, uv + vec2<f32>(1.0, -1.0) * texel_size).rgb;
    let rgbSW = textureSample(scene_texture, scene_sampler, uv + vec2<f32>(-1.0, 1.0) * texel_size).rgb;
    let rgbSE = textureSample(scene_texture, scene_sampler, uv + vec2<f32>(1.0, 1.0) * texel_size).rgb;

    // Приближенный расчет яркости (веса для человеческого глаза)
    let luma = vec3<f32>(0.299, 0.587, 0.114);
    let lumaM = dot(rgbM, luma);
    let lumaNW = dot(rgbNW, luma);
    let lumaNE = dot(rgbNE, luma);
    let lumaSW = dot(rgbSW, luma);
    let lumaSE = dot(rgbSE, luma);

    // Находим минимальную и максимальную яркость вокруг пикселя
    let lumaMin = min(lumaM, min(min(lumaNW, lumaNE), min(lumaSW, lumaSE)));
    let lumaMax = max(lumaM, max(max(lumaNW, lumaNE), max(lumaSW, lumaSE)));

    // Если контраст очень низкий, сглаживание не требуется
    let contrast = lumaMax - lumaMin;
    if contrast < max(0.0312, lumaMax * 0.125) {
        return vec4<f32>(rgbM, 1.0);
    }

    // Направление размытия вдоль градиента контраста
    var dir: vec2<f32>;
    dir.x = -((lumaNW + lumaNE) - (lumaSW + lumaSE));
    dir.y = ((lumaNW + lumaSW) - (lumaNE + lumaSE));

    let dirReduce = max((lumaNW + lumaNE + lumaSW + lumaSE) * 0.0078125, 0.00001);
    let rcpDirMin = 1.0 / (min(abs(dir.x), abs(dir.y)) + dirReduce);

    dir = min(vec2<f32>(8.0, 8.0), max(vec2<f32>(-8.0, -8.0), dir * rcpDirMin)) * texel_size;

    // Смешиваем цвета в направлении градиента
    let rgbA = 0.5 * (textureSample(scene_texture, scene_sampler, uv + dir * (1.0 / 3.0 - 0.5)).rgb +
        textureSample(scene_texture, scene_sampler, uv + dir * (2.0 / 3.0 - 0.5)).rgb);
    let rgbB = rgbA * 0.5 + 0.25 * (textureSample(scene_texture, scene_sampler, uv + dir * (0.0 / 3.0 - 0.5)).rgb +
        textureSample(scene_texture, scene_sampler, uv + dir * (3.0 / 3.0 - 0.5)).rgb);

    let lumaB = dot(rgbB, luma);
    if lumaB < lumaMin || lumaB > lumaMax {
        return vec4<f32>(rgbA, 1.0);
    }
    return vec4<f32>(rgbB, 1.0);
}
