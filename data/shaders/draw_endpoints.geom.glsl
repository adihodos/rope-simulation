// version directive is inserted from the application

layout (location = 0) uniform mat4 TransformMatrix;
layout (location = 1) uniform float LineLength;

in gl_PerVertex {
  vec4 gl_Position;
} gl_in[];

out gl_PerVertex {
  vec4 gl_Position;
};

vec2 perp_vector(in const vec2 v) {
  return vec2(-v.y, v.x);
}

#if defined(END_CAPS_TRIANGLES)

layout (lines) in;
layout (triangle_strip, max_vertices = 6) out;
void main(void) {
  const vec2 v0 = gl_in[0].gl_Position.xy;
  const vec2 v1 = gl_in[1].gl_Position.xy;

  const vec2 d = normalize(v1 - v0);
  const vec2 p = perp_vector(d);

  gl_Position = TransformMatrix * vec4(v0 + p * LineLength * 0.5, 0.0, 1.0);
  EmitVertex();

  gl_Position = TransformMatrix * vec4(v0 - d * LineLength, 0.0, 1.0);
  EmitVertex();

  gl_Position = TransformMatrix * vec4(v0 - p * LineLength * 0.5, 0.0, 1.0);
  EmitVertex();
  EndPrimitive();

  gl_Position = TransformMatrix * vec4(v1 - p * LineLength * 0.5, 0.0, 1.0);
  EmitVertex();

  gl_Position = TransformMatrix * vec4(v1 + d * LineLength, 0.0, 1.0);
  EmitVertex();

  gl_Position = TransformMatrix * vec4(v1 + p * LineLength * 0.5, 0.0, 1.0);
  EmitVertex();
  EndPrimitive();
}

#elif defined(END_CAPS_SQUARED)

layout (lines) in;
layout (triangle_strip, max_vertices = 8) out;
void main(void) {
  const vec2 v0 = gl_in[0].gl_Position.xy;
  const vec2 v1 = gl_in[1].gl_Position.xy;

  const vec2 d = normalize(v1 - v0);
  const vec2 p = perp_vector(d);

  // Square #0
  {
    const vec2 a = v0 - p * LineLength * 0.5;
    const vec2 b = v0 + p * LineLength * 0.5;
    const vec2 c = b - d * LineLength;
    const vec2 d = a - d * LineLength;

    gl_Position = TransformMatrix * vec4(a, 0.0, 1.0);
    EmitVertex();
    gl_Position = TransformMatrix * vec4(b, 0.0, 1.0);
    EmitVertex();
    gl_Position = TransformMatrix * vec4(c, 0.0, 1.0);
    EmitVertex();
    gl_Position = TransformMatrix * vec4(d, 0.0, 1.0);
    EmitVertex();
    EndPrimitive();
  }

  // Square #1
  {
    const vec2 a = v1 - p * LineLength * 0.5;
    const vec2 b = a + d * LineLength;
    const vec2 c = b + p * LineLength;
    const vec2 d = c - d * LineLength;

    gl_Position = TransformMatrix * vec4(a, 0.0, 1.0);
    EmitVertex();
    gl_Position = TransformMatrix * vec4(b, 0.0, 1.0);
    EmitVertex();
    gl_Position = TransformMatrix * vec4(c, 0.0, 1.0);
    EmitVertex();
    gl_Position = TransformMatrix * vec4(d, 0.0, 1.0);
    EmitVertex();
    EndPrimitive();
  }
}

#elif defined(END_CAPS_CIRCLE)

const uint CIRCLE_SEGMENTS = 16;

const vec2 CIRCLE_VERTICES[CIRCLE_SEGMENTS + 1 + 1] = {
  vec2(0, 0),
  vec2(1, 0),
  vec2(0.92388, 0.382683),
  vec2(0.707107, 0.707107),
  vec2(0.382683, 0.92388),
  vec2(-4.37114e-08, 1),
  vec2(-0.382684, 0.92388),
  vec2(-0.707107, 0.707107),
  vec2(-0.92388, 0.382683),
  vec2(-1, -8.74228e-08),
  vec2(-0.92388, -0.382683),
  vec2(-0.707107, -0.707107),
  vec2(-0.382683, -0.92388),
  vec2(1.19249e-08, -1),
  vec2(0.382684, -0.923879),
  vec2(0.707107, -0.707107),
  vec2(0.92388, -0.382683),
  vec2(1, 0)
};

layout (points) in;
layout (triangle_strip, max_vertices = 48) out;
void main(void) {
  const vec2 C = gl_in[0].gl_Position.xy;

  for (uint i = 0; i < CIRCLE_SEGMENTS; ++i) {
    const vec2 p0 = C + LineLength * CIRCLE_VERTICES[i + 1];
    const vec2 p1 = C + LineLength * CIRCLE_VERTICES[i + 2];

    gl_Position = TransformMatrix * vec4(C, 0.0, 1.0);
    EmitVertex();
    gl_Position = TransformMatrix * vec4(p0, 0.0, 1.0);
    EmitVertex();
    gl_Position = TransformMatrix * vec4(p1, 0.0, 1.0);
    EmitVertex();
    EndPrimitive();
  }
}

#else

// simple pass-though 
layout (lines) in;
layout (line_strip, max_vertices = 2) out;

void main(void) {
  for (int i = 0; i < gl_in.length(); ++i) {
    gl_Position = TransformMatrix * gl_in[i].gl_Position;
    EmitVertex();
  }
  EndPrimitive();
}

#endif
