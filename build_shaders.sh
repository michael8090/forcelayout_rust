#!/bin/sh

for file in $(find . -type f -name "*.glsl"); do
    glslangValidator -V ${file} -o ${file/glsl/spv};
done