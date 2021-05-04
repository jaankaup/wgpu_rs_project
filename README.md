# wgpu_rs_project

A small render/compute engine using wgpu-rs.

The main goal is to implement parallel fast marching method using wgpu-rs and wgsl. The shaders are now implemented using GLSL 
because it seems that wgsl doesn't yet support atomic types and operations. Finally (some day) the projects are going to run on web browser.

There are now only one "finished" test project using the engine. 

## Marching cubes slime project (test project)

You can compile and run the project as follows

$ cargo run --example hello_project 

On each frame, the density values of "slime" is computed and marching cubes is run to create triangle a new triangle mesh of slime.
Marching cubes creates ~500 000 triangles on each frame.
