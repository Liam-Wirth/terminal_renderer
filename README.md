
inspo/help
https://github.com/ryanweideman/3d-terminal-renderer/tree/main/src



[![wakatime](https://wakatime.com/badge/user/d40f8d42-5a14-4981-a36e-39f7bd209ef3/project/71a9c622-7c23-41f4-846d-dbab20a25231.svg)](https://wakatime.com/badge/user/d40f8d42-5a14-4981-a36e-39f7bd209ef3/project/71a9c622-7c23-41f4-846d-dbab20a25231)




# TODO
    - [ ] Decouple Rendering logic from the terminal output
    - [x] ~~Implement parallelism with rayon~~
    - [x] Implement more fleshed out "Tri" struct, possibly in own file -
        - [ ] After that, redefine the "Faces" of a mesh to actually just be a collection of Tris
    - [ ] Unify useage of Nalgebra functionality accross all files
        - Basically, I think I'm not using nalgebra to it's fullest potential and there's probably a lot of functionality I'm leaving "on the table"
    - [ ] Better document code, Rust is very much self-documenting, but it's good to explain things as I go as well
    - [ ] Beautify the links as markdown links for the "Links I looked at section"



## TODO (much less pressing/important)

    - [x] fix wakatime badge to update lmao
    - [ ] github ci/cd with clippy and rustfmt

## Things I should look into:
https://learnopengl.com/Guest-Articles/2021/Scene/Frustum-Culling <- Frustum Culling


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
- [https://www.desmos.com/3d/8t6uvxkyh6][Desmos Graph for Dodecahedron]
- https://www.youtube.com/watch?v=Hqi8QREXwrE

## Other cool projects! (that ended up helping me out :)
| Link | Comments/Notes      |
| ------------- | ------------- |
| https://github.com/ryanweideman/3d-terminal-renderer/tree/main |  Of course, I wasn't the only person to try and build this project in rust, and someone much smarter than me did something that ended up being pretty simmilar. About 16 hours in I stumbled across that first project, and it was really interesting and insightful to see what someone else did. Kudos to them!|
| https://github.com/TermTrack/TermTrack | Super cool project to see, and is awesome to see a much more performant/realtime representation of rendering using crossterm|
| https://github.com/JasondeWolff/rusterizer| |
|https://github.com/ecumene/rust-sloth |



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

