#version 330 core

uniform mat4 u_projViewMatrix;
uniform mat4 u_modelMatrix;

in vec3 i_position;
in vec3 i_normal;
in vec3 i_color;

out vec3 f_position;
out vec3 f_normal;
out vec3 f_color;

out vec3 f_lightPos;

void main() {
	vec4 modelPos = u_modelMatrix * vec4(i_position, 1.0);
	gl_Position = u_projViewMatrix * modelPos;
	
	f_position = modelPos.xyz;
	f_normal = i_normal;
	f_color = i_color;
	
	f_lightPos = (u_modelMatrix * vec4(0.0, 100.0, 100.0, 1.0)).xyz;
}
