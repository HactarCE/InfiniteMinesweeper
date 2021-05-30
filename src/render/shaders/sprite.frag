#version 140

in vec2 uv;

uniform sampler2D spritesheet;

void main() {
    gl_FragColor = texture(spritesheet, uv);
}
