#version 330 core

#include <noise.glsl>
#include <luts.glsl>

in vec3 vert_pos;
in uint vert_attrib;

layout (std140)
uniform model_consts {
	mat4 model_mat;
};

layout (std140)
uniform global_consts {
	mat4 view_mat;
	mat4 proj_mat;
	vec4 cam_origin;
	vec4 play_origin;
	vec4 view_distance;
	vec4 time;
};

out vec3 frag_pos;
out vec3 frag_world_pos;
out vec4 frag_col;
out float frag_ao;
flat out vec3 frag_norm;
flat out uint frag_mat;

void main() {
	// This is kind of ugly, but hey - parallel code!
	uvec4 attr = (uvec4(vert_attrib) >> uvec4(
		0,
		16,
		20,
		24
	)) & uvec4(
		0xFFFFu,
		0x0F,
		0x0F,
		0xFF
	);

	vec3 world_pos = (model_mat * vec4(vert_pos, 1)).xyz;

	frag_pos = vert_pos;
	frag_world_pos = world_pos;
    frag_col = col_lut[attr.x & 0xFFu];
    frag_ao = float(attr.y);
	frag_norm = norm_lut[attr.z];
	frag_mat = attr.w;

	gl_Position = proj_mat * view_mat * vec4(world_pos, 1);
}
