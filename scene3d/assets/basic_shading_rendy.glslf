#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 UV;
layout(location = 3) in vec3 LightDirection_cameraspace;
layout(location = 4) in vec3 Normal_cameraspace;

layout(set = 0, binding = 0) uniform Args {
    uniform mat4 MVP;
    uniform mat4 M;
    uniform mat4 V;
    uniform vec3 LightPosition_worldspace;
    uniform float ambientLight;
};

layout(set = 1, binding = 0) uniform sampler2D myTextureSampler;

layout(location = 0) out vec4 color;

void main() {
    vec3 MaterialDiffuseColor = texture(myTextureSampler, UV).rgb;
    vec3 MaterialAmbientColor = MaterialDiffuseColor * ambientLight;
    vec3 n = normalize(Normal_cameraspace);
    vec3 l = normalize(LightDirection_cameraspace);
    float cosTheta = clamp(dot(n, l), 0, 1);
    color = vec4(MaterialAmbientColor + MaterialDiffuseColor * cosTheta, 1.0);
}
