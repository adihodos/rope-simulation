#version 450 core

layout (location = 0) in vec2 vsInPos;

out gl_PerVertex {
  vec4 gl_Position;
};

void main(void) {
  gl_Position = vec4(vsInPos, 0.0, 1.0);
}
