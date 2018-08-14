#version 330 core

#include <noise.glsl>
#include <common.glsl>
#include <luts.glsl>
#include <sky.glsl>
#include <bsdf.glsl>

in vec3 frag_pos;
in vec3 frag_world_pos;
in vec4 frag_col;
in float frag_ao;
flat in vec3 frag_norm;
flat in uint frag_mat;

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

const vec3 off = vec3(-1, 0, 1);
const vec2 offsets[5] = vec2[5](
    off.yy, off.xy, off.zy, off.yx, off.yz
);

void main() {
	Material mat = mat_lut[frag_mat];
	// Sunlight
	float sunAngularRadius = 0.017; // 1 degree radius, 2 degree diameter (not realistic, irl sun is ~0.5 deg diameter)
	float time_of_day = get_time_of_day(time.x);
	vec3 sun_color = get_sun_color(time_of_day);
	vec3 sun_dir = get_sun_dir(time_of_day);
	
	// Bump
	float s[5];
	float scale = 0.25;
	vec2 size = vec2(scale * 0.01, 0.0);
	for (int i = 0; i < 5; i++) {
			vec2 offset = offsets[i] * size.x;
			s[i] = snoise(vec3((frag_world_pos.xy * scale) + offset, time_of_day * 60)) * 0.5 + 1.0;
	}
	vec3 va = normalize(vec3(size.xy,s[2]-s[1]));
	vec3 vb = normalize(vec3(size.yx,s[4]-s[3]));
	float bumpFactor = 0.05;
	vec4 bump = vec4( cross(va,vb), s[0] * bumpFactor );

	// Geometry
	vec3 N = normalize((model_mat * vec4(frag_norm, 0)).xyz);
	N = normalize(N + bump.xyz * bump.w);
	vec3 V = normalize(cam_origin.xyz - frag_world_pos);

	// calculate closest direction on sun's disk to reflection vector
	float r = sin(sunAngularRadius);
	float d = cos(sunAngularRadius);
	vec3 R = reflect(-V, N);
	float DdotR = dot(sun_dir, R);
	vec3 S = R - DdotR * sun_dir;
	vec3 L = DdotR < d ? normalize(d * sun_dir + normalize(S) * r) : R;

	float NdotV = abs(dot(N, V));
	float NdotL = clamp(dot(N, L), 0.0, 1.0);
	vec3 H = normalize(V + L);
	float LdotH = clamp(dot(L, H), 0.0, 1.0);
	float VdotH = clamp(dot(V, H), 0.0, 1.0);
	float NdotH = clamp(dot(N, H), 0.0, 0.99999995);// fix artifact

	vec3 sky_chroma = get_sky_chroma(-V, time_of_day);
	vec3 atmos_color = get_sky_chroma(N, time_of_day);
	atmos_color.r *= 0.5 + 0.5 * clamp(sunrise_anticycle(1, 0.9, time_of_day), 0, 1); // TODO: make less janky

	float ao = (frag_ao / 3.0);
	ao *= 1 - bump.w;
	float ambient_intensity = 0.2;
	vec3 ambient = frag_col.rgb * ambient_intensity * ao * atmos_color;

	float smoothness = mat.smoothness;
	float roughness_linear = clamp(1 - (smoothness - 0.01), 0, 1);
	float roughness = roughness_linear * roughness_linear;

	float reflectance = mat.reflectance;
	float metalness = mat.metalness;
	vec3 f0 = mix(vec3(mix(0.02, 0.18, reflectance)), frag_col.rgb, metalness);
	float f90 = 1.0;
	vec3 fresnel = f_Schlick(f0, f90, VdotH);
	float geo = vis_SmithGGXCorrelated(NdotL, NdotV, roughness);
	float ndf = ndf_GGX(NdotH, roughness);
	vec3 specular = fresnel * ndf * geo / PI;

	float fD = fr_DisneyDiffuse(NdotV, NdotL, LdotH, roughness_linear) / PI;
	vec3 diffuse = fD * frag_col.rgb;

	float sun_level = clamp(day_cycle(1, 0.9, time_of_day), 0.0, 1);
	float sun_intensity = sun_level * 80000;
	float sun_illuminance = sun_intensity * NdotL;

	vec3 lighted = ambient + ((diffuse + specular) * sun_color * sun_illuminance * ao);

	// Mist
	float mist_start = view_distance.x * 0.7;// + snoise(vec4(world_pos, time) * 0.02) * 50.0;
	float mist_end = view_distance.x;// + snoise(vec4(world_pos, -time) * 0.02) * 50.0;
	float mist_delta = mist_end - mist_start;
	float play_dist = length(play_origin.xy - frag_world_pos.xy);
	float dist = max(play_dist - mist_start, 0);
	float percent = clamp(dist / mist_delta, 0, 1);
	float mist_value = percent * percent * percent;

	float fres_n = f_Schlick(f0, f90, NdotV).r;
    float smax = max(specular.r, max(specular.g, specular.b));
    float a = mix(saturate(frag_col.a + smax), 1, fres_n);
	target = mix(vec4(lighted, a), vec4(sky_chroma, 1.0), mist_value);
}
