#version 330 core

uniform mat4 u_pvm;

in float i_shapeId;
in vec2 i_position;
in vec3 i_color;
in vec2 i_size;
in float i_rotation;
in float i_outline;

out float g_shapeId;
out vec3 g_color;
out vec2 g_size;
out float g_rotation;
out float g_outline;

void main()
{
	g_shapeId = i_shapeId;
    gl_Position = vec4(i_position, 0., 1.);
    g_color = i_color;
	g_size = i_size;
	g_rotation = i_rotation;
	g_outline = i_outline;
}
