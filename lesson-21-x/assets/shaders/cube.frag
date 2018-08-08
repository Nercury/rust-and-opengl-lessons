#version 330 core

uniform sampler2D Texture;
uniform sampler2D Normals;

in VS_OUTPUT {
    vec2 Uv;
    vec3 TangentCameraPos;
    vec3 TangentPosition;
} IN;

out vec4 Color;

void main()
{
    vec3 normal = texture(Normals, IN.Uv).rgb; // obtain normal from normal map in range [0,1]
    normal = normalize(normal * 2.0 - 1.0); // transform normal vector to range [-1,1]

    // get diffuse color
    vec3 color = texture(Texture, IN.Uv).rgb;
    // ambient
    vec3 ambient = 0.3 * color;
    // diffuse
    vec3 lightDir = normalize(IN.TangentCameraPos - IN.TangentPosition);
    float diff = max(dot(lightDir, normal), 0.0);
    vec3 diffuse = diff * color;
    // specular
    vec3 viewDir = normalize(IN.TangentCameraPos - IN.TangentPosition);
    vec3 reflectDir = reflect(-lightDir, normal);
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), 12.0);

    vec3 specular = vec3(0.2) * spec;
    Color = vec4(ambient + diffuse + specular, 1.0);
}