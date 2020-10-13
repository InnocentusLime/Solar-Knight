#version 140

in vec2 pos;
in vec2 tex_coord;

out vec2 frag_tex_coord;

uniform vec2 scale;
uniform mat4 mvp;

void main() {
    gl_Position = (mvp * vec4(pos * scale, 0.0, 1.0));
    frag_tex_coord = tex_coord;
}
