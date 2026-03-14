#version 330 core

uniform mat4 u_pvm;

in vec3 i_position;
in vec3 i_color;
in mat4 i_model;

out vec3 f_color;

void main() {
    gl_Position = u_pvm * i_model * vec4(i_position, 1.0);
    f_color = i_color;
}