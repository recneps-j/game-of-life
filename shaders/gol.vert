#version 330 core

layout (location = 0) in vec2 Position;

uniform mat4 u_transform;

void main() {
  gl_Position =  vec4(Position, 0.0, 1.0);
}