#version 330 core

#include <noise.glsl>
#include <sky.glsl>
#include <bsdf.glsl>

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

out vec3 target;

void main() {
	// Sunlight
	float sunAngularRadius = 0.017; // 1 degree radius, 2 degree diameter (not realistic, irl sun is ~0.5 deg diameter)
	vec3 sun_color = get_sun_color(time.x);
	vec3 sun_dir = -get_sun_dir(time.x);

	// Geometry
	vec3 N = normalize((model_mat * vec4(frag_norm, 0)).xyz);
	vec3 world_pos = (model_mat * vec4(frag_pos, 1)).xyz;
	vec3 V = normalize(cam_origin.xyz - world_pos);

	// calculate closest direction on sun's disk to reflection vector
	float r = sin(sunAngularRadius);
	float d = cos(sunAngularRadius);
	vec3 R = reflect(-V, N);
	float DdotR = dot(sun_dir, R);
	vec3 S = R - DdotR * sun_dir;
	vec3 L = DdotR < d ? normalize(d * sun_dir + normalize(S) * r) : R;

	float NdotV = clamp(dot(N, V), 0.0, 1.0);
	float NdotL = clamp(dot(N, L), 0.0, 1.0);
	vec3 H = normalize(V + L);
	float LdotH = clamp(dot(L, H), 0.0, 1.0);
	float NdotH = clamp(dot(N, H), 0.0, 0.99999995);// fix artifact

	vec3 sky_chroma = get_sky_chroma(-V, time.x);
	vec3 atmos_color = get_sky_chroma(N, time.x);
	atmos_color.r *= 0.5 + 0.5 * clamp(sunrise_anticycle(1, 0.9, time.x), 0, 1); // TODO: make less janky

	float ambient_intensity = 0.2;
	// vec3 ambient = frag_col.rgb * ambient_intensity * mix(atmos_color, sun_color, 0.5 * clamp(day_cycle(1.0, 0.9, time.x), 0, 1));
	vec3 ambient = frag_col.rgb * ambient_intensity * atmos_color;

	float smoothness = 0.3;
	float roughness_linear = clamp(1 - (smoothness - 0.01), 0, 1);
	float roughness = roughness_linear * roughness_linear;

	float metallic = 0.0;
	float reflectance = 0.2;
	vec3 f0 = mix(vec3(mix(0.02, 0.18, reflectance)), frag_col.rgb, metallic);
	float f90 = 1.0;
	vec3 fresnel = f_Schlick(f0, f90, LdotH);
	float geo = vis_SmithGGXCorrelated(NdotL, NdotV, roughness);
	float ndf = ndf_GGX(NdotH, roughness);
	vec3 specular = fresnel * ndf * geo / PI;

	float fD = fr_DisneyDiffuse(NdotV, NdotL, LdotH, roughness_linear) / PI;
	vec3 diffuse = fD * frag_col.rgb;

	float sun_level = clamp(day_cycle(1, 0.9, time.x), 0.0, 1);
	float sun_intensity = sun_level * 80000;
	float sun_illuminance = sun_intensity * NdotL;

	vec3 lighted = ambient + ((diffuse + specular) * sun_color * sun_illuminance);

	// Mist
	float mist_start = view_distance.x * 0.7;// + snoise(vec4(world_pos, time) * 0.02) * 50.0;
	float mist_end = view_distance.x;// + snoise(vec4(world_pos, -time) * 0.02) * 50.0;
	float mist_delta = mist_end - mist_start;
	float play_dist = length(play_origin.xy - world_pos.xy);
	float dist = max(play_dist - mist_start, 0);
	float percent = clamp(dist / mist_delta, 0, 1);
	float mist_value = percent * percent * percent;

	target = mix(lighted, sky_chroma, mist_value);
}
