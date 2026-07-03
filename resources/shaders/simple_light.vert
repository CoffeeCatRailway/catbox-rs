#version 330 core

uniform mat4 u_projViewMatrix;
uniform mat4 u_viewMatrix;
uniform mat4 u_modelMatrix;
uniform mat4 u_lightSpaceMatrix;

in vec3 i_position;
in vec3 i_normal;
in vec2 i_uv;

out vec3 f_position;
out vec3 f_normal;
out vec2 f_uv;
out vec4 f_positionLightSpace;

void main() {
	vec4 modelPos = u_modelMatrix * vec4(i_position, 1.0);
	gl_Position = u_projViewMatrix * modelPos;
	
	f_position = modelPos.xyz;
    f_normal = mat3(transpose(inverse(u_modelMatrix))) * i_normal;
    f_uv = i_uv;
    f_positionLightSpace = u_lightSpaceMatrix * vec4(f_position, 1.0);
}
