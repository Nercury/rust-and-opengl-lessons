#version 330 core

layout (location = 0) in vec2 Position;
layout (location = 1) in vec2 Normal;

uniform mat4 ViewProjection;
uniform mat4 Model;
uniform vec4 Color;

out VS_OUTPUT {
    vec4 Color;
} OUT;

void main()
{
    gl_Position = ViewProjection * Model * vec4(Position, 0.0, 1.0);
    OUT.Color = Color;
}