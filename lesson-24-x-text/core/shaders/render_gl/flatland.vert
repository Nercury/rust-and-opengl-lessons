#version 330 core

layout (location = 0) in vec2 Position;
layout (location = 1) in vec2 Normal;
layout (location = 2) in float OffsetX;
layout (location = 3) in float OffsetY;
layout (location = 4) in vec4 ModelCol0;
layout (location = 5) in vec4 ModelCol1;
layout (location = 6) in vec4 ModelCol2;
layout (location = 7) in vec4 ModelCol3;
layout (location = 8) in vec4 Color;

uniform mat4 ViewProjection;

out VS_OUTPUT {
    vec4 Color;
} OUT;

void main()
{
    mat4 Model;
    Model[0] = ModelCol0;
    Model[1] = ModelCol1;
    Model[2] = ModelCol2;
    Model[3] = ModelCol3;

    vec4 OutPos = ViewProjection * Model * vec4(Position.x + OffsetX, Position.y + OffsetY, 0.0, 1.0);
    gl_Position = vec4(OutPos.x, OutPos.y, OutPos.z, OutPos.w);
    OUT.Color = Color;
}