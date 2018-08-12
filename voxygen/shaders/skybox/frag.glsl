#version 330 core

#include <noise.glsl>
#include <sky.glsl>

in vec3 frag_pos;

layout (std140)
uniform global_consts {
	mat4 view_mat;
	mat4 proj_mat;
	vec4 cam_origin;
	vec4 play_origin;
	vec4 view_distance;
	vec4 time;
};

out vec4 target;

void main() {
	target = vec4(get_sky_chroma(frag_pos, time.x), 1);
}
