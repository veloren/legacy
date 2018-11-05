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

out vec4 target;

void main() {
	float tod = get_time_of_day(time.x);
	target = vec4(get_skybox(normalize(frag_pos), tod) * 3.0, 1.0);
	// target = vec4(vec3(0.5), 1.0);
}
