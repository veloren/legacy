#version 330 core

#include <noise.glsl>
#include <common.glsl>
#include <luts.glsl>
#include <sky.glsl>
#include <bsdf.glsl>
#include <luts.glsl>

in vec3 frag_pos;
in vec3 frag_world_pos;
//in vec4 frag_col;
in float frag_ao;
flat in vec3 frag_norm;
flat in uint frag_mat;
flat in uint frag_col_attr;

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

out vec4 target;

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

float diffuse_factor = 0.5;
float ambiant_factor = 0.2;
vec3  sun_direction = normalize(vec3(-1.5, -0.8, -1));
vec3  sun_colorr     = vec3(1, 1, 1);
float sun_factor    = 50;
float sun_shine     = 0;


void main() {
	vec4 frag_col = get_color(frag_col_attr);

	Material mat = mat_lut[frag_mat];
	// Sunlight
	float sunAngularRadius = 0.017; // 1 degree radius, 2 degree diameter (not realistic, irl sun is ~0.5 deg diameter)
	float time_of_day = get_time_of_day(time.x);
	vec3 sun_color = get_sun_color(time_of_day);
	vec3 sun_dir = get_sun_dir(time_of_day);

	// Geometry
	vec3 N = normalize((model_mat * vec4(frag_norm, 0)).xyz);
	vec3 V = normalize(cam_origin.xyz - frag_world_pos);

	// calculate closest direction on sun's disk to reflection vector
	float r = sin(sunAngularRadius);
	float d = cos(sunAngularRadius);
	vec3 R = reflect(-V, N);
	float DdotR = dot(sun_dir, R);
	vec3 S = R - DdotR * sun_dir;
	vec3 L = DdotR < d ? normalize(d * sun_dir + normalize(S) * r) : R;

	float NdotV = abs(dot(N, V));
	float NdotL = saturate(dot(N, L));
	vec3 H = normalize(V + L);
	float LdotH = saturate(dot(L, H));
	float NdotH = clamp(dot(N, H), 0.0, 0.99999995);// fix artifact

	vec3 atmos_color = get_sky(N, time_of_day, false);

	vec3 col_noise = vec3(0,0,0);

	// TODO: Figure out a way to do this more efficiently
	// if (mat.color_variance > 0.0) {
	// 	vec3 noise_in = frag_world_pos * 0.01 * mat.color_variance_scale;
	// 	col_noise = vec3(
	// 		snoise(noise_in + vec3(100, 0, 0)),
	// 		snoise(noise_in + vec3(0, 100, 0)),
	// 		snoise(noise_in + vec3(0, 0, 100))
	// 	) * mat.color_variance;
	// }

	vec3 col = frag_col.rgb;// + col_noise;

	float smoothness = mat.smoothness;
	float roughness_linear = saturate(1 - (smoothness - 0.01));
	float roughness = roughness_linear * roughness_linear;

	float reflectance = mat.reflectance;
	float metalness = mat.metalness;
	float omm = 1.0 - mat.metalness;
	vec3 f0 = mix(vec3(mix(0.02, 0.18, reflectance)), col.rgb, metalness);
	float f90 = 1.0;
	vec3 fresnel = f_Schlick(f0, f90, LdotH);
	float geo = vis_SmithGGXCorrelated(NdotL, NdotV, roughness);
	float ndf = ndf_GGX(NdotH, roughness);
	vec3 specular = fresnel * ndf * geo / PI;

	float fD = fr_DisneyDiffuse(NdotV, NdotL, LdotH, roughness_linear) / PI;
	float ao = (frag_ao / 3.0);
	vec3 diffuse = fD * col.rgb * omm * ao;

	float sun_level = saturate(day_cycle(1, 0.9, time_of_day));
	float sun_intensity = sun_level * 80000;
	vec3 sun_illuminance = sun_color * sun_intensity;

    float ambient_intensity = 2.0 * omm; // TODO: have specular ambient so that we don't have to hack this
	vec3 ambient = col.rgb * ambient_intensity * atmos_color;

	vec3 lighted = ambient * ao + (saturate((diffuse + specular) * NdotL) * sun_illuminance * ao);
	//vec3 lighted = ambient + ((diffuse + specular) * sun_illuminance) * ao;

	// Mist
	float mist_start = view_distance.x * 0.9;// + snoise(vec4(world_pos, time) * 0.02) * 50.0;
	float mist_end = view_distance.x;// + snoise(vec4(world_pos, -time) * 0.02) * 50.0;
	float mist_delta = mist_end - mist_start;
	float play_dist = length(play_origin.xyz - frag_world_pos.xyz);
	float dist = max(play_dist - mist_start, 0);
	float percent = saturate(dist / mist_delta);
	float mist_value = percent;

	vec3 sky_chroma = get_sky_chroma(-V, time_of_day);
    float smax = max(specular.r, max(specular.g, specular.b));
    float a = clamp(smax + frag_col.a, 0, 1);
	target = mix(vec4(lighted, a), vec4(sky_chroma, 1.0), mist_value);
}
