#version 330 core

layout (location = 0) in vec3 Position;
layout (location = 1) in vec4 Color;
layout (location = 2) in vec3 Normal;

uniform mat4 ViewProjection;

out VS_OUTPUT {
    vec4 Color;
    vec3 Normal;
    vec3 Position;
} OUT;

void main()
{
    gl_Position = ViewProjection * vec4(Position, 1.0);
    OUT.Position = Position;
    OUT.Color = Color;
    OUT.Normal = Normal;
}