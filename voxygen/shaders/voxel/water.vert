#version 330 core

#include <luts.glsl>
#include <noise.glsl>

in vec3 vert_pos;
in uint vert_norm_ao_col_mat;

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
    uint norm_id = vert_norm_ao_col_mat & 0xFFu;
    uint ao = (vert_norm_ao_col_mat & 0xFF00u) >> 8;
    uint col_id = (vert_norm_ao_col_mat & 0xFF0000u) >> 16;
    uint mat_id = (vert_norm_ao_col_mat & 0xFF000000u) >> 24;

	vec3 world_pos = (model_mat * vec4(vert_pos, 1)).xyz;

	frag_pos = vert_pos;
	frag_world_pos = world_pos;
    frag_col = col_lut[col_id];
    frag_ao = float(ao);
	frag_norm = norm_lut[norm_id];
	frag_mat = mat_id;

	gl_Position = proj_mat * view_mat * vec4(world_pos, 1);
}
