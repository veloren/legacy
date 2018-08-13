vec3 f_Schlick(vec3 f0, float f90, float u) {
    return f0 + (vec3(f90) - f0) * pow(1.0 - u, 5.0);
}

float fr_DisneyDiffuse (float NdotV, float NdotL, float LdotH, float linearRoughness) {
    float energyBias = mix(0, 0.5, linearRoughness);
    float energyFactor = mix(1.0, 1.0 / 1.51, linearRoughness);
    float fd90 = energyBias + 2.0 * LdotH * LdotH * linearRoughness;
    vec3 f0 = vec3(1.0, 1.0, 1.0);
    float lightScatter = f_Schlick(f0, fd90, NdotL).r;
    float viewScatter = f_Schlick(f0, fd90, NdotV).r;
    return lightScatter * viewScatter * energyFactor;
}

float vis_SmithGGXCorrelated(float NdotL, float NdotV, float roughness) {
    float r2 = roughness * roughness;
    float lambda_GGXV = NdotL * sqrt((-NdotV * r2 + NdotV) * NdotV + r2);
    float lambda_GGXL = NdotV * sqrt((-NdotL * r2 + NdotL) * NdotL + r2);

    return 0.5 / (lambda_GGXV + lambda_GGXL);
}

float ndf_GGX(float NdotH, float m) {
    float m2 = m * m;
    float f = (NdotH * m2 - NdotH) * NdotH + 1.0;
    return m2 / (f * f);
}