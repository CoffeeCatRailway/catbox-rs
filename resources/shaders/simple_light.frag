#version 330 core

const uint LIGHT_DIRECTIONAL = 0u;
const uint LIGHT_POINT = 1u;

struct Material {
    sampler2D diffuse;
    sampler2D specular;
    float shininess;
};

struct Light {
    uint type;
    vec3 position;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

uniform vec3 u_viewPos;
uniform Light u_sunLight;
uniform Material u_material;

in vec3 f_position;
in vec3 f_normal;
in vec3 f_color;
in vec2 f_uv;

out vec4 o_color;

void main() {
    // ambient
    vec3 ambient = u_sunLight.ambient * texture(u_material.diffuse, f_uv).rgb;

    // diffuese
    vec3 normal = normalize(f_normal);
//	normal = normalize(cross(dFdx(f_position), dFdy(f_position)));
    vec3 lightDir = -u_sunLight.position;
    if (u_sunLight.type != LIGHT_DIRECTIONAL) {
        lightDir = u_sunLight.position - f_position;
    }
    lightDir = normalize(lightDir);
//    float diff = max(dot(normal, lightDir), 0.0); // 0-1 clamped
    float diff = dot(normal, lightDir) * 0.5 + 0.5; // 0-1
//    float diff = dot(normal, lightDir) * 0.425 + 0.475; // 0.05-0.9
    vec3 diffuse = u_sunLight.diffuse * diff * texture(u_material.diffuse, f_uv).rgb;

    // specular
    vec3 viewDir = normalize(u_viewPos - f_position);
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), u_material.shininess);
    vec3 specular = u_sunLight.specular * spec * texture(u_material.specular, f_uv).rgb;

    vec3 result = (ambient + diffuse + specular) * f_color;
    o_color = vec4(result, 1.0);
}
