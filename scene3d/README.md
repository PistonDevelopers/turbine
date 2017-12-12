# Turbine-Scene3D

Scene rendering for the Turbine game engine.

<video width="320" height="240" controls>
  <source src="https://i.imgur.com/M0frz9B.mp4" type="video/mp4">
Your browser does not support the video tag.
</video>

### Design

- Scene object stores all resources used for rendering
- Frame graph stores command lists

This design allows flexible programming of scenes, without the need for
a tree structure to store nodes for scene data.
The frame graph can be used to debug the scene.
