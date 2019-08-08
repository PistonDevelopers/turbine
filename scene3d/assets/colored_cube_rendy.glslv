#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 vertexPosition_modelspace;
layout(location = 1) in vec4 vertexColor;
layout(location = 2) in vec2 vertexTexCoords;
layout(location = 3) in vec3 vertexNormals;

layout(set = 0, binding = 0) uniform Args {
    uniform mat4 MVP;
};

layout(location = 0) out vec4 fragmentColor;

void main() {
    gl_Position = MVP * vec4(vertexPosition_modelspace, 1);
    fragmentColor = vertexColor;
}
