#version 140

uniform sampler2D tex;

out vec4 col;
in vec2 frag_tex_coord;
in vec4 frag_col;

void main() {
	col = frag_col * texture2D(tex, frag_tex_coord);
}
