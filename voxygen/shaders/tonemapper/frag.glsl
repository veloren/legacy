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

// ACES fit by Stephen Hill (@self_shadow), adapted from the HLSL implementation
// here https://github.com/TheRealMJP/BakingLab/blob/master/BakingLab/ACES.hlsl
vec3 RRTAndODTFit( in vec3 v ) {
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

// Applies one of a selection of curves to either the luminance, chroma, or both
vec4 Curves( in vec4 colorInput, in int Mode, in int Formula, in float Contrast ) {
    float Contrast_blend;
    float luma;
    vec3 chroma;
    vec3 x;
    vec3 xstep;
    vec3 xstep_shift;
    vec3 shifted_x;
    vec3 color;
    vec3 color_1;

    Contrast_blend = Contrast;
    luma = dot( lumCoeff, colorInput.xyz );
    chroma = (colorInput.xyz  - luma);
    if ( (Mode == 0) ){
        x = vec3( luma);
    }
    else{
        if ( (Mode == 1) ){
            x = chroma, x = ((x * 0.500000) + 0.500000);
        }
        else{
            x = colorInput.xyz ;
        }
    }
    if ( (Formula == 0) ){
        x = sin( (1.57080 * x) );
        x *= x;
    }
    if ( (Formula == 1) ){
        x = (x - 0.500000);
        x = ((x / (0.500000 + abs( x ))) + 0.500000);
    }
    if ( (Formula == 2) ){
        x = ((x * x) * (3.00000 - (2.00000 * x)));
    }
    if ( (Formula == 3) ){
        x = (((1.05240 * exp( (6.00000 * x) )) - 1.05248) / (exp( (6.00000 * x) ) + 20.0855));
    }
    if ( (Formula == 4) ){
        x = (x * ((x * (1.50000 - x)) + 0.500000));
        Contrast_blend = (Contrast * 2.00000);
    }
    if ( (Formula == 5) ){
        x = (((x * x) * x) * ((x * ((x * 6.00000) - 15.0000)) + 10.0000));
    }
    if ( (Formula == 6) ){
        x = (x - 0.500000);
        x = ((x / ((abs( x ) * 1.25000) + 0.375000)) + 0.500000);
    }
    if ( (Formula == 7) ){
        x = ((((x * ((x * ((x * ((x * ((x * ((x * ((1.60000 * x) - 7.20000)) + 10.8000)) - 4.20000)) - 3.60000)) + 2.70000)) - 1.80000)) + 2.70000) * x) * x);
    }
    if ( (Formula == 8) ){
        x = (((-0.500000 * ((x * 2.00000) - 1.00000)) * (abs( ((x * 2.00000) - 1.00000) ) - 2.00000)) + 0.500000);
    }
    if ( (Formula == 9) ){
        xstep = step( x, vec3( 0.500000));
        xstep_shift = (xstep - 0.500000);
        shifted_x = (x + xstep_shift);
        x = (abs( (xstep - sqrt( ((( -shifted_x ) * shifted_x) + shifted_x) )) ) - xstep_shift);
        Contrast_blend = (Contrast * 0.500000);
    }
    if ( (Mode == 0) ){
        x = vec3( mix( luma, float( x), Contrast_blend));
        colorInput.xyz  = (x + chroma);
    }
    else{
        if ( (Mode == 1) ){
            x = ((x * 2.00000) - 1.00000);
            color = (luma + x);
            colorInput.xyz  = mix( colorInput.xyz , color, vec3( Contrast_blend));
        }
        else{
            color_1 = x;
            colorInput.xyz  = mix( colorInput.xyz , color_1, vec3( Contrast_blend));
        }
    }
    return colorInput;
}

// Intelligently increases saturation for pixels whose saturation is low while leaving higher
// saturation pixels less affected
vec3 Vibrance(vec3 color, float vibrance, vec3 bias) {
    float luma = dot(lumCoeff, color);

    float max = max(color.r, max(color.g, color.b));
    float min = min(color.r, min(color.g, color.b));

    float sat = max - min;

    vec3 v = bias * vibrance;

    color = mix(vec3(luma), color, 1 + (v * (1 - (sign(v) * sat))));

    return color;
}

vec3 ACESFitted(vec3 color)
{
    color = ACESInput * color;

    // Do Color correction
    color = ColorGrade * color;
    color = Vibrance(color, 0.25, vec3(1.0, 1.0, 1.0));
    // color = Curves(vec4(color, 1), 0, 8, 0.2).rgb;

    // Apply RRT and ODT
    color = RRTAndODTFit(color);

    color = ACESOutput * color;
    // Clamp to [0, 1]
    color = clamp(color, 0, 1);

    return color;
}

vec3 LinearTosRGB(in vec3 color)
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
    float ac = day_anticycle(1.0, 0.5, time.x);
    float fstop = ac * ac * -15.5 + 16.5;
    float Exp = pow(2.0, -fstop);
    vec3 mapped = hdrColor * Exp;
  
    // tone map
    mapped = ACESFitted(mapped);

    // gamma correction 
    mapped = LinearTosRGB(mapped);
  
    target = vec4(mapped, 1.0);
}    