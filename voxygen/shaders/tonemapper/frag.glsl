#version 330 core

#include <sky.glsl>

in vec2 uv;

uniform sampler2D t_Hdr;

layout (std140)
uniform global_consts {
	mat4 view_mat;
	mat4 proj_mat;
	vec4 cam_origin;
	vec4 play_origin;
	vec4 view_distance;
	vec4 time;
};

// The tonemapping curve from Uncharted 2, released by the author here
// http://filmicworlds.com/blog/filmic-tonemapping-operators/
vec3 uncharted2Tonemap(const vec3 x) {
	const float A = 0.15;
	const float B = 0.50;
	const float C = 0.10;
	const float D = 0.20;
	const float E = 0.02;
	const float F = 0.30;
	return ((x * (A * x + C * B) + D * E) / (x * (A * x + B) + D * F)) - E / F;
}

vec3 tonemapUncharted2(const vec3 color, const float exposureBias) {
	const float W = 11.2;
	vec3 curr = uncharted2Tonemap(exposureBias * color);
	vec3 whiteScale = 1.0 / uncharted2Tonemap(vec3(W));
	return curr * whiteScale;
}

// Luminance-only ACES tonemapping fit from
// https://knarkowicz.wordpress.com/2016/01/06/aces-filmic-tone-mapping-curve/
vec3 acesFilm(const vec3 x) {
    const float a = 2.51;
    const float b = 0.03;
    const float c = 2.43;
    const float d = 0.59;
    const float e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d ) + e), 0.0, 1.0);
}

vec3 RRTAndODTFit(vec3 v)
{
    vec3 a = v * (v + vec3(0.0245786)) - vec3(0.000090537);
    vec3 b = v * (0.983729 * v + vec3(0.4329510)) + vec3(0.238081);
    return a / b;
}

// ACES fit by Stephen Hill (@self_shadow), adapted from the HLSL implementation
// here https://github.com/TheRealMJP/BakingLab/blob/master/BakingLab/ACES.hlsl
vec3 ACESFitted(vec3 color)
{

    // Apply RRT and ODT
    color = RRTAndODTFit(color);

    // Clamp to [0, 1]
    color = clamp(color, 0, 1);

    return color;
}

out vec4 target;

void main() {             
    const float gamma = 2.2;
    vec3 hdrColor = texture(t_Hdr, uv.xy).rgb;

    // exposure correction. Varies between F/16 at midday and F/2.8 at night.
    float ac = day_anticycle(1.0, 0.5, time.x);
    float fstop = ac * ac * -13.2 + 16;
    float Exp = pow(2.0, -fstop);
    vec3 mapped = hdrColor * Exp;
  
    // tone map
    // mapped = ACESFitted(mapped);
    mapped = acesFilm(mapped);
    // mapped = tonemapUncharted2(mapped, Exp);

    // gamma correction 
    mapped = pow(mapped, vec3(1.0 / gamma));
  
    target = vec4(mapped, 1.0);
}    