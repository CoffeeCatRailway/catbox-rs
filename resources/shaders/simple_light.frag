#version 330 core

const float LIGHT_DIRECTIONAL = 0.0;
const float LIGHT_POINT = 1.0;

struct Material {
    vec3 color;
    sampler2D diffuse;
    vec4 specular; // a is shininess
};

struct Light {
    vec4 position; // w is type

    vec3 color;
    vec3 strength;

    vec2 attenuation; // intensity, radius
};

uniform vec3 u_viewPos;
uniform Material u_material;
#define MAX_POINT_LIGHTS 8
uniform Light u_lights[MAX_POINT_LIGHTS];

in vec3 f_position;
in vec3 f_normal;
in vec2 f_uv;

out vec4 o_color;

void main() {
    vec3 matDiffuse = texture(u_material.diffuse, f_uv).rgb * u_material.color;
    vec3 normal = normalize(f_normal);
//    normal = normalize(cross(dFdx(f_position), dFdy(f_position)));
    vec3 viewDir = normalize(u_viewPos - f_position);

    vec3 finalCol = vec3(0.0);
    for (int i = 0; i < MAX_POINT_LIGHTS; i++) {
        Light light = u_lights[i];

        vec3 lightDir;
        if (light.position.w == LIGHT_DIRECTIONAL) {
            lightDir = normalize(-light.position.xyz);
        } else {
            lightDir = normalize(light.position.xyz - f_position);
        }

        // diffuse
        float diffuseCoef = max(dot(normal, lightDir), 0.0);

        // specular
        vec3 reflectDir = reflect(-lightDir, normal);
        float specularCoef = pow(max(dot(viewDir, reflectDir), 0.0), u_material.specular.a);

        // combine
        vec3 ambient = light.color * light.strength.x * matDiffuse;
        vec3 diffuse = light.color * light.strength.y * diffuseCoef * matDiffuse;
        vec3 specular = light.color * light.strength.z * specularCoef * u_material.specular.rgb;
        vec3 result = ambient + diffuse + specular;

        if (light.position.w != LIGHT_DIRECTIONAL) {
            // attenuation
            float dist = length(light.position.xyz - f_position);
            float s = min(pow(dist / light.attenuation.y, 2.0), 1.0);
//            float attenuation = light.attenuation.x * pow((1.0 - s) / (1.0 + s), 2.0);
            float attenuation = light.attenuation.x * (1.0 - s) / (1.0 + s);
            result *= attenuation;
        }
        finalCol += result;
	}

	o_color = vec4(finalCol, 1.0);
}
