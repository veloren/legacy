
float get_time_of_day(float time) {
	return mod(time / 60, 2.0);
}

vec3 get_sun_dir(float time) {
	return vec3(sin(PI * time), 0.0, cos(PI * time));
}

float day_cycle(float c, float factor, float time) {
	return cos(PI * c * time) * factor + 1.0 - factor;
}

float sunrise_cycle(float c, float factor, float time) {
	return sin(PI * 2 * c * time - PI/2) * factor + 1.0 - factor;
}

float sunrise_anticycle(float c, float factor, float time) {
	return cos(PI * 2 * c * time) * factor + 1.0 - factor;
}

float day_anticycle(float c, float factor, float time) {
	return cos(PI * c * time + PI) * factor + 1.0 - factor;
}

vec3 get_sun_color(float time) {
	return vec3(sunrise_cycle(1, 0.25, time) + 1.1, sunrise_anticycle(1, 0.2, time) + 0.2, sunrise_anticycle(1, 0.4, time) - 0.2);
}

vec3 interp(vec3 v1, vec3 v2, vec3 v3, float r1, float r2, float r3, float r4) {
	return r4 > 0.0 ? mix(v2, v1, r4)
		   : r3 > 0.0 ? mix(v3, v2, r3)
		   : r2 > 0.0 ? mix(v2, v3, r2)
		   : mix(v1, v2, r1);
}

float interp(float v1, float v2, float v3, float r1, float r2, float r3, float r4) {
	return r4 > 0.0 ? mix(v2, v1, r4)
		   : r3 > 0.0 ? mix(v3, v2, r3)
		   : r2 > 0.0 ? mix(v2, v3, r2)
		   : mix(v1, v2, r1);
}

////// Main gradient params://////
// Noon
const vec3 noon_top_col = vec3(0.45, 0.45, 0.5);
const float noon_top_strength = 20000.0;
const vec3 noon_mid_col = vec3(0.7, 0.8, 0.7);
const float noon_mid_strength = 20000.0;
const vec3 noon_bot_col = vec3(0.25, 0.3, 0.25);
const float noon_bot_strength = 15000.0;
// Sunset
const vec3 sunset_top_col = vec3(0.15, 0.1, 0.175);
const float sunset_top_strength = 6000;
const vec3 sunset_mid_col = vec3(1.0, 0.4, 0.225);
const float sunset_mid_strength = 2000;
const vec3 sunset_bot_col = vec3(0.075, 0.05, 0.1);
const float sunset_bot_strength = 1900;
// Midnight
const vec3 midnight_top_col = vec3(0.105, 0.11, 0.25);
const float midnight_top_strength = 0.5;
const vec3 midnight_mid_col = vec3(0.13, 0.14, 0.17);
const float midnight_mid_strength = 0.4;
const vec3 midnight_bot_col = vec3(0.075, 0.05, 0.1);
const float midnight_bot_strength = 0.3;

////// Sun disc params: ////
// Midday
const float noon_sun_strength = 100000.0;
// Sunset
const float sunset_sun_strength = 100000.0 * 0.25;
// Midnight
const float midnight_sun_strength = 0.0;
// Universal
const float sun_size = 0.03; // 3.5 deg ang diameter
const float sun_bloom = 1.5;

////// Sun halo params: ///////
// Midday
const vec3 noon_sun_halo_col = vec3(0.2, 0.8, 1.2);
const float noon_sun_halo_strength = 10000;
const float noon_sun_halo_bloom = 6;
// Sunset
const vec3 sunset_sun_halo_col = vec3(1.0, 0.10, 0.0);
const float sunset_sun_halo_strength = 15000;
const float sunset_sun_halo_bloom = 10;
// Midnight
const vec3 midnight_sun_halo_col = vec3(0.12, 0.10, 0.1);
const float midnight_sun_halo_strength = 0.0;
const float midnight_sun_halo_bloom = 0;

////// Horizon halo params: //////
// Midday
const vec3 noon_horiz_halo_col = vec3(2.0, 0.8, 0.2);
const float noon_horiz_halo_strength = 0.0;
// Sunset
const vec3 sunset_horiz_halo_col = vec3(2.0, 0.8, 0.2);
const float sunset_horiz_halo_strength = 4000;
// Midnight
const vec3 midnight_horiz_halo_col = vec3(0.12, 0.10, 0.11);
const float midnight_horiz_halo_strength = 0;
// Universal
const float horiz_halo_bloom = 6;

#define OUTPUT_GRADIENT
#define OUTPUT_DISC
#define OUTPUT_SUN_HALO
#define OUTPUT_HORIZ_HALO

vec3 get_sky(vec3 dir, float time, bool sun) {
	// Noon to sunset
	float nts = saturate(time * 2.0);
	// Sunset to midnight
	float x = clamp(time * 2.0 - 2.0, -1.0, 1.0);
	float sts = pow(max0(abs(x) * 2.0 - 1.0), 6.0);
	float stm = saturate(1.0 - sts);
	// Midnight to sunrise
	float mts = step(0.0, time * 2.0 - 2.0) * saturate(sts);
	// Sunrise to noon
	float stn = saturate(time * 2.0 - 3.0);

	// Main gradient variables:
	vec3 top_col = interp(noon_top_col, sunset_top_col, midnight_top_col, nts, stm, mts, stn);
	float top_strength = interp(noon_top_strength, sunset_top_strength, midnight_top_strength, nts, stm, mts, stn);
	vec3 mid_col = interp(noon_mid_col, sunset_mid_col, midnight_mid_col, nts, stm, mts, stn);
	float mid_strength = interp(noon_mid_strength, sunset_mid_strength, midnight_mid_strength, nts, stm, mts, stn);
	vec3 bot_col = interp(noon_bot_col, sunset_bot_col, midnight_bot_col, nts, stm, mts, stn);
	float bot_strength = interp(noon_bot_strength, sunset_bot_strength, midnight_bot_strength, nts, stm, mts, stn);

	// Sun disc params:
    vec3 sun_col = get_sun_color(time);
	float sun_strength = interp(noon_sun_strength, sunset_sun_strength, midnight_sun_strength, nts, stm, mts, stn);

	// Sun halo params:
	vec3 sun_halo_col = interp(noon_sun_halo_col, sunset_sun_halo_col, midnight_sun_halo_col, nts, stm, mts, stn);
	float sun_halo_strength = interp(noon_sun_halo_strength, sunset_sun_halo_strength, midnight_sun_halo_strength, nts, stm, mts, stn);
	float sun_halo_bloom = interp(noon_sun_halo_bloom, sunset_sun_halo_bloom, midnight_sun_halo_bloom, nts, stm, mts, stn);

	// Horizon halo params:
	vec3 horiz_halo_col = interp(noon_horiz_halo_col, sunset_horiz_halo_col, midnight_horiz_halo_col, nts, stm, mts, stn);
	float horiz_halo_strength = interp(noon_horiz_halo_strength, sunset_horiz_halo_strength, midnight_horiz_halo_strength, nts, stm, mts, stn);
	float horiz_halo_bloom = 10;

	// Output
	vec3 output_col = vec3(0);

	// Main gradient builder
    float dottop = dot(vec3(0,0,1), dir);
	float dotbot = -dottop;
	float omdt = 1 - dottop;
	float omdt2 = omdt * omdt;
	float omdt4 = omdt2 * omdt2;
	float omdb = 0.6 - 0.6 * dotbot;
	float ssdt = smoothstep(-0.0, 0.2, dottop);
	float ssds = smoothstep(-0.0, 0.2, dottop);
	float ssdb = smoothstep(-0.2, 0.0, dotbot);

	#ifdef OUTPUT_GRADIENT
	output_col += mix(top_col * top_strength, mid_col * mid_strength, omdt4) * ssdt;
	output_col += mix(bot_col * bot_strength, mid_col * mid_strength, omdb * omdb) * ssdb;
	// output_col += mid_col * mid_strength * omdt4 * ssdt;
	// output_col += mid_col * mid_strength * omdb * omdb * ssdb;
	#endif

	// Sun disc builder
	vec3 sun_dir = get_sun_dir(time);
	float ds = dot(sun_dir, dir);
	float dotsun = saturate(ds);
	float d = cos(sun_size);
	float disc_factor = smoothstep(d - sun_bloom / 1000, d, dotsun) * ssds;

	#ifdef OUTPUT_DISC
	output_col += sun ? sun_col * sun_strength * disc_factor : vec3(0.0);
	#endif

	// Sun halo builder
	float halo_factor = pow(dotsun, 1 / (sun_halo_bloom + sun_halo_bloom * omdt4 * omdt2) * 100);
	halo_factor += 0.5 * pow(dotsun, 1 / (sun_halo_bloom * omdt4) * 200);
	halo_factor *= ssds;

	#ifdef OUTPUT_SUN_HALO
	output_col += sun_halo_col * sun_halo_strength * saturate(halo_factor);
	#endif

	// horizon halo builder
	float horiz_halo_factor = pow(1 - abs(dottop), 1 / horiz_halo_bloom * 100);
	float sun_fac = saturate((PI - acos(ds)) / PI);
	#ifdef OUTPUT_HORIZ_HALO
	output_col += horiz_halo_col * horiz_halo_strength * horiz_halo_factor * sun_fac * sun_fac;
	#endif

	return output_col;
	// return sun_col * 10000;
}

vec3 get_sky_chroma(vec3 dir, float time) {
	return get_sky(dir, time, false) * 3.0 * vec3(0.4, 0.65, 1.5);
}

vec3 get_skybox(vec3 dir, float time) {
	return get_sky(dir, time, true) * 3.0 * vec3(0.4, 0.65, 1.5);
}
