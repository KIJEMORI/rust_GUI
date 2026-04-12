struct ScreenUniform {
    size: vec2<f32>,
};
@group(0) @binding(0) var<uniform> screen: ScreenUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,      // Координаты буквы в атласе
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) canvas_pos: vec2<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;


    let final_pos = model.position;

    let x = (final_pos.x / screen.size.x) * 2.0 - 1.0;
    let y = 1.0 - (final_pos.y / screen.size.y) * 2.0;

    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.color = model.color;
    out.uv = model.uv;

    out.canvas_pos = final_pos;

    return out;
}

@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(t_diffuse, s_diffuse, in.uv).r;
    if (alpha < 0.1) { discard; }
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
