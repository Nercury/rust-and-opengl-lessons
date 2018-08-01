#version 330 core

uniform vec3 CameraPos;

in VS_OUTPUT {
    vec4 Color;
    vec3 Normal;
    vec3 Position;
} IN;

out vec4 Color;

void main()
{
    float highlight = dot(normalize(CameraPos - IN.Position), IN.Normal);
    Color = IN.Color * clamp(highlight, 0, 1);
}