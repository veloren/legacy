#define PI 3.14159256

float get_time_of_day(float time) {
	return 1.0;
	// return time / 60;
}

vec3 get_sun_dir(float time) {
	return vec3(sin(PI * get_time_of_day(time)), 0.0, cos(PI * get_time_of_day(time)));
}

float day_cycle(float c, float factor, float time) {
	return cos(PI * c * get_time_of_day(time)) * factor + 1.0 - factor;
}

float sunrise_cycle(float c, float factor, float time) {
	return sin(PI * 2 * c * get_time_of_day(time) - PI/2) * factor + 1.0 - factor;
}

float sunrise_anticycle(float c, float factor, float time) {
	return cos(PI * 2 * c * get_time_of_day(time)) * factor + 1.0 - factor;
}

float day_anticycle(float c, float factor, float time) {
	return cos(PI * c * get_time_of_day(time) + PI) * factor + 1.0 - factor;
}

vec3 get_sun_color(float time) {
	return vec3(sunrise_cycle(1, 0.25, time) + 1.0, sunrise_anticycle(1, 0.2, time) + 0.4, sunrise_anticycle(1, 0.4, time));
}

vec3 get_atmos_color(float time) {
    float ac = day_anticycle(1.0, 0.5, time);
	vec3 atmos_color = vec3(
		0.25 + 0.2 * sunrise_cycle(1.0, 0.5, time),
        0.5 - 0.1 * ac,
        1.0
    );
    return atmos_color;
}

vec3 hsv2rgb(vec3 c) {
  vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
  return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

vec3 get_sky_chroma(vec3 dir, float time) {
	// Main gradient variables:
	// Midday
	// vec3 top_col = vec3(0.02, 0.3, 1.0);
	// float top_strength = 20000.0;
	// vec3 mid_col = vec3(0.2, 0.8, 1.2);
	// float mid_strength = 20000.0;
	// vec3 bot_col = vec3(0.03, 0.15, 0.4);
	// float bot_strength = 20000.0;

	// Sunset
	// vec3 top_col = vec3(0.15, 0.1, 0.175);
	// float top_strength = 20000.0 * 0.05;
	// vec3 mid_col = vec3(1.0, 0.15, 0.0);
	// float mid_strength = 20000.0 * 0.1;
	// vec3 bot_col = vec3(0.075, 0.05, 0.1);
	// float bot_strength = 20000.0 * 0.05;

	// Midnight
	vec3 top_col = vec3(0.105, 0.11, 0.25);
	float top_strength = 20000.0 * 0.00002;
	vec3 mid_col = vec3(0.13, 0.14, 0.17);
	float mid_strength = 20000.0 * 0.0000025;
	vec3 bot_col = vec3(0.075, 0.05, 0.1);
	float bot_strength = 20000.0 * 0.000005;

	// Sun disc params:
    vec3 sun_col = get_sun_color(time);
	// Midday
	// float sun_strength = 100000.0;
	// Sunset
	// float sun_strength = 100000.0 * 0.25;
	// Midnight
	float sun_strength = 0.0;
	float sun_size = 0.017; // 2 deg ang diameter
	float sun_bloom = 1;

	// Sun halo params:
	// Midday
	// vec3 sun_halo_col = vec3(0.2, 0.8, 1.2);
	// float sun_halo_strength = 20000.0 * 0.5;
	// Sunset
	// vec3 sun_halo_col = vec3(1.0, 0.10, 0.0);
	// float sun_halo_strength = 20000.0 * 0.5;
	// Midnight
	vec3 sun_halo_col = vec3(0.12, 0.10, 0.1);
	float sun_halo_strength = 0.0;
	float sun_halo_bloom = 10;

	// Horizon halo params:
	// Midday
	// vec3 horiz_halo_col = vec3(2.0, 0.8, 0.2);
	// float horiz_halo_strength = 0.0;
	// Sunset
	// vec3 horiz_halo_col = vec3(2.0, 0.8, 0.2);
	// float horiz_halo_strength = 20000.0 * 0.1;
	// Midnight
	vec3 horiz_halo_col = vec3(0.12, 0.10, 0.11);
	float horiz_halo_strength = 20000.0 * 0.000001;
	float horiz_halo_bloom = 10;

	vec3 output_col = vec3(0);

	// Main gradient builder
    float dottop = dot(vec3(0,0,1), dir);
	float dotbot = -dottop;
	float omdt = 1 - dottop;
	float omdt2 = omdt * omdt;
	float omdt4 = omdt2 * omdt2;
	float omdb = 0.6 - 0.6 * dotbot;
	float ssdt = smoothstep(-0.01, 0.0, dottop);
	float ssdb = smoothstep(0.0, 0.01, dotbot);
	output_col += top_col * top_strength * ssdt;
	output_col += bot_col * bot_strength * ssdb;
	output_col += mid_col * mid_strength * omdt4 * ssdt;
	output_col += mid_col * mid_strength * omdb * omdb * ssdb;

	// Sun disc builder
	vec3 sun_dir = get_sun_dir(time);
	float dotsun = clamp(dot(sun_dir, dir), 0, 1);
	float d = cos(sun_size);
	float disc_factor = smoothstep(d - sun_bloom / 1000, d, dotsun) * ssdt;
	output_col += sun_col * sun_strength * disc_factor;

	// Sun halo builder
	float halo_factor = pow(dotsun, 1 / (sun_halo_bloom + sun_halo_bloom * omdt4 * omdt2) * 100);
	halo_factor += 0.5 * pow(dotsun, 1 / (sun_halo_bloom * omdt4) * 200);
	halo_factor *= ssdt;
	output_col += sun_halo_col * sun_halo_strength * clamp(halo_factor, 0, 1);

	// horizon halo builder
	float horiz_halo_factor = pow(1 - abs(dottop), 1 / horiz_halo_bloom * 100);
	output_col += horiz_halo_col * horiz_halo_strength * horiz_halo_factor;


    // float c = clamp(day_cycle(1.0,0.5,time), 0, 1);
    // vec3 atmos_color = get_atmos_color(time) * 20000.0 * clamp(c*c*c*c, 0.00001, 1);

	// float angle = dot(-sun_dir, dir);

	// float factor = (pow(angle, 1 / sun_bloom) - 1 + sun_size) * sun_strength * clamp(dothoriz * 0.1, 0, 1);

	// float red_factor = pow(clamp((angle - abs(dothoriz) - 0.3), 0, 1) * smoothstep(-0.1, 0.0, dothoriz), 2);
	// vec3 sky_color = mix(atmos_color, vec3(1, 0.15, 0.0) * 25000.0 * clamp(c*c, 0.00001, 1), red_factor);

	// return mix(sky_color.xyz, sun_color, clamp(factor, 0, 1));
	return output_col;
}
