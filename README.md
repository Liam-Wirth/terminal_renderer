
inspo/help
https://github.com/ryanweideman/3d-terminal-renderer/tree/main/src



[![wakatime](https://wakatime.com/badge/user/d40f8d42-5a14-4981-a36e-39f7bd209ef3/project/71a9c622-7c23-41f4-846d-dbab20a25231.svg)](https://wakatime.com/badge/user/d40f8d42-5a14-4981-a36e-39f7bd209ef3/project/71a9c622-7c23-41f4-846d-dbab20a25231)

[https://www.desmos.com/3d/8t6uvxkyh6][Desmos Graph for Dodecahedron]



# TODO
- [ ] Decouple Rendering logic from the terminal output
- [ ] Implement parallelism with rayon
- [ ] Implement more fleshed out "Tri" struct, possibly in own file
    - [ ] After that, redefine the "Faces" of a mesh to actually just be a collection of Tris
- [ ] Unify useage of Nalgebra functionality accross all files
    - Basically, I think I'm not using nalgebra to it's fullest potential and there's probably a lot of functionality I'm leaving "on the table"
- [ ] Better document code, Rust is very much self-documenting, but it's good to explain things as I go as well
- [ ] Beautify the links as markdown links for the "Links I looked at section"



# TODO (much less pressing/important)

- [ ] fix wakatime badge to update lmao
- [ ] github ci/cd with clippy and rustfmt



# Credit/Helped along the way
I had a feeling I wasn't the only person to ever try to build a 3d renderer in the terminal, and so whenever I used an online resource In the code comments I put a link above the function or something to the relevant page I used

Of course, I wasn't the only person to try and build this project in rust, and someone much smarter than me did something that ended up being pretty simmilar. About 16 hours in I stumbled across this:
https://github.com/ryanweideman/3d-terminal-renderer/tree/main

and it was helpful/cool to peruse the code and see how someone smarter/more experienced than me implemented some stuff. Made some decisions that I didn't make, etc.
Was a nice resource to look at, and helped at times cause I've never taken a linear algebra class and all of this stuff is very new to me





## Links I Looked at:
https:en.wikipedia.org/wiki/Curve_orientation#Orientation_of_a_simple_polygon <- Backface Culling?
https:groups.csail.mit.edu/graphics/classes/6.837/F98/Lecture7/triangles.html <- Triangle Filling

