#import bevy_render::globals::Globals
@group(0) @binding(1) var<uniform> globals: Globals;


#import bevy_sprite::{
    mesh2d_functions as mesh_functions,
    mesh2d_vertex_output::VertexOutput,
}

struct CustomMaterial {
    base_color: vec4<f32>,
    bloom: f32,
    wave_freq: f32,
    wave_speed: f32,
};

@group(2) @binding(0)
var<uniform> mat: CustomMaterial;

// Simplex Noise 2D: https://www.shadertoy.com/view/Msf3WH
// TODO: MIT
fn hash(p: vec2<f32>) -> vec2<f32>
{
    let p2 = vec2<f32>( dot(p,vec2<f32>(127.1,311.7)), dot(p,vec2<f32>(269.5,183.3)) );
    return -1.0 + 2.0*fract(sin(p2)*43758.5453123);
}

fn simplex2d(p: vec2<f32>) -> f32
{
    let K1 = 0.366025404; // (sqrt(3)-1)/2;
    let K2 = 0.211324865; // (3-sqrt(3))/6;
    let i = floor( p + (p.x+p.y)*K1 );
    let a = p - i + (i.x+i.y)*K2;
    let o = step(a.yx,a.xy);
    let b = a - o + K2;
    let c = a - 1.0 + 2.0*K2;
    let h = max( 0.5-vec3<f32>(dot(a,a), dot(b,b), dot(c,c) ), vec3<f32>(0.) );
    let n = h*h*h*h*vec3<f32>( dot(a,hash(i+0.0)), dot(b,hash(i+o)), dot(c,hash(i+1.0)));
    return dot( n, vec3<f32>(70.0) );
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let du_dx: f32 = dpdx(in.uv.x);
    let du_dy: f32 = dpdy(in.uv.x);

    let wdx: vec4<f32> = dpdx(in.world_position);
    let wdy: vec4<f32> = dpdy(in.world_position);

    let numerator: vec3<f32> = wdx.xyz * du_dx + wdy.xyz * du_dy;
    let denom: f32 = du_dx * du_dx + du_dy * du_dy;
    let dworld_du: vec3<f32> = numerator / denom;

    let world_length: f32 = length(dworld_du);
    let world_x: f32 = in.uv.x * world_length;

    let t = globals.time;
    let uv = vec2(world_x, in.uv.y);

    var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    for (var i: u32 = 0u; i < 4; i = i + 1u) {
        let r = hash(vec2(f32(i), 0.0));
        let freq: f32 = mat.wave_freq * r.x;
        let speed: f32 = mat.wave_speed + f32(i) * 0.5;
        let wave_amp = 0.5 - (0.02 * f32(i));
        let thickness: f32 = 0.1 - (0.02 * f32(i));

        let wave = simplex2d(vec2(uv.x * freq, t * speed)) * wave_amp;
        let d = abs(uv.y - 0.5 - wave);
        let aa = fwidth(d);
        let line = 1.0 - smoothstep(thickness - aa, thickness + aa, d);
        color += mat.base_color * line;
    }


    return color * mat.bloom;
    // return vec4<f32>(1.0, 1.0, 1.0, line);
}



