#version 330 core

uniform mat4 u_projViewMatrix;
uniform mat4 u_modelMatrix;

in vec3 i_position;
in vec3 i_normal;
in vec3 i_color;

out vec3 f_position;
out vec3 f_normal;
out vec3 f_color;

void main() {
	vec4 modelPos = u_modelMatrix * vec4(i_position, 1.0);
	gl_Position = u_projViewMatrix * modelPos;
	
	f_position = modelPos.xyz;
	f_normal = (u_modelMatrix * vec4(i_normal, 1.0)).xyz;
//    f_normal = mat3(transpose(inverse(u_modelMatrix))) * i_normal;
	f_color = i_color;
}
