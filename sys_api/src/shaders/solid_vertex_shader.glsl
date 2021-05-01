#version 140

in vec2 pos;

out vec4 frag_col;

uniform vec4 col;
uniform mat4 mvp;

void main() {
    gl_Position = (mvp * vec4(pos, 0.0, 1.0));
    frag_col = col;
}
