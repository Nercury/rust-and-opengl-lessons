#version 330 core

layout (location = 0) in vec3 Position;
layout (location = 1) in vec4 Color;

uniform mat4 ViewProjection;
uniform mat4 Model;

out VS_OUTPUT {
    vec4 Color;
} OUT;

void main()
{
    gl_Position = ViewProjection * Model * vec4(Position, 1.0);
    OUT.Color = Color;
}