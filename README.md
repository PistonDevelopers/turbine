# Turbine

3D game engine for content production

Content production can be described from two perspectives:

- Production perspective
- Content perspective

### Production perspective: Content Production vs Game Engine

In Game Development, the word "game engine" is used in different ways, often with overlapping meaning:

- A set of libraries used to make a game
- The tools used to make a game
- The framework that integrates libraries in a coherent ecosystem
- The parts of gaming software that is not game specific

In the context of Content Production, the game engine is oriented toward the parts of production
software that is not game specific, usually a mixed bias of two orthogonal concepts:

- Project oriented Content Production (multiple tools)
- Tool oriented Content Production (multiple projects)

A game engine for Content Production is a software system that integrates tools and projects with the pipelines, workflows and human guided creative processes.

This usage of "game engine" can differ from the more strict usage as a set of libraries used to make a game.

In the Turbine project, there is overlapping meaning a set of libraries and a game engine in the sense of Content Production.

### Content perspective: Content Production vs Data

Content Production is an umbrella term for all processes
that produce content in some project,
typically in Animation or Game Development:

- Assets (e.g. 3D models, sound effects)
- Pipelines (e.g. from 3D editor to game engine)
- Scripts (e.g. procedural generation)
- Workflows (e.g. design iterations)

For example, in Animation, Content Production means
the processes that produce storyboard, animation,
voice acting, sound, music, compositing and rendering.

Content Production is characterized by the property that
the result is usually data that can be used, modified or
utilized in multiple ways that fit in a project.

Some tools or temporary assets used during Content Production might not be part of the final result,
but used to speed up design iterations.

Requirements to performance and flexibility can differ.
This can depend on the specifics of projects or tools.

### Design

Reexports:

- [turbine_process3d](https://github.com/PistonDevelopers/turbine/tree/master/process3d)
- [turbine_reactive](https://github.com/PistonDevelopers/turbine/tree/master/reactive)
- [turbine_scene3d](https://github.com/PistonDevelopers/turbine/tree/master/scene3d)
