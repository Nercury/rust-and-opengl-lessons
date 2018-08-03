#version 330 core

layout (location = 0) in vec3 Position;
layout (location = 1) in vec4 Color;
layout (location = 2) in vec3 T;
layout (location = 3) in vec3 B;
layout (location = 4) in vec3 N;
layout (location = 5) in vec2 Uv;

uniform vec3 CameraPos;
uniform mat4 View;
uniform mat4 Projection;

out VS_OUTPUT {
    vec4 Color;
    vec2 Uv;
    vec3 TangentCameraPos;
    vec3 TangentPosition;
} OUT;

void main()
{
    gl_Position = Projection * View * vec4(Position, 1.0);

    OUT.Color = Color;
    OUT.Uv = Uv;

    mat3 TBN = mat3(T, B, N);
    OUT.TangentCameraPos = TBN * CameraPos;
    OUT.TangentPosition = TBN * Position;
}