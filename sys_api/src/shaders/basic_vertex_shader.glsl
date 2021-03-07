#version 140

in vec2 pos;
in vec2 tex_coord;

out vec2 frag_tex_coord;
out vec4 frag_col;

uniform mat4 mvp;

void main() {
    gl_Position = (mvp * vec4(pos, 0.0, 1.0));
    frag_tex_coord = tex_coord;
    frag_col = vec4(1.0, 1.0, 1.0, 1.0);
}
