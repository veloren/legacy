#version 330 core

#include <noise.glsl>

in vec3 frag_pos;

layout (std140)
uniform global_consts {
	mat4 view_mat;
	mat4 proj_mat;
	vec4 sky_color;
	vec4 play_origin;
	vec4 view_distance;
	vec4 time;
};

out vec4 target;

void main() {
	vec3 sun_dir = normalize(vec3(-1.0, -1.0, -1.0));
	vec3 sun_color = vec3(1.5, 1.5, 1.0);
	float sun_size = 0.02;
	float sun_strength = 50;
	float sun_bloom = 5;

	vec3 dir = normalize(frag_pos.xyz);

	float angle = dot(-sun_dir, dir);
	float factor = (pow(angle, 1 / sun_bloom) -1 + sun_size) * sun_strength;

	vec3 col = mix(sky_color.xyz, sun_color, clamp(factor, 0, 1));

	target = vec4(col, 1.0);
}
