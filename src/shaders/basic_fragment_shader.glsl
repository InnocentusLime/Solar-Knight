#version 140

uniform sampler2D tex;

out vec4 col;
in vec2 frag_tex_coord;

void main() {
	col = texture2D(tex, frag_tex_coord);
}
