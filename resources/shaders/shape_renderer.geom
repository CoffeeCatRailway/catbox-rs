#version 330 core

const float ID_CIRCLE = 0.;
const float ID_BOX = 1.;
const float ID_LINE = 2.;

uniform mat4 u_pvm;

layout (points) in;
layout (triangle_strip, max_vertices = 4) out;

in float g_shapeId[];
in vec3 g_color[];
in vec2 g_size[];
in float g_rotation[];
in float g_outline[];

out float f_shapeId;
out vec3 f_color;
out vec2 f_size;
//out float f_rotation;
out float f_outline;
out vec2 f_uv;

vec2 rotate(vec2 origin, in vec2 p, float rad)
{
	float s = sin(rad);
	float c = cos(rad);
	
	p -= origin;
	
	float nx = p.x * c - p.y * s;
	float ny = p.x * s + p.y * c;
	
	return vec2(nx, ny) + origin;
}

void main()
{
	f_shapeId = g_shapeId[0];
	f_color = g_color[0];
	f_size = g_size[0];
	f_outline = g_outline[0];
	
	vec4 bl, br, tl, tr;
	if (g_shapeId[0] == ID_CIRCLE)
	{
		float r = g_size[0].x;
		bl = vec4(-r, -r, 0., 0.);
		br = vec4(r, -r, 0., 0.);
		tl = vec4(-r, r, 0., 0.);
		tr = vec4(r, r, 0., 0.);
	} else if (g_shapeId[0] == ID_BOX)
	{
		bl = vec4(-g_size[0].x * .5f, -g_size[0].y * .5f, 0., 0.);
		br = vec4(g_size[0].x * .5f, -g_size[0].y * .5f, 0., 0.);
		tl = vec4(-g_size[0].x * .5f, g_size[0].y * .5f, 0., 0.);
		tr = vec4(g_size[0].x * .5f, g_size[0].y * .5f, 0., 0.);
	} else if (g_shapeId[0] == ID_LINE)
	{
		bl = vec4(0., -g_size[0].y * .5f, 0., 0.);
		br = vec4(g_size[0].x, -g_size[0].y * .5f, 0., 0.);
		tl = vec4(0., g_size[0].y * .5f, 0., 0.);
		tr = vec4(g_size[0].x, g_size[0].y * .5f, 0., 0.);
	}
	
	if (g_shapeId[0] != ID_CIRCLE)
	{
		bl.xy = rotate(vec2(0.), bl.xy, g_rotation[0]);
		br.xy = rotate(vec2(0.), br.xy, g_rotation[0]);
		tl.xy = rotate(vec2(0.), tl.xy, g_rotation[0]);
		tr.xy = rotate(vec2(0.), tr.xy, g_rotation[0]);
	}
	
    vec4 center = gl_in[0].gl_Position;
	
	gl_Position = u_pvm * (center + bl);
	f_uv = vec2(-1., -1.);
	EmitVertex();
	
	gl_Position = u_pvm * (center + br);
	f_uv = vec2(1., -1.);
	EmitVertex();
	
	gl_Position = u_pvm * (center + tl);
	f_uv = vec2(-1., 1.);
	EmitVertex();
	
	gl_Position = u_pvm * (center + tr);
	f_uv = vec2(1., 1.);
	EmitVertex();
	
	EndPrimitive();
}
