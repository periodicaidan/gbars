#ifdef GL_ES
precision mediump float;
#endif

layout (location=0) in vec2 pos;
layout (location=1) in vec2 tex;

out vec2 TexCoord;

void main() {
    gl_Position = vec4(pos, 0.0, 1.0);
    TexCoord = tex;
}
