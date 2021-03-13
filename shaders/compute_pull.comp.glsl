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
    Bubble input_bubbles[];
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

vec2 calculate_bubble_repulsion(Bubble bubble_from, Bubble bubble_to) {
    vec2 d_from_to = bubble_to.p - bubble_from.p;
    float pull_force_factor = 1.0;
    vec2 pull_force_from_to = d_from_to * pull_force_factor;

    vec2 a_from = pull_force_from_to / bubble_from.m;
    return a_from;
}

int compute_repulsion(uint bubble_index) {
    for (int i = 0; i < edge_count; i++) {
        Edge edge = edges[i];
        int i_from = edge.from;
        int i_to = edge.to;
        int connected_index = -1;
        if (i_from == bubble_index) {
            connected_index = i_to;
        }
        if (i_to == bubble_index) {
            connected_index = i_from;
        }
        if (connected_index != -1) {
            Bubble bubble = input_bubbles[bubble_index];
            Bubble connected_bubble = input_bubbles[connected_index];
            bubble.a = bubble.a + calculate_bubble_repulsion(bubble, connected_bubble);
            input_bubbles[bubble_index] = bubble;
        }
    }

    return 0;
}

// there should be only one entry point and it should be named "main"
// ultimately, Emu has to kind of restrict how you use GLSL because it is compute focused
void main() {
    uint index = gl_GlobalInvocationID.x; // this gives us the index in the x dimension of the thread space
    compute_repulsion(index);
}