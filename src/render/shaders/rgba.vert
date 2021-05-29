#version 140

in vec2 pos;
in ivec2 tile_coords;
in uvec2 sprite_coords;

uniform sampler2D spritesheet;
uniform uvec2 sprite_size;

uniform ivec2 camera_center;
uniform mat4 transform;

out vec2 uv;

void main() {
    gl_Position = transform * vec4(pos + vec2(tile_coords - camera_center), 0.0, 1.0);
    uv = (vec2(sprite_coords) + pos * vec2(sprite_size)) / vec2(textureSize(spritesheet, 0));
}
