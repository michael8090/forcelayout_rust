#version 450
layout(local_size_x = 1) in; // our thread block size is 1, that is we only have 1 thread per block

struct Bubble {
    float m;
    vec2 p;
    vec2 v;
    vec2 a;
};

struct Edge {
    int from;
    int to;
};

// make sure to use only a single set and keep all your n parameters in n storage buffers in bindings 0 to n-1
// you shouldn't use push constants or anything OTHER than storage buffers for passing stuff into the kernel
// just use buffers with one buffer per binding
layout(std140, binding = 0) buffer B1 {
    Bubble input_bubbles[2];
}; // this is used as both input and output for convenience

layout(std140, binding = 1) buffer B2 {
    uint bubble_count;
    uint edge_count;
}; // this is used as both input and output for convenience

layout(std140, binding = 2) buffer B3 {
    Edge edges[1];
};

int compute_position(uint bubble_index) {
    // uint bubble_count = gl_NumWorkGroups.x;
    float time_step = 0.5;
    Bubble bubble = input_bubbles[bubble_index];
    // bubble.v = bubble.v + bubble.a * time_step;
    // float damping_factor = 1.0 - atan(bubble.v.length() * 0.1) *2.0 / 3.14159265358979323846;
    // bubble.v = bubble.v * min(damping_factor, 0.9);
    // bubble.p = bubble.p + bubble.v * time_step;
    bubble.p = vec2(1.0, 1.0);
    bubble.v = vec2(1.0, 1.0);
    bubble.a = vec2(1.0, 1.0);
    bubble.m = 1.0;

    input_bubbles[bubble_index] = bubble;
    return 0;
}

// there should be only one entry point and it should be named "main"
// ultimately, Emu has to kind of restrict how you use GLSL because it is compute focused
void main() {
    uint index = gl_GlobalInvocationID.x; // this gives us the index in the x dimension of the thread space
    compute_position(index);
}