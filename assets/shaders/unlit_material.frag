#version 450

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 o_Color;

layout(set = 1, binding = 0) uniform texture2D u_texture;
layout(set = 1, binding = 1) uniform sampler u_image;

void main() {
    o_Color = vec4(texture(sampler2D(u_texture, u_image), in_uv).rgb, 1.0);
}