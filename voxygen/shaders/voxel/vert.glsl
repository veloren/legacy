#version 330 core

in vec3 vert_pos;
in vec3 vert_norm;
in vec4 vert_col;

layout (std140)
uniform model_consts {
	mat4 model_mat;
};

layout (std140)
uniform global_consts {
	mat4 view_mat;
	mat4 proj_mat;
	vec4 play_origin;
	vec4 view_distance;
	vec4 time;
};

out vec3 frag_pos;
out vec3 frag_norm;
out vec4 frag_col;

void main() {
	frag_pos = vert_pos;
	frag_norm = vert_norm;
	frag_col = vert_col;

	gl_Position = proj_mat * view_mat * model_mat * vec4(vert_pos, 1);
}
