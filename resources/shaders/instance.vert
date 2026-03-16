#version 330 core

uniform mat4 u_pvm;

in vec3 i_position;

in mat4 i_model;
in vec4 i_color;

out vec4 f_color;

void main() {
    gl_Position = u_pvm * i_model * vec4(i_position, 1.0);
    f_color = i_color;
}