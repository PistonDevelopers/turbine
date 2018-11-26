extern crate piston;
extern crate sdl2_window;
extern crate turbine_scene3d;
extern crate vecmath;
extern crate camera_controllers;

use piston::window::*;
use piston::event_loop::*;
use piston::input::RenderEvent;
use sdl2_window::Sdl2Window;
use turbine_scene3d::*;
use turbine_scene3d::Command::*;
use vecmath::*;
use camera_controllers::*;

fn main() {
    let settings = WindowSettings::new("colored cube", [512; 2])
        .samples(4)
        .exit_on_esc(true);
    let mut window: Sdl2Window = settings.build().unwrap();
    window.set_capture_cursor(true);
    let mut events = Events::new(EventSettings::new());

    let mut scene = Scene::new(SceneSettings::new());
    let mut frame_graph = FrameGraph::new();

    let cube = {
        let vertex_shader = scene.vertex_shader(include_str!("../assets/colored_cube.glslv"))
            .unwrap();
        let fragment_shader = scene.fragment_shader(include_str!("../assets/colored_cube.glslf"))
            .unwrap();
        let program = scene.program_from_vertex_fragment(vertex_shader, fragment_shader);
        let mvp = scene.matrix4_uniform(program, "MVP").unwrap();

        let vertex_array = scene.vertex_array();
        let vertex_buffer = scene.vertex_buffer3(vertex_array, 0, &vertex_buffer_data());
        let _ = scene.color_buffer(vertex_array, 1, &color_buffer_data());

        frame_graph.command_list(vec![
            EnableCullFace,
            CullFaceBack,
            UseProgram(program),
            SetModelViewProjection(mvp),
            DrawTriangles(vertex_array, vertex_buffer.len()),
        ])
    };

    let cubes = frame_graph.command_list(vec![
            Scale([1.0, 1.0, 1.0]),
            Draw(cube),
            Translate([2.5, 0.0, 0.0]),
            RotateAxisDeg(vec3_normalized([1.0, 0.0, 1.0]), 45.0),
            Draw(cube)
        ]);

    let mut first_person = FirstPerson::new(
        [0.5, 0.5, 4.0],
        FirstPersonSettings::keyboard_wasd()
    );

    while let Some(e) = events.next(&mut window) {
        first_person.event(&e);

        if let Some(args) = e.render_args() {
            scene.projection = get_projection(&window);
            scene.camera = first_person.camera(args.ext_dt).orthogonal();
            scene.model = mat4_id();
            scene.clear([0.0, 0.0, 0.0, 1.0]);
            scene.scale([1.0, 1.0, 1.0]);

            scene.draw(cubes, &frame_graph);
        }
    }
}

fn get_projection<W: Window>(w: &W) -> Matrix4<f32> {
    let draw_size = w.draw_size();
    CameraPerspective {
        fov: 90.0, near_clip: 0.1, far_clip: 1000.0,
        aspect_ratio: (draw_size.width as f32) / (draw_size.height as f32)
    }.projection()
}


fn vertex_buffer_data() -> Vec<f32> {
    vec![
        -1.0,   -1.0,   -1.0, // triangle 1 : begin
        -1.0,   -1.0,   1.0,
        -1.0,   1.0,    1.0, // triangle 1 : end

        1.0,    1.0,    -1.0, // triangle 2 : begin
        -1.0,   -1.0,   -1.0,
        -1.0,   1.0,    -1.0, // triangle 2 : end

        1.0,    -1.0,   1.0,
        -1.0,   -1.0,   -1.0,
        1.0,    -1.0,   -1.0,

        1.0,    1.0,    -1.0,
        1.0,    -1.0,   -1.0,
        -1.0,   -1.0,   -1.0,

        -1.0,   -1.0,   -1.0,
        -1.0,   1.0,    1.0,
        -1.0,   1.0,    -1.0,

        1.0,    -1.0,   1.0,
        -1.0,   -1.0,   1.0,
        -1.0,   -1.0,   -1.0,

        -1.0,   1.0,    1.0,
        -1.0,   -1.0,   1.0,
        1.0,    -1.0,   1.0,

        1.0,    1.0,    1.0,
        1.0,    -1.0,   -1.0,
        1.0,    1.0,    -1.0,

        1.0,    -1.0,   -1.0,
        1.0,    1.0,    1.0,
        1.0,    -1.0,   1.0,

        1.0,    1.0,    1.0,
        1.0,    1.0,    -1.0,
        -1.0,   1.0,    -1.0,

        1.0,    1.0,    1.0,
        -1.0,   1.0,    -1.0,
        -1.0,   1.0,    1.0,

        1.0,    1.0,    1.0,
        -1.0,   1.0,    1.0,
        1.0,    -1.0,   1.0
    ]
}

fn color_buffer_data() -> Vec<f32> {
    vec![
        0.583,  0.771,  0.014,
        0.609,  0.115,  0.436,
        0.327,  0.483,  0.844,
        0.822,  0.569,  0.201,
        0.435,  0.602,  0.223,
        0.310,  0.747,  0.185,
        0.597,  0.770,  0.761,
        0.559,  0.436,  0.730,
        0.359,  0.583,  0.152,
        0.483,  0.596,  0.789,
        0.559,  0.861,  0.639,
        0.195,  0.548,  0.859,
        0.014,  0.184,  0.576,
        0.771,  0.328,  0.970,
        0.406,  0.615,  0.116,
        0.676,  0.977,  0.133,
        0.971,  0.572,  0.833,
        0.140,  0.616,  0.489,
        0.997,  0.513,  0.064,
        0.945,  0.719,  0.592,
        0.543,  0.021,  0.978,
        0.279,  0.317,  0.505,
        0.167,  0.620,  0.077,
        0.347,  0.857,  0.137,
        0.055,  0.953,  0.042,
        0.714,  0.505,  0.345,
        0.783,  0.290,  0.734,
        0.722,  0.645,  0.174,
        0.302,  0.455,  0.848,
        0.225,  0.587,  0.040,
        0.517,  0.713,  0.338,
        0.053,  0.959,  0.120,
        0.393,  0.621,  0.362,
        0.673,  0.211,  0.457,
        0.820,  0.883,  0.371,
        0.982,  0.099,  0.879
    ]
}
