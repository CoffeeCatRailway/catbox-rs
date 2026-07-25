#version 460 core

layout(triangles, invocations = 5) in;
layout(triangle_strip, max_vertices = 3) out;

layout(std140) uniform LightSpaceMats {
    mat4 u_lightSpaceMats[16];
};
//uniform mat4 u_lightSpaceMats[16];

void main() {
    for (int i = 0; i < 3; i++) {
        gl_Position = u_lightSpaceMats[gl_InvocationID] * gl_in[i].gl_Position;
        gl_Layer = gl_InvocationID;
        EmitVertex();
    }
    EndPrimitive();
}
