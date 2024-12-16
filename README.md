
[![wakatime](https://wakatime.com/badge/user/d40f8d42-5a14-4981-a36e-39f7bd209ef3/project/71a9c622-7c23-41f4-846d-dbab20a25231.svg)](https://wakatime.com/badge/user/d40f8d42-5a14-4981-a36e-39f7bd209ef3/project/71a9c622-7c23-41f4-846d-dbab20a25231)

# Rewrite/Refactor Count
## 0.1: Initial Write <look through commit history and put date here>
  - Very buggy, slow, and due to poor optimization, fundamentally incapable of rendering pretty simple models like "suzanne" at a framerate greater than 1
  - Learned a lot
## 0.2: <start> - 11-12-2024: Version 0.2, first rewrite
  - Much faster in terms of code written, pipeline was still unclear, but the renderer was much more capable
  - Could render the Teapot w/ roughly 6.5k tris at 50fps in the terminal, applying transformations
  - Never got triangles to fill properly but ultimately abandoned this version due to an unclear pipeline that was hard to maintain
  - TLDR, code was a great start, but disorganized and needed more clear structure
  - v: 0.3 will likely include a lot of core logic written at this stage
  ### Visualized Pipeline:
  ```mermaid
  flowchart TD
      A[Scene] --> B[Entity Loop]
      B --> C[Transform/Update]
      C --> D[Project Vertices]
      D --> E[Triangle Processing]
      E --> F[Rasterization]
      F --> G[Buffer Management]
  ```

## 0.3: Third Time's A Charm: Start: 11th December 2024
  - Goals:
  - clear pipeline
  - everything else




# TODO
- [ ] Decouple Rendering logic from the terminal output
- [ ] Better document code, Rust is very much self-documenting, but it's good to explain things as I go as well
- [ ] Beautify the links as markdown links for the "Links I looked at section"
- [ ] migrate to possibly just using the mesh with tobj?
- [ ] OR Finalize mesh struct, implementing materials
- [ ] Lighting
- [ ] Shadows

## TODO: Clipping
- [ ] Bounding boxes for entities/meshes
- [ ] full frustum culling
- [ ] use bounding boxes in frustum culling
- [ ] Clip space operations on faces/primatives instead of on vertices
- [ ] Early rejection of faces and objects that are outside of the frustum



# TODO (done) (just for my own reference)
- [x] ~~Implement parallelism with rayon
- [x] Implement more fleshed out "Tri" struct, possibly in own file -
- [x] After that, redefine the "Faces" of a mesh to actually just be a collection of Tris
- [x] ~~fix wakatime badge to update lmao~~

## TODO (much less pressing/important)
- [ ] github ci/cd with clippy and rustfmt

## Things I should look into:
https://learnopengl.com/Guest-Articles/2021/Scene/Frustum-Culling <- Frustum Culling
https://github.com/ryanweideman/3d-terminal-renderer/tree/main/src


# Credit/Helped along the way

I had a feeling I wasn't the only person to ever try to build a 3d renderer in the terminal,
and so whenever I used an online resource In the code comments I put a link above the function or something to the relevant page I used
I've done my best to agregate that all here, and try and organize it!


and it was helpful/cool to peruse the code and see how someone smarter/more experienced than me implemented some stuff. Made some decisions that I didn't make, etc.
Was a nice resource to look at, and helped at times cause I've never taken a linear algebra class and all of this stuff is very new to me


## Links I Looked at:
Articles, Extremely old lecture material, Wikipedia Pages, etc, all of these played a hand in the project!
- https://en.wikipedia.org/wiki/Curve_orientation#Orientation_of_a_simple_polygon <- Backface Culling?
- https://groups.csail.mit.edu/graphics/classes/6.837/F98/Lecture7/triangles.html <- Triangle Filling
- https://en.wikipedia.org/wiki/Viewing_frustum
- https://en.wikipedia.org/wiki/Back-face_culling
- https://en.wikipedia.org/wiki/3D_projection
- https://en.wikipedia.org/wiki/Clip_coordinates
- [https://www.desmos.com/3d/8t6uvxkyh6][Desmos Graph for Dodecahedron]
- https://www.youtube.com/watch?v=Hqi8QREXwrE
- https://en.wikipedia.org/wiki/Scanline_rendering
- https://www.geometrictools.com/Documentation/ClipMesh.pdf
- https://computergraphics.stackexchange.com/questions/9463/bounding-box-of-a-rotating-mesh
- https://gamedev.stackexchange.com/questions/159511/how-can-i-generate-the-smallest-enclosing-sphere-from-a-mesh
- https://www.sunshine2k.de/coding/java/Welzl/Welzl.html
- https://www.sunshine2k.de/coding_java.html   <- Good amount of rasterization algorithms/implementations here
- https://github.com/ssloy/tinyrenderer/wiki
- https://en.wikipedia.org/wiki/Polygon_mesh
- https://en.wikipedia.org/wiki/Triangle_mesh


## Other cool projects! (that ended up helping me out :)
| Link | Comments/Notes      |
| ------------- | ------------- |
| https://github.com/ryanweideman/3d-terminal-renderer/tree/main |  Of course, I wasn't the only person to try and build this project in rust, and someone much smarter than me did something that ended up being pretty simmilar. About 16 hours in I stumbled across that first project, and it was really interesting and insightful to see what someone else did. Kudos to them!|
| https://github.com/TermTrack/TermTrack | Super cool project to see, and is awesome to see a much more performant/realtime representation of rendering using crossterm|
| https://github.com/JasondeWolff/rusterizer| |
|https://github.com/ecumene/rust-sloth |
|http://www.blitzcode.net/3d_1.shtml#Software_Rasterizer| ex-nvidia employee had some fun with rust|
| https://medium.com/@aminere/software-rendering-from-scratch-f60127a7cd58 | Useful to look at if I go about another another rewrite|
|https://jamesgisele.com/blog/software_renderer/#list| this was helpful|


# NOTES on pipeline:
Summary of Key Pipeline Stages

Modeling Transformations (Object → World)
Viewing Transformations (World → Camera)
Projection (Camera → Screen)
Rasterization (Convert triangles to pixels)
Depth Testing and Shading (Determine visible surfaces and color them)
Frame Buffering and Output (Store and render the final image)

This pipeline can be optimized by focusing on the most computationally expensive stages, such as rasterization and shading.



# New "Pipeline" as it exists in my sick and twisted mind:
Modeling -> Transformations -> Viewing Projection -> Chunking buffers for rasterization -> Rasterization -> Depth Checks -> Frame Buffer -> Overlay Hud -> Output






code needs to have global state in which it's values are initialized once, and are simply mutated/updated on each frame


TermRenderer:
    pub buffer: Buffer <- Possibly with some sorta mutex lock thingy mabob on it, or just atomics
    pub camera: Camera
    pub scene: Scene <- Entities, Lights (eventually), etc
    pub hud: Hud (TODO!)
    pub chunks: Vec<Buffer> <-
