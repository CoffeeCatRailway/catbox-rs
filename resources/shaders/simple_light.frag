#version 330 core

const uint LIGHT_DIRECTIONAL = 0u;
const uint LIGHT_POINT = 0u;

struct Light {
    uint type;
    vec3 position;
    vec3 ambient;
    float ambientStrength;
    float diffuseStrength;
    float specularStrength;
};

uniform vec3 u_viewPos;
uniform Light u_sunLight;

in vec3 f_position;
in vec3 f_normal;
in vec3 f_color;

out vec4 o_color;

void main() {
    vec3 ambient = u_sunLight.ambientStrength * u_sunLight.ambient;

    vec3 normal = normalize(f_normal);
//	normal = normalize(cross(dFdx(f_position), dFdy(f_position)));
    vec3 lightDir = -u_sunLight.position;
    if (u_sunLight.type != LIGHT_DIRECTIONAL) {
        lightDir = normalize(u_sunLight.position - f_position);
    }
//    float diff = max(dot(normal, lightDir), 0.0);
    float diff = (dot(normal, lightDir) * 0.5 + 0.5) * 0.9;
    vec3 diffuse = u_sunLight.diffuseStrength * diff * u_sunLight.ambient;

    vec3 viewDir = normalize(u_viewPos - f_position);
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32.0);
    vec3 specular = u_sunLight.specularStrength * spec * u_sunLight.ambient;

    vec3 result = (ambient + diffuse + specular) * f_color;
    o_color = vec4(result, 1.0);
}
