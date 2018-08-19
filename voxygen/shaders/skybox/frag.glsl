#version 330 core

#include <common.glsl>
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

out vec3 target;

void main() {
	float tod = get_time_of_day(time.x);
	target = get_skybox(normalize(frag_pos), tod);
}
