#version 330 core

layout (location = 0) in vec3 Position;
layout (location = 1) in vec4 Color;
layout (location = 2) in vec3 Normal;
layout (location = 3) in vec3 Tn;
layout (location = 4) in vec2 Uv;

uniform vec3 CameraPos;
uniform mat4 View;
uniform mat4 Projection;

out VS_OUTPUT {
    vec4 Color;
    vec3 Position;
    vec2 Uv;
    vec3 TangentCameraPos;
    vec3 TangentPosition;
} OUT;

void main()
{
    OUT.Position = Position;
    OUT.Color = Color;
    OUT.Uv = Uv;

    vec3 T = normalize(Tn);
    vec3 N = normalize(Normal);
    vec3 B = cross(N, T);

    mat3 TBN = transpose(mat3(T, B, N));
    OUT.TangentCameraPos  = TBN * CameraPos;
    OUT.TangentPosition  = TBN * Position;

    gl_Position = Projection * View * vec4(Position, 1.0);
}