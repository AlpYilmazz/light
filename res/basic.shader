#shader vertex
#version 140

in vec2 position;
out vec2 my_attr;

uniform mat4 model;
        
void main() {
    my_attr = position;
    gl_Position = model * vec4(position, 0.0, 1.0);
}

#shader fragment
#version 140

in vec2 my_attr;
out vec4 color;

void main() {
    color = vec4(my_attr, 0.0, 1.0);
}