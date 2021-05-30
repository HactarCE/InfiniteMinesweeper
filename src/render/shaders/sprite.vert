#version 140

in vec2 pos;
in ivec2 tile_coords;
in uvec2 sprite_coords;

uniform sampler2D spritesheet;

uniform ivec2 camera_center;
uniform mat4 transform;

out vec2 uv;

const float SPRITE_SIZE = 64.0;

void main() {
    gl_Position = transform * vec4(pos + vec2(tile_coords - camera_center), 0.0, 1.0);
    uv = (SPRITE_SIZE * (sprite_coords + pos)) / vec2(textureSize(spritesheet, 0));
}
