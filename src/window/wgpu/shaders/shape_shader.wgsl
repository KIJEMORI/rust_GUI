struct ScreenUniform {
    size: vec2<f32>,
};
@group(0) @binding(0) var<uniform> screen: ScreenUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) clip: vec4<f32>,
    @location(3) p_a: vec2<f32>,
    @location(4) p_b: vec2<f32>,
    @location(5) params: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) canvas_pos: vec2<f32>,
    @location(2) clip_rect: vec4<f32>,
    @location(3) p_a: vec2<f32>,       // Центр или Старт
    @location(4) p_b: vec2<f32>,       // Размер или Конец
    @location(5) params: vec4<f32>,    // [радиус/толщина, тип, сглаживание, _]
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Перевод из пикселей в NDC (-1.0 to 1.0)
    let x = (model.position.x / screen.size.x) * 2.0 - 1.0;
    let y = 1.0 - (model.position.y / screen.size.y) * 2.0;

    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.color = model.color;
    out.canvas_pos = model.position;
    out.clip_rect = model.clip;
    out.p_a = model.p_a;
    out.p_b = model.p_b;
    out.params = model.params;

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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.canvas_pos.x < in.clip_rect.x || in.canvas_pos.x > in.clip_rect.z ||
        in.canvas_pos.y < in.clip_rect.y || in.canvas_pos.y > in.clip_rect.w) {
        discard;
    }

    var distance: f32;


    if (in.params.y < 0.5) {
        // ТИП Прямоугольник (p_a - центр, p_b - размер)
        let p = in.canvas_pos - in.p_a;
        let half_size = in.p_b * 0.5;
        distance = sd_rounded_rect(p, half_size, in.params.x);
    } else {
        // ТИП Линия (p_a - старт, p_b - конец, params.x - толщина)
        distance = sd_segment(in.canvas_pos, in.p_a, in.p_b) - in.params.x;
    }

    // Сглаживание края
    // params.z — это ширина сглаживания (обычно 1.0)
    // let alpha = 1.0 - smoothstep(-in.params.z, 0.0, distance);

    let alpha = clamp(0.5 - distance / in.params.z, 0.0, 1.0);

    if (alpha <= 0.0) {
        discard;
    }

    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
