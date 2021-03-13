#version 450
layout(local_size_x = 1) in; // our thread block size is 1, that is we only have 1 thread per block

struct Bubble {
    float m;
    float _pad1;
    vec2 p;
    vec2 v;
    vec2 a;
};

struct Edge {
    int from;
    int to;
    int _pad1;
    int _pad2;
};

// make sure to use only a single set and keep all your n parameters in n storage buffers in bindings 0 to n-1
// you shouldn't use push constants or anything OTHER than storage buffers for passing stuff into the kernel
// just use buffers with one buffer per binding
layout(std140, binding = 0) buffer B1 {
    Bubble[] input_bubbles;
}; // this is used as both input and output for convenience

layout(std140, binding = 1) buffer B2 {
    uint bubble_count;
    uint edge_count;
    uint _pad1;
    uint _pad2;
}; // this is used as both input and output for convenience

layout(std140, binding = 2) buffer B3 {
    Edge[] edges;
};

int compute_repulsion(uint bubble_index) {
    // uint bubble_count = gl_NumWorkGroups.x;
    Bubble input_bubble = input_bubbles[bubble_index];
    input_bubble.a = vec2(0.0, 0.0);
    for (int i = 0; i < bubble_count; i++) {
        if (i == bubble_index) {
            continue;
        }
        Bubble bubble_b = input_bubbles[i];
        vec2 d_ab = bubble_b.p - input_bubble.p;
        float len = max(d_ab.length(), 0.1);
        vec2 nd_ab = normalize(d_ab);
        // float repulsive_force_factor = 0.00012;
        float repulsive_force_factor = 1;

        vec2 repulsive_force = nd_ab * (repulsive_force_factor * input_bubble.m * bubble_b.m / (len * len));
        vec2 a_a = repulsive_force * (-1.0 / input_bubble.m);
        // vec2 a_a = vec2(0.0, 0.0) * (-1.0 / input_bubble.m);
        // input_bubble.a = input_bubble.a + a_a;
        input_bubble.a = bubble_b.p;
        input_bubble.v = d_ab;
        input_bubble.p = vec2(1.0, 0);
        // input_bubble.m = (repulsive_force_factor * input_bubble.m * bubble_b.m / (len * len));
        input_bubble.p = vec2(1.0, 0.0);
        input_bubble.m = input_bubble.p.length();
    }
    // input_bubble.a = vec2(bubble_index, bubble_count);
    input_bubbles[bubble_index] = input_bubble;
    return 0;
}

// there should be only one entry point and it should be named "main"
// ultimately, Emu has to kind of restrict how you use GLSL because it is compute focused
void main() {
    uint index = gl_GlobalInvocationID.x; // this gives us the index in the x dimension of the thread space
    compute_repulsion(index);
}