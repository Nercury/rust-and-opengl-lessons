#version 330 core

uniform sampler2D Texture;

in VS_OUTPUT {
    vec2 Uv;
    vec3 Normal;
    vec3 CameraPos;
    vec3 Position;
} IN;

out vec4 Color;

void main()
{
    vec3 LightPos = IN.CameraPos;

    vec3 color = texture(Texture, IN.Uv).rgb;

    // normal
    vec3 normal = IN.Normal;
    // ambient
    vec3 ambient = 0.3 * color;
    // diffuse
    vec3 lightDir = normalize(LightPos - IN.Position);
    float diff = max(dot(lightDir, normal), 0.0);
    vec3 diffuse = diff * color;

    // specular
    vec3 viewDir = normalize(IN.CameraPos - IN.Position);
    vec3 reflectDir = reflect(-lightDir, normal);
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), 12.0);

    vec3 specular = vec3(0.2) * spec;
    Color = vec4(ambient + diffuse + specular, 1.0);
}