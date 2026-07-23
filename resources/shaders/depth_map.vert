#version 460 core

uniform mat4 u_modelMatrix;
uniform mat4 u_lightSpaceMatrix;

in vec3 i_position;

void main() {
    gl_Position = u_lightSpaceMatrix * u_modelMatrix * vec4(i_position, 1.0);
}
