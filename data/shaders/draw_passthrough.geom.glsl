#version 450 core

layout (lines) in;
layout (line_strip, max_vertices = 2) out;

in gl_PerVertex {
  vec4 gl_Position;
} gl_in[];

out gl_PerVertex {
  vec4 gl_Position;
};

layout (location = 0) uniform mat4 WorldViewProj;

void main(void) {
  for (int i = 0; i < 2; ++i) {
    gl_Position = WorldViewProj * gl_in[i].gl_Position;
    EmitVertex();
  }
  EndPrimitive();
}
