#version 330 core

layout (location = 0) in vec3 Position;
out vec4 v_colour;

uniform mat4 u_transform;

void main() {
  vec4 fg_col = vec4(1.0f, 0.5f, 0.2f, 1.0f);
  vec4 bg_col = vec4(0.4f, 0.2f, 0.1f, 1.0f);
  v_colour = mix(fg_col, bg_col, Position.z * 2.0);
  gl_Position = u_transform * vec4(Position, 1.0);
}