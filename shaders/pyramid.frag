#version 330 core

out vec4 Color;
in vec4 v_colour;

void main() {
  Color = v_colour;
}