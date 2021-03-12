#!/bin/sh
glslangValidator -V ./shaders/geometry.vert.glsl -o ./shaders/geometry.vert.spv
glslangValidator -V ./shaders/geometry.frag.glsl -o ./shaders/geometry.frag.spv
glslangValidator -V ./shaders/background.vert.glsl -o ./shaders/background.vert.spv
glslangValidator -V ./shaders/background.frag.glsl -o ./shaders/background.frag.spv
glslangValidator -V ./shaders/compute_repulsion.comp.glsl -o ./shaders/compute_repulsion.comp.spv
glslangValidator -V ./shaders/compute_pull.comp.glsl -o ./shaders/compute_pull.comp.spv
glslangValidator -V ./shaders/compute_position.comp.glsl -o ./shaders/compute_position.comp.spv
