#version 330 core

const float LIGHT_DIRECTIONAL = 0.0;
const float LIGHT_POINT = 1.0;

struct Material {
    vec3 color;
    sampler2D diffuse;
    vec4 specular; // w is shininess
};

struct Light {
    vec4 position; // w is type
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

uniform vec3 u_viewPos;
uniform Light u_sunLight;
uniform Material u_material;

in vec3 f_position;
in vec3 f_normal;
in vec2 f_uv;

out vec4 o_color;

void main() {
    vec3 matDiffuse = texture(u_material.diffuse, f_uv).rgb * u_material.color;

    // ambient
    vec3 ambient = u_sunLight.ambient * matDiffuse;

    // diffuese
    vec3 normal = normalize(f_normal);
//	normal = normalize(cross(dFdx(f_position), dFdy(f_position)));

    vec3 lightDir = vec3(0.0, 1.0, 0.0);
    if (u_sunLight.position.w == LIGHT_DIRECTIONAL) {
        lightDir = normalize(-u_sunLight.position.xyz);
    } else {
        lightDir = normalize(u_sunLight.position.xyz - f_position);
    }

    float diff = max(dot(normal, lightDir), 0.0); // 0-1 clamped
//    float diff = dot(normal, lightDir) * 0.5 + 0.5; // 0-1
//    float diff = dot(normal, lightDir) * 0.425 + 0.475; // 0.05-0.9
    vec3 diffuse = u_sunLight.diffuse * diff * matDiffuse;

    // specular
    vec3 viewDir = normalize(u_viewPos - f_position);
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), u_material.specular.w);
    vec3 specular = u_sunLight.specular * spec * u_material.specular.rgb;

    o_color = vec4(ambient + diffuse + specular, 1.0);
}
