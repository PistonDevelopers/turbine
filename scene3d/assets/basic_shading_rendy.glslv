#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 vertexPosition_modelspace;
layout(location = 1) in vec4 vertexColor;
layout(location = 2) in vec2 vertexUV_modelspace;
layout(location = 3) in vec3 vertexNormal_modelspace;

layout(set = 0, binding = 0) uniform Args {
    uniform mat4 MVP;
    uniform mat4 M;
    uniform mat4 V;
    uniform vec3 LightPosition_worldspace;
    uniform float ambientLight;
};

layout(location = 0) out vec2 UV;
layout(location = 1) out vec3 Position_worldspace;
layout(location = 2) out vec3 EyeDirection_cameraspace;
layout(location = 3) out vec3 LightDirection_cameraspace;
layout(location = 4) out vec3 Normal_cameraspace;

void main() {
    gl_Position = MVP * vec4(vertexPosition_modelspace, 1);
    // Position of vertex in world space.
    Position_worldspace = (M * vec4(vertexPosition_modelspace, 1)).xyz;
    // Vector that goes from vertex to the camera, in camera space.
    vec3 vertexPosition_cameraspace = (V * M * vec4(vertexPosition_modelspace, 1)).xyz;
    EyeDirection_cameraspace = vec3(0, 0, 0) - vertexPosition_cameraspace;
    // Vector that goes from vertex to the light, in camera space.
    // M is omitted because it is identity.
    vec3 LightPosition_cameraspace = (V * vec4(LightPosition_worldspace, 1)).xyz;
    LightDirection_cameraspace = LightPosition_cameraspace + EyeDirection_cameraspace;
    // Normal of the vertex, in camera space.
    // Use its inverse transpose if M scales the model.
    Normal_cameraspace = (V * M * vec4(vertexNormal_modelspace, 0)).xyz;
    UV = vertexUV_modelspace;
}
