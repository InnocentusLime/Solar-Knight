#version 140

in vec2 pos;
in vec2 tex_coord;

in vec4 mat_col1;
in vec4 mat_col2;
in vec4 mat_col3;
in vec4 mat_col4;

out vec2 frag_tex_coord;

uniform vec2 scale;
uniform mat4 vp;

void main() {
    mat4 m = mat4(mat_col1, mat_col2, mat_col3, mat_col4);
    mat4 mvp = vp * m;
    gl_Position = (mvp * vec4(pos * scale, 0.0, 1.0));
    frag_tex_coord = tex_coord;
}
