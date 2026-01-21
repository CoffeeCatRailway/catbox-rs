#version 330 core

const float ID_CIRCLE = 0.;
const float ID_BOX = 1.;
const float ID_LINE = 2.;

in float f_shapeId;
in vec3 f_color;
in vec2 f_size;
//in float f_rotation;
in float f_outline;
in vec2 f_uv;

out vec4 o_color;

void main()
{
	vec2 uvSize = f_uv * f_size;
	bool isCircle = f_shapeId == ID_CIRCLE;
	if (isCircle && length(f_uv) >= 1.)
		discard;

    o_color = vec4(f_color, 1.);
//	o_color = vec4(f_uv * .5 + .5, 0., 1.);
	
	bool circleOutline = isCircle && length(uvSize) > f_size.x - f_outline;
	bool boxLineOutline = abs(uvSize.x) > f_size.x - f_outline || abs(uvSize.y) > f_size.y - f_outline;
	if (circleOutline || boxLineOutline)
        o_color =	vec4(0., 0., 0., 1.);
}
