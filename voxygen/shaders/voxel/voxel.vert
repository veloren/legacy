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

vec4 get_color(uint col_attr) {
	// extract the second byte
	uint grad = (col_attr >> 8) & 0xFFu;

	// Palette mode
	if ((grad & 0xC0u) == 0x80u) {
		// This mode is just a simple index into a large 256-entry palette
		return col_lut[col_attr & 0xFFu];
	// Double gradient mode
	} else if ((grad & 0xC0u) == 0x40u) {
		// This mode blends between two colors: a, b
		// a and b are blended using the grad value

		// Calculate the a, b colours based on their indices
		vec4 col_a = col_lut[grad2_a_lut[(col_attr >> 0) & 0xFu]];
		vec4 col_b = col_lut[grad2_b_lut[(col_attr >> 4) & 0xFu]];

		return mix(col_a, col_b, float(grad & 0x3Fu) / 64.0);
	// Triple gradient mode
	} else if ((grad & 0xC0u) == 0xC0u) {
		// This mode blends between 3 colors: o, a, b
		// a and b are blended first using the grad_ab value
		// Then, the resulting color (col_ab) is blended with o

		// Calculate the o, a, b colours based on their indices
		vec4 col_o = col_lut[grad3_o_lut[(col_attr >> 0) & 0x1u]];
		vec4 col_a = col_lut[grad3_a_lut[(col_attr >> 1) & 0x1u]];
		vec4 col_b = col_lut[grad3_b_lut[(col_attr >> 2) & 0x1u]];

		vec4 col_ab = mix(col_a, col_b, float((col_attr >> 3) & 0x1Fu) / 32.0);

		return mix(col_o, col_ab, float(grad & 0x3Fu) / 64.0);
	// Fallback
	} else {
		return vec4(1, 1, 1, 1);
	}
}

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
    frag_col = get_color(attr.x);
    frag_ao = float(attr.y);
	frag_norm = norm_lut[attr.z];
	frag_mat = attr.w;

	gl_Position = proj_mat * view_mat * vec4(world_pos, 1);
}
