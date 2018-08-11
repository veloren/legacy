#version 330 core

#include <noise.glsl>
#include <sky.glsl>
#include <bsdf.glsl>

#define PI 3.14159265

in vec3 frag_pos;
in vec3 frag_norm;
in vec4 frag_col;

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

void main() {
	target = frag_col;

	// Sunlight
	float diffuse_factor = 0.9;
	float ambient_intensity = 0.075;
	float sunAngularRadius = 0.017 + day_anticycle(2, 1.0, time.x) * 0.005; // 1 degree, 2 degree diameter
	vec3 sun_color = vec3(day_anticycle(2, 0.1, time.x) + 0.1, day_cycle(2, 0.3, time.x) + 0.3, day_cycle(2, 0.4, time.x) + 0.4);

	// Geometry
	vec3 N = normalize((model_mat * vec4(frag_norm, 0)).xyz);
	vec3 world_pos = (model_mat * vec4(frag_pos, 1)).xyz;
	vec3 V = normalize(cam_origin.xyz - world_pos);
	float play_dist = length(play_origin.xy - world_pos.xy);

	// Sunlight
	vec3 sun_dir = get_sun_dir(time.x);
	vec3 sky_chroma = get_sky_chroma(world_pos - cam_origin.xyz, time.x);

	// calculate closest direction on sun's disk to reflection vector
	float r = sin(sunAngularRadius);
	float d = cos(sunAngularRadius);
	vec3 R = reflect(V, N);
	float DdotR = dot(sun_dir, R);
	vec3 S = R - DdotR * sun_dir;
	vec3 L = DdotR < d ? normalize(d * sun_dir + normalize(S) * r) : R;

	// precalculations
	float NdotV = abs(dot(N, V)) + 0.000001;
	float NdotL = clamp(dot(N, L), 0.0, 1.0);
	vec3 H = normalize(V + L);
	float LdotH = clamp(dot(L, H), 0.0, 1.0);
	float NdotH = clamp(dot(N, H), 0.0, 1.0);

	float ambient_level = clamp(day_cycle(1.0, 1.0, time.x), 0.1, 1);
	vec3 ambient = frag_col.rgb * ambient_intensity * sun_color;

	float noise_factor = 0.8;
	bool is_grass = frag_col.g > frag_col.r && frag_col.g > frag_col.b;
	float wetness = snoise(frag_pos.xy / 5.0) * noise_factor + 1 - noise_factor;
	float roughness_linear = is_grass ? wetness : 0.8;
	float roughness = roughness_linear * roughness_linear;

	float metallic = 0.0;
	float reflectance = is_grass ? 0.2 + wetness : 0.5;
	vec3 f0 = mix(vec3(mix(0.02, 0.18, reflectance)), frag_col.rgb, metallic);
	float f90 = 1.0;
	vec3 fresnel = f_Schlick(f0, f90, LdotH);
	float vis = vis_SmithGGXCorrelated(NdotL, NdotV, roughness);
	float norm_dist = ndf_GGX(NdotH, roughness);
	vec3 specular = fresnel * norm_dist * vis / PI;

	float fD = fr_DisneyDiffuse(NdotV, NdotL, LdotH, roughness_linear) / PI;
	vec3 diffuse = fD * frag_col.rgb;

	float sun_level = clamp(day_cycle(1, 0.9, time.x), 0.0, 1);
	float sun_intensity = sun_level * 2.5;
	float sun_illuminance = sun_intensity * NdotL;

	vec3 lighted = ambient + ((diffuse + specular) * sun_color * sun_illuminance);

	// Mist
	float mist_start = view_distance.x * 0.8;// + snoise(vec4(world_pos, time) * 0.02) * 50.0;
	float mist_end = view_distance.x;// + snoise(vec4(world_pos, -time) * 0.02) * 50.0;
	float mist_delta = 1 / (mist_end - mist_start);
	float mist_value = clamp(play_dist * mist_delta - mist_delta * mist_start, 0.0, 1.0);

	target = mix(vec4(lighted, frag_col.w), vec4(sky_chroma, 1), mist_value);
}
