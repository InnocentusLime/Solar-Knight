#version 140

in vec2 pos;
in vec2 tex_coord;

in vec4 mat_col1;
in vec4 mat_col2;
in vec4 mat_col3;
in vec4 mat_col4;
in vec2 texture_bottom_left;
in vec2 width_height;
in vec4 color;

out vec2 frag_tex_coord;
out vec4 frag_col;

uniform mat4 vp;

void main() {
    mat4 mvp = vp * mat4(mat_col1, mat_col2, mat_col3, mat_col4);

    gl_Position = mvp * vec4(pos.xy, 0.0, 1.0);
    frag_tex_coord = tex_coord * width_height + texture_bottom_left;
    frag_col = color;
}
