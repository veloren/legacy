vec3 get_sun_dir(float time) {
	float day_length = 600;
	return normalize(vec3(-1));
	return vec3(-sin(time / (3.14 * day_length)), 0.0, -cos(time / (3.14 * day_length)));
}

vec3 get_sky_chroma(vec3 sky_color, vec3 dir, float time) {
	vec3 sun_color = vec3(1.5, 1.5, 1.0);
	float sun_size = 0.01;
	float sun_strength = 50;
	float sun_bloom = 8;

	vec3 sun_dir = get_sun_dir(time);

	float angle = dot(-sun_dir, normalize(dir));
	float factor = (pow(angle, 1 / sun_bloom) - 1 + sun_size) * sun_strength;

	return mix(sky_color.xyz, sun_color, clamp(factor, 0, 1));
}
