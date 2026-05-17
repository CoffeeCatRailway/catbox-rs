#version 330 core

uniform mat4 u_projViewMatrix;

in vec3 i_position;
in vec3 i_color;

out vec3 f_color;

void main() {
	gl_Position = u_projViewMatrix * vec4(i_position, 1.0);
	f_color = i_color;
}
