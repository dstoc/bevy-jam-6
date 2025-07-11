#import bevy_render::globals::Globals
@group(0) @binding(1) var<uniform> globals: Globals;


#import bevy_sprite::{
    mesh2d_functions as mesh_functions,
    mesh2d_vertex_output::VertexOutput,
}

struct CustomMaterial {
    camera: vec4<f32>,
};

@group(2) @binding(0)
var<uniform> mat: CustomMaterial;

fn rand(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453123);
}

@fragment
fn fragment(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    let grid_size = vec2<f32>(0.01, 0.01);
    let star_radius = 0.01;
    let twinkle_speed = 2.0;

    let seed0 = rand(floor((in.world_position.xy) * grid_size));

    const LAYERS: f32 = 4.0;
    let layer: f32 = floor(seed0 * f32(LAYERS));
    let parallax = (layer + 1) * 0.1;

    let world_pos = in.world_position.xy + mat.camera.xy * parallax;

    let st = world_pos * grid_size;
    let cell = floor(st);
    let cell_uv = fract(st);

    let seed1 = rand(cell);
    let star_x = rand(cell + vec2<f32>(1.0, 0.0));
    let star_y = rand(cell + vec2<f32>(0.0, 1.0));
    let star_pos = vec2<f32>(star_x, star_y);

    let d = length(cell_uv - star_pos);


    var color = vec3<f32>(0.0);

    if (d < star_radius) {
        let phase = seed0 * 6.283185307 + globals.time * twinkle_speed;
        let tw = 0.5 + 0.5 * sin(phase);
        let intensity = (1.0 - d / star_radius) * tw;
        color = vec3<f32>(intensity);
        return vec4<f32>(vec3<f32>(1.0), intensity);
    } else {
        return vec4<f32>(0.0);
        
    }

}
