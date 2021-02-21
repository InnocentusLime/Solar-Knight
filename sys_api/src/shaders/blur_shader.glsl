#version 140

// Fast bluring algorithm
// source : https://github.com/Jam3/glsl-fast-gaussian-blur/blob/master/9.glsl
// explanation : http://rastergrid.com/blog/2010/09/efficient-gaussian-blur-with-linear-sampling/
vec4 blur9(sampler2D image, vec2 uv, vec2 resolution, vec2 direction) {
    vec4 color = vec4(0.0);
    vec2 off1 = vec2(1.3846153846) * direction;
    vec2 off2 = vec2(3.2307692308) * direction;
    color += texture2D(image, uv) * 0.2270270270;
    color += texture2D(image, uv + (off1 / resolution)) * 0.3162162162;
    color += texture2D(image, uv - (off1 / resolution)) * 0.3162162162;
    color += texture2D(image, uv + (off2 / resolution)) * 0.0702702703;
    color += texture2D(image, uv - (off2 / resolution)) * 0.0702702703;
    return color;
}

uniform vec2 resolution;
uniform vec2 direction;
uniform sampler2D tex;

out vec4 col;

void main() {
	vec2 uv = gl_FragCoord.xy / resolution; 
    col = blur9(tex, uv, resolution, direction);
}