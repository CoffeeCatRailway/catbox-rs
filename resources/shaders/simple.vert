#version 330 core

uniform mat4 u_pvm;

in vec3 i_position;
in vec3 i_normal;
in vec3 i_color;

out vec3 f_position;
out vec3 f_normal;
out vec3 f_color;

void main() {
	gl_Position = u_pvm * vec4(i_position, 1.);
	f_position = i_position;
	f_normal = i_normal;
	f_color = i_color;
}
