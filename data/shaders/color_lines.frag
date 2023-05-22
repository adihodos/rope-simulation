#version 450 core

layout (location = 0) uniform vec4 LineColor;
layout (location = 0) out vec4 FinalFragColor;

void main(void) {
  FinalFragColor = LineColor;
}