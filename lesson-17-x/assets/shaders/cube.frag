#version 330 core

uniform sampler2D TexFace;
uniform sampler2D TexNormal;
uniform sampler2D TexSpecular;

in VS_OUTPUT {
    vec4 Color;
    vec3 Position;
    vec2 Uv;
    vec3 TangentCameraPos;
    vec3 TangentPosition;
} IN;

out vec4 Color;

void main()
{
    vec3 specColor = texture(TexSpecular, IN.Uv).rgb;

    vec3 color = texture(TexFace, IN.Uv).rgb;

    vec3 normal = texture(TexNormal, IN.Uv).rgb; // obtain normal from normal map in range [0,1]
    normal = normalize(normal * 2.0 - 1.0); // transform normal vector to range [-1,1]

    // diffuse
    vec3 lightDir = normalize(IN.TangentCameraPos - IN.TangentPosition);
    float diff = max(dot(lightDir, normal), 0.0);
    vec3 diffuse = diff * color;

    // specular
    vec3 viewDir = normalize(IN.TangentCameraPos - IN.TangentPosition);
    vec3 reflectDir = reflect(-lightDir, normal);
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), 32.0);

    vec3 specular = specColor * spec;
    Color = vec4(diffuse + specular, 1.0);
}