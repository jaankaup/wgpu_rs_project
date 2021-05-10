# wgpu_rs_project

A small render/compute engine using wgpu-rs.

The main goal is to implement parallel fast marching method using wgpu-rs and
wgsl. The shaders are now implemented using GLSL because it seems that wgsl
doesn't yet support atomic types and operations. Finally (some day) the
projects are going to run on web browser.

This project is part of my master's thesis (parallel FMM). The code is now a
mess, and some day I will probably refactor and reogranize my code.

There are now only one "finished" test project using the engine.

The textures aren't mine. They are some random free textures downloaded from
the net. TODO: replace the old textures with new ones, and provide the download
links for those textures.

wood.obj is a slightly modivied version of
https://3dhaupt.com/wooden-watch-tower/. This is a test mesh for fast marching
method algorithm.

## Marching cubes slime project (test project)

The fist test scene a made. Just to get something on the screen.
You can compile and run the project as follows

$ cargo run --example hello_project 

On each frame, the density values of "slime" is computed and marching cubes is executed to produce a new triangle mesh.
Marching cubes creates ~500 000 triangles on each frame.

![hello_project](/pics/slime.png "The slime ocean.")

## Fast marching method (Under constrution. Do not compile)

This example is under contruction. Now the Belloch parallel prefix sum is
implemented and working. This is used for the global scan (find the blocks
where there are atleast one band point).

## Debug shaders (Under construction. Do not compile)

A logger for GPU state visualization. This is also under construction. gl_Pointsize is not available on wgsl. An instanced version must be developed. 
A custom fonts where implemented using compute shader and bezier curves.

![prefix sum](/pics/belloch_prefix_scan1.png "A screenshot of belloch prefix sum log visualization.")
![prefix sum2](/pics/belloch_prefix_scan2.png "Another screenshot of belloch prefix sum log visualization.")
![memory_model](/pics/memory_model.png "A visualization of memory model on GPU.")
