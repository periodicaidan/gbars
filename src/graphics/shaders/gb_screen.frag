#ifdef GL_ES
precision mediump float;
#endif

in vec2 TexCoord;

out vec4 fragColor;

uniform sampler2D tex;

void main() {
    fragColor = texture(tex, TexCoord);
}
