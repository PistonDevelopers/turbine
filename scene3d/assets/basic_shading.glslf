#version 330 core
in vec2 UV;
in vec3 LightDirection_cameraspace;
in vec3 Normal_cameraspace;

uniform sampler2D myTextureSampler;
uniform float ambientLight;

out vec3 color;

void main() {
    vec3 MaterialDiffuseColor = texture(myTextureSampler, UV).rgb;
    vec3 MaterialAmbientColor = MaterialDiffuseColor * ambientLight;
    vec3 n = normalize(Normal_cameraspace);
    vec3 l = normalize(LightDirection_cameraspace);
    float cosTheta = clamp(dot(n, l), 0, 1);
    color = MaterialAmbientColor + MaterialDiffuseColor * cosTheta;
}
