﻿# forcelayout_rust

A simple forcelayout, with

- lyon to build gpu friendly geometries
- wgpu-rs for rendering
- using compute shader to do the calculation


Currently it can handle about 10000 bubbles and 9999 edges with reasonable performance.

Below is how it looks with 5000 bubbles and 4999 edges:

![5000 bubbles](https://user-images.githubusercontent.com/2306105/111027247-11611500-842a-11eb-9225-5299ab6f7988.PNG)

