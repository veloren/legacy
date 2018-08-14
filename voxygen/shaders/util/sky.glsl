#define PI 3.14159256

float get_time_of_day(float time) {
	return time / 60;
}

vec3 get_sun_dir(float time) {
	return vec3(-sin(PI * get_time_of_day(time)), 0.0, -cos(PI * get_time_of_day(time)));
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

vec3 get_sky_chroma(vec3 dir, float time) {
    vec3 sun_color = get_sun_color(time) * 80000.0;
	float sun_size = 0.0004;
	float sun_strength = 10000;
	float sun_bloom = 20;

    float c = clamp(day_cycle(1.0,0.5,time), 0, 1);
    vec3 atmos_color = get_atmos_color(time) * 20000.0 * clamp(c*c*c*c, 0.00001, 1);

	dir = normalize(dir);

	vec3 sun_dir = get_sun_dir(time);

	float angle = dot(-sun_dir, dir);
    float dothoriz = dot(vec3(0,0,1), dir);
	float factor = (pow(angle, 1 / sun_bloom) - 1 + sun_size) * sun_strength * clamp(dothoriz * 0.1, 0, 1);

	float red_factor = pow(clamp((angle - abs(dothoriz) - 0.3), 0, 1) * smoothstep(-0.1, 0.0, dothoriz), 2);
	vec3 sky_color = mix(atmos_color, vec3(1, 0.15, 0.0) * 25000.0 * clamp(c*c, 0.00001, 1), red_factor);

	return mix(sky_color.xyz, sun_color, clamp(factor, 0, 1));
}
