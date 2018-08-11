float get_time_of_day(float time) {
	return time / 60;
}

vec3 get_sun_dir(float time) {
	return vec3(-sin(3.14 * get_time_of_day(time)), 0.0, -cos(3.14 * get_time_of_day(time)));
}

vec3 get_sky_chroma(vec3 dir, float time) {
	vec3 sun_color = vec3(1.8, 1.5, 1.0);
	float sun_size = 0.0004;
	float sun_strength = 10000;
	float sun_bloom = 12;

	vec3 atmos_color = vec3(0.5, 0.7, 1.0) * min(max(cos(3.14 * get_time_of_day(time)), 0.05), 1);

	dir = normalize(dir);

	vec3 sun_dir = get_sun_dir(time);

	float angle = dot(-sun_dir, dir);
	float factor = (pow(angle, 1 / sun_bloom) - 1 + sun_size) * sun_strength * clamp(dot(vec3(0, 0, 1), dir) * 0.1, 0, 1);

	float red_factor = pow(clamp((dot(dir, -sun_dir) - abs(dot(vec3(0, 0, 1), dir)) - 0.3), 0, 1), 2);
	vec3 sky_color = mix(atmos_color, vec3(1, 0.3, 0), red_factor);

	return mix(sky_color.xyz, sun_color, clamp(factor, 0, 1));
}
