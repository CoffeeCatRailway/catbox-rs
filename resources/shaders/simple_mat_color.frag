#version 330 core

struct Material {
    vec3 color;
    sampler2D diffuse;
    vec4 specular; // not used here
};

uniform Material u_material;

in vec2 f_uv;

out vec4 o_color;

void main() {
	o_color = texture(u_material.diffuse, f_uv) * vec4(u_material.color, 1.0);
}
