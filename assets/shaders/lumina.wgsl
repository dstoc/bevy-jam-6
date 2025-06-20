#import bevy_render::globals::Globals
@group(0) @binding(1) var<uniform> globals: Globals;


#import bevy_sprite::{
    mesh2d_functions as mesh_functions,
    mesh2d_vertex_output::VertexOutput,
}

struct CustomMaterial {
    base_color: vec4<f32>,
    fill_color: vec4<f32>,
    bloom: f32,
    freq: f32,
};

@group(2) @binding(0)
var<uniform> mat: CustomMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let pos: vec2<f32> = in.uv * 2.0 - vec2<f32>(1.0);
    let r: f32 = length(pos);
    let t = globals.time;

    let theta: f32 = atan2(pos.y, pos.x);
    let freq: f32 = mat.freq;
    let n_coord: vec2<f32> = vec2<f32>(cos(theta), sin(theta)) * freq + vec2(t, t); 
    let n: f32 = simplex2d(n_coord);

    let base_radius: f32 = 0.8;
    let amp: f32 = 0.1;
    let radius: f32 = base_radius + amp * n;

    let d: f32 = abs(r - radius);
    let thickness: f32 = 0.04;
    let aa: f32 = 0.002;
    let edge_alpha: f32 = 1.0 - smoothstep(thickness - aa, thickness + aa, d);

    let final_color: vec3<f32> = mix(mat.fill_color.xyz * mat.bloom, mat.base_color.xyz * mat.bloom, edge_alpha);
    let final_alpha: f32 = max(edge_alpha, select(0.0, 1.0, r < radius));

    return vec4<f32>(final_color, final_alpha);
}


// https://gist.github.com/davidar/ebd53bc4d99f8edd63b623ef0439d10c
// Simplex Noise 2D: https://www.shadertoy.com/view/Msf3WH
//
// MIT License
//
// Copyright (c) 2013 Inigo Quilez <https://iquilezles.org/>
// Copyright (c) 2013 Nikita Miropolskiy
// Copyright (c) 2022 David A Roberts <https://davidar.io/>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
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


