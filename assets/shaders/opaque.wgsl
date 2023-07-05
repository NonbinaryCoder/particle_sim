#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

#import bevy_pbr::mesh_functions

@vertex
fn vertex(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(
        mesh.model,
        vec4<f32>(vertex.position, 1.0),
    );
    out.color = vertex.color;
    return out;
}

struct FragmentInput {
    @location(0) color: vec4<f32>,
}

@fragment
fn fragment(input: FragmentInput) -> @location(0) vec4<f32> {
    return input.color;
}
