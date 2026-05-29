# CatBox-RS
My attempt at a physics engine sandbox based on verlet integration. <br>
This is also a continuation of my Java attempt [CatBox](https://github.com/CoffeeCatRailway/CatBox), I decided to try remaking this in Rust for two reasons.
1. I ran into limitations using Java and got bored using it.
2. I discovered Rust and got fixated...

### Features:
- [x] Line Renderer for debug info
- [x] Camera controller
- [x] Imgui
- [x] ~~Primitive Shape Renderer (Circle, Box, Triangle-WIP)~~ Reworked into `Renderable`
- [x] Mesh builder for 3D
- [x] Textures
- [x] Environment controls (Gravity, Pause/Step, Step time or DT)
- [ ] Simple object (ball)
- [ ] Collide with world boundaries
- [ ] Separate solver thread (Maybe later)
- [ ] Collide with other objects
    - [ ] Sweep and Prune (5-18 fps)
    - [ ] Space partition (QuadTree or BSP, 15+ fps)
    - [ ] Combination
- [ ] Constraints (Fixed distance & Spring)
- [ ] Constraint collision (Box)
- [ ] Editor/Interface to interact with and add/remove objects
