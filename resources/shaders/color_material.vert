#version 460 core

uniform mat4 u_projViewMatrix;
uniform mat4 u_modelMatrix;

in vec3 i_position;
in vec2 i_uv;

out vec2 f_uv;

void main() {
    gl_Position = u_projViewMatrix * u_modelMatrix * vec4(i_position, 1.0);

    f_uv = i_uv;
}
