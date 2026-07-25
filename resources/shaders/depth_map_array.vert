#version 460 core

uniform mat4 u_modelMatrix;

in vec3 i_position;

void main() {
    gl_Position = u_modelMatrix * vec4(i_position, 1.0);
}
