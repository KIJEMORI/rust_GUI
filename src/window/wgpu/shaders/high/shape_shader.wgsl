struct ScreenUniform {
    size: vec2<f32>,
};
@group(0) @binding(0) var<uniform> screen: ScreenUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) p_a: vec2<f32>, // uv для текста
    @location(3) p_b: vec2<f32>,
    @location(4) params: vec4<f32>, // [радиус/толщина, тип, сглаживание, ширина рамки]
    @location(5) border_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) canvas_pos: vec2<f32>,
    @location(2) p_a: vec2<f32>,       // Центр или Старт
    @location(3) p_b: vec2<f32>,       // Размер или Конец
    @location(4) params: vec4<f32>,    // [радиус/толщина, тип, сглаживание, ширина рамки]
    @location(5) border_color: vec4<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let final_pos = model.position;

    // Перевод из пикселей в NDC (-1.0 to 1.0)
    let x = (final_pos.x / screen.size.x) * 2.0 - 1.0;
    let y = 1.0 - (final_pos.y / screen.size.y) * 2.0;

    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.color = model.color;
    out.canvas_pos = final_pos;
    out.p_a = model.p_a;
    out.p_b = model.p_b;
    out.params = model.params;
    out.border_color = model.border_color;

    return out;
}

// SDF Скругленного прямоугольника
fn sd_rounded_rect(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec2<f32>(r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r;
}

// SDF Линии (Капсула)
fn sd_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    let params = in.params;

    let smoothing = params.z;
    let thickness = params.w; // Толщина рамки
    var d_outer: f32;
    var d_inner: f32;

    let obj_type = params.y;

    if (obj_type > 1.5){ // Текст
        // let dist = textureSample(t_diffuse, s_diffuse, in.p_a).r;
        // let smoothing = fwidth(dist);
        // let alpha = smoothstep(0.5 - smoothing, 0.5 + smoothing, dist);
        // if (alpha <= 0.0) { discard; }
        // return vec4<f32>(in.color.rgb, in.color.a * alpha);
        // let alpha = textureSample(t_diffuse, s_diffuse, in.p_a).r;
        // if (alpha < 0.1) { discard; }
        // return vec4<f32>(in.color.rgb, in.color.a * alpha);
        let dist = textureSample(t_diffuse, s_diffuse, in.p_a).r;

        let smoothing = fwidth(dist) * 0.7;

        let threshold = 0.54;
        let alpha = smoothstep(threshold - smoothing, threshold + smoothing, dist);
        let final_alpha = pow(alpha, 1.0 / 2.2);

        if (final_alpha <= 0.05) { discard; }

        // Умножаем альфу на итоговый цвет
        return vec4<f32>(in.color.rgb, in.color.a * final_alpha);

    }
    else if (obj_type > 0.5) { // Линия
        d_outer = sd_segment(in.canvas_pos, in.p_a, in.p_b) - params.x;
        d_inner = 1.0;
    } else { // Прямоугольник
        let p = in.canvas_pos - in.p_a;
        let half_size = in.p_b * 0.5;
        let radius = params.x;

        // Дистанция до внешнего края
        d_outer = sd_rounded_rect(p, half_size, radius);
        // Дистанция до внутреннего края (уменьшаем размер на толщину)
        d_inner = sd_rounded_rect(p, half_size - vec2<f32>(thickness), max(0.0, radius - thickness));
    }

    if (d_outer > 0.0) {
        discard;
    }

    // alpha_outer — общая видимость объекта (сглаживание внешнего края)
    // let alpha_outer = clamp(0.5 - d_outer / smoothing, 0.0, 1.0);
    let alpha_outer = smoothstep(smoothing, -smoothing, d_outer);

    // fill_mask — определяет, где начинается основной цвет
    let fill_mask = clamp(0.5 - d_inner / smoothing, 0.0, 1.0);

    if (alpha_outer <= 0.0) { discard; }

    // Смешиваем цвета
    // Если толщина 0, рисуем только основной цвет
    var final_color: vec4<f32>;
    if (thickness > 0.0) {
        // Сначала берем цвет рамки, потом "накладываем" на него основной цвет по маске
        let mixed_rgb = mix(in.border_color.rgb, in.color.rgb, fill_mask);
        let mixed_a = mix(in.border_color.a, in.color.a, fill_mask);
        final_color = vec4<f32>(mixed_rgb, mixed_a * alpha_outer);
    } else {
        final_color = vec4<f32>(in.color.rgb, in.color.a * alpha_outer);
    }

    return final_color;
}
