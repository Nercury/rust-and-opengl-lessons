#version 330 core

layout (location = 0) in vec3 Position;
layout (location = 2) in vec3 Normal;
layout (location = 3) in vec2 Uv;

uniform vec3 CameraPos;
uniform mat4 View;
uniform mat4 Projection;

out VS_OUTPUT {
    vec2 Uv;
    vec3 Normal;
    vec3 CameraPos;
    vec3 Position;
} OUT;

void main()
{
    gl_Position = Projection * View * vec4(Position, 1.0);

    OUT.Uv = Uv;
    OUT.Normal = Normal;
    OUT.CameraPos = CameraPos;
    OUT.Position = Position;
}