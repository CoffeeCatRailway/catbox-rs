#version 330 core

in vec3 f_position;
in vec3 f_normal;
in vec3 f_color;

out vec4 o_color;

void main() {
	vec3 normal = normalize(f_normal);
//	vec3 normal = normalize(cross(dFdx(f_position), dFdy(f_position)));

    vec3 lightPos = vec3(0.0, 100.0, 100.0);
	vec3 lightDir = normalize(lightPos - f_position);
//	float diff = max(dot(normal, lightDir), 0.0);
	float diff = dot(normal, lightDir) * 0.5 + 0.5;
	
	vec3 diffuse = diff * vec3(1.0);
	o_color = vec4(f_color * diffuse, 1.0);
}
