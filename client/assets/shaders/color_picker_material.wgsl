#import bevy_ui::ui_vertex_output::UiVertexOutput;

@group(1) @binding(0) var<uniform> color: vec4<f32>;
@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    // Normalized pixel coordinates (from 0 to 1)
    let uv = in.uv;

    // Time varying pixel color
    let white = vec3(1.0, 1.0, 1.0);
    let h_lerp_col = mix(white, color.xyz, uv.x);

    let black = vec3(0.0, 0.0, 0.0);
    let lerped_col = mix(h_lerp_col, black, uv.y);
    let c_lerped_col = pow(lerped_col, vec3(2.2));
    // Output to screen
    return vec4(c_lerped_col,1.0);
}
