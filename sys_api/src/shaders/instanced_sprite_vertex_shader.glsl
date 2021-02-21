#version 140

in vec2 pos;
in vec2 tex_coord;

in vec4 mat_col1;
in vec4 mat_col2;
in vec4 mat_col3;
in vec4 mat_col4;
in vec2 texture_bottom_left;
in vec2 texture_top_right;

out vec2 frag_tex_coord;

uniform mat4 vp;

void main() {
    mat4 m = mat4(mat_col1, mat_col2, mat_col3, mat_col4);
    mat4 mvp = vp * m;
    vec2 delta = texture_top_right - texture_bottom_left;

    gl_Position = (mvp * vec4(pos, 0.0, 1.0));
    frag_tex_coord = texture_bottom_left + delta * tex_coord;
}
