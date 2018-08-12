    #version 330 core

layout (location = 0) in vec3 Position;
layout (location = 1) in vec2 Uv;
layout (location = 2) in vec3 T;
layout (location = 3) in vec3 N;

uniform vec3 CameraPos;
uniform mat4 ViewProjection;
uniform mat4 Model;

out VS_OUTPUT {
    vec2 Uv;
    vec3 TangentCameraPos;
    vec3 TangentPosition;
} OUT;

void main()
{
    vec3 WorldPosition = vec3(Model * vec4(Position, 1.0));
    gl_Position = ViewProjection * vec4(WorldPosition, 1.0);

    OUT.Uv = Uv;

    mat3 IntoModelMatrix = transpose(inverse(mat3(Model)));

    vec3 ModelT = normalize(IntoModelMatrix * T);
    vec3 ModelN = normalize(IntoModelMatrix * N);
    ModelT = normalize(ModelT - dot(ModelT, ModelN) * ModelN);
    vec3 ModelB = cross(ModelN, ModelT);

    mat3 TBN = transpose(mat3(ModelT, ModelB, ModelN));
    OUT.TangentCameraPos = TBN * CameraPos;
    OUT.TangentPosition = TBN * WorldPosition;
}