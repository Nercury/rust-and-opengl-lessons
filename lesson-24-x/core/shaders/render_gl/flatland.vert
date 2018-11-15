#version 330 core

layout (location = 0) in vec2 Position;
layout (location = 1) in vec2 Normal;
layout (location = 2) in float OffsetX;
layout (location = 3) in float OffsetY;

uniform mat4 ViewProjection;
uniform mat4 Model;
uniform vec4 Color;

out VS_OUTPUT {
    vec4 Color;
} OUT;

void main()
{
    vec4 OutPos = ViewProjection * Model * vec4(Position.x + OffsetX, Position.y + OffsetY, 0.0, 1.0);
    gl_Position = vec4(OutPos.x, OutPos.y, OutPos.z, OutPos.w);
    OUT.Color = Color;
}