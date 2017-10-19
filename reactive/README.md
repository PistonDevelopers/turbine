# Design, Animate and Program Geometry

With other words: It is a type safe, dynamic, functional reactive library with homotopy maps.

This library uses ideas researched by Sven Nilsen of using homotopy maps to construct geometry.
Control points flags and ragdoll engine is based on Cutout Pro's Elemento.

### Motivation

This libray is intended for design, animating and programming geometry.

Function reactive programming is more fun when dynamic.
This means the user can modify the behavior at runtime.

The use of homotopy maps is a promising technique for constructing geometry.
A homotopy map is a continuous function that maps from N-dimension normalized coordinates
to some M-dimensional space.
See https://github.com/pistondevelopers/construct for more information.

The problem with dynamic frameworks for functional reactive programming
is that there often is a tradeoff with making it nice to use at compile time.
For example, when getting a reference to a spline,
it is easy to accidentally use it with another type, e.g. a surface.

By separating memory by function types, it is possible to
make the API type safe in Rust.

### Features (work in progress)

 - Functional graph evaluation
 - Environment with time and delta time (allows formal reasoning about animation and motion)
 - Homotopy maps up to 4D
 - Colors (converts sRGB to linear color space for internal use)
 - Ragdoll physics
 - Fine tuned control over caching
 - Control point selection and locking

### Goals

 - Create a modernized version of Cutout Pro's Elemento core for use in Turbine
 - Reusable in other game engines
 - Design for editing in VR
 - AI behavior trees (not decided yet)

### Design

All data is stored in memory by their function type.
This means one function can reference another function using an index,
while keeping type safety at compile time.

Objects are created by combining functions.
