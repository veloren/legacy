
#define PI 3.14159256

float saturate(float v) {
    return clamp(v, 0.0, 1.0);
}

vec3 saturate(vec3 v) {
    return clamp(v, vec3(0.0), vec3(1.0));
}

float max0(float v) {
    return max(v, 0.0);
}

vec3 max0(vec3 v) {
    return max(v, vec3(0.0));
}
