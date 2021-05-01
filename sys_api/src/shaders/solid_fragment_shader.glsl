#version 140

out vec4 col;
in vec4 frag_col;

void main() {
	col = frag_col;
}
