#version 330 core

#include <common.glsl>
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

// ACES fit by Stephen Hill (@self_shadow), adapted from the HLSL implementation
// here https://github.com/TheRealMJP/BakingLab/blob/master/BakingLab/ACES.hlsl
vec3 rrt_and_odt( in vec3 v ) {
    vec3 a;
    vec3 b;

    a = ((v * (v + 0.0245786)) - 0.000090537);
    b = ((v * ((0.983729 * v) + 0.432951)) + 0.238081);
    return (a / b);
}

// Rec709 .. XYZ .. D65_D60 .. AP1 .. RRT_SAT
// Converts from linear rec709 space to ACES space
const mat3 ACESInput = mat3 (
    0.59719, 0.07600, 0.02840,
    0.35458, 0.90834, 0.13383,
    0.04823, 0.01566, 0.83777
);

// ODT_SAT .. XYZ .. D60_D65 .. Rec709
// Converts from OCES to linear rec709
const mat3 ACESOutput = mat3 (
    1.60475, -0.10208, -0.00327,
    -0.53108, 1.10813, -0.00605,
    -0.00327, -0.07276, 1.07602
);

// The way this works is that each column (well, each visual column
// since in reality glsl matrices are column-major) maps to
// the resultant R, G, and B value respectively. The (visual) rows
// represent the current R/G/B value and you add the contributions
// down the column. For example if column 1 was
// [ 1.0 ]
// [ 1.5 ]
// [ 2.0 ]
// Then the resulting R value for the pixel would be
// (current R * 1.0) + (current B * 1.5) + (current G * 2.0)
// Currently this just desaturates reds slightly
const mat3 ColorGrade = mat3 (
     0.95000,  0.00000,  0.00000,
     0.02500,  1.00000,  0.00000,
     0.02500,  0.00000,  1.00000
);

vec3 lumCoeff = vec3( 0.212600, 0.715200, 0.0722000);

// Intelligently increases saturation for pixels whose saturation is low while leaving higher
// saturation pixels less affected
vec3 vibrance(vec3 color, float vibrance, vec3 bias) {
    float luma = dot(lumCoeff, color);

    float max = max(color.r, max(color.g, color.b));
    float min = min(color.r, min(color.g, color.b));

    float sat = max - min;

    vec3 v = bias * vibrance;

    color = mix(vec3(luma), color, 1 + (v * (1 - (sign(v) * sat))));

    return color;
}

vec3 aces(vec3 color)
{
    color = ACESInput * color;

    // Do Color correction
    color = ColorGrade * color;
    color = vibrance(color, 0.35, vec3(1.0, 1.0, 1.0));
    // color = Curves(vec4(color, 1), 0, 8, 0.2).rgb;

    // Apply RRT and ODT
    color = rrt_and_odt(color);

    color = ACESOutput * color;
    // Clamp to [0, 1]
    color = clamp(color, 0, 1);

    return color;
}

vec3 linear_to_srgb(in vec3 color)
{
    vec3 x = color * 12.92;
    vec3 y = 1.055 * pow(clamp(color, 0, 1), vec3(1.0 / 2.4)) - vec3(0.055);

    vec3 clr = color;
    clr.r = color.r < 0.0031308 ? x.r : y.r;
    clr.g = color.g < 0.0031308 ? x.g : y.g;
    clr.b = color.b < 0.0031308 ? x.b : y.b;

    return clr;
}

out vec4 target;

void main() {
    vec3 hdrColor = texture(t_Hdr, uv.xy).rgb;

    // exposure correction. Varies between F/16 at midday and F/2.8 at night.
    float tod = get_time_of_day(time.x);
    float day_part = saturate(cos(PI * tod));
	float x = clamp(tod * 2.0 - 2.0, -1.0, 1.0);
	float night_part = 1.0 - pow(max0(abs(x) * 2.0 - 1.0), 6.0);
    float denom = 3.0 + (0.2 + 0.8 * day_part - 0.2 * night_part) * 60000.0;
    float exposure = 1.0 / denom;
    vec3 mapped = hdrColor * exposure;

    // tone map
    mapped = aces(mapped);

    // gamma correction
    //mapped = linear_to_srgb(mapped);

    target = vec4(mapped, 1.0);
    //target = vec4(hdrColor, 1.0);
}
