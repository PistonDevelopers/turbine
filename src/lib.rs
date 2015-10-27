#![deny(missing_docs)]

//! A 3D game engine with built-in editor.

#[macro_use]
extern crate bitflags;
extern crate vecmath;
extern crate piston_window;
extern crate sdl2_window;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate piston_meta;
extern crate camera_controllers;
extern crate gfx_debug_draw;
extern crate gfx_text;
#[macro_use]
extern crate log;

pub use math::Vec3;
pub use math::Mat4;
pub use math::AABB;
pub use math::Ray;

pub use world::World;

pub mod math;
pub mod render;
pub mod logger;
pub mod world;

/// Starts Turbine pointing it to a project folder.
pub fn start(_project_folder: &str) {
    use piston_window::*;
    use sdl2_window::Sdl2Window;
    use camera_controllers::*;
    use gfx_debug_draw::DebugRenderer;
    use math::{ Matrix, Vector };

    println!("
~~~~~~~~   TURBINE   ~~~~~~~~\n\
=============================\n\
Camera navigation (on/off): C\n\
Camera control: WASD\n\
");

    let mut window: PistonWindow<(), Sdl2Window> =
        WindowSettings::new("Turbine", [1024, 768])
        .exit_on_esc(true)
        .samples(4)
        .build()
        .unwrap();

    logger::init().unwrap();

    let mut capture_cursor = true;

    window.set_capture_cursor(capture_cursor);

    let fov = 90.0;
    let near = 0.1;
    let far = 1000.0;
    let get_projection = |draw_size: Size| {
        CameraPerspective {
            fov: fov, near_clip: near, far_clip: far,
            aspect_ratio: (draw_size.width as f32) / (draw_size.height as f32)
        }.projection()
    };

    let mut projection = get_projection(window.draw_size());
    let mut first_person = FirstPerson::new(
        [0.5, 0.5, 4.0],
        FirstPersonSettings::keyboard_wasd()
    );

    let mut debug_renderer = {
        let text_renderer =
            gfx_text::new(window.factory.borrow().clone()).unwrap();
        DebugRenderer::new(window.factory.borrow().clone(),
            text_renderer, 64).ok().unwrap()
    };

    let mut cursor_pos = [0.0, 0.0];
    let mut ground_pos = [0.0, 0.0, 0.0];
    let mut ortho = false;

    for mut e in window {
        if capture_cursor {
            first_person.event(&e);
        }
        e.draw_3d(|stream| {
            let draw_size = e.draw_size();
            let draw_size = [draw_size.width, draw_size.height];
            let args = e.render_args().unwrap();
            let camera = first_person.camera(args.ext_dt);
            let mvp = model_view_projection(
                Matrix::id(),
                camera.orthogonal(),
                projection
            );
            render::clear(stream);

            render::axes(&camera, projection, draw_size, &mut debug_renderer);
            debug_renderer.draw_marker(ground_pos, 0.1, [1.0, 1.0, 0.0, 1.0]);
            debug_renderer.render(stream, mvp).unwrap();
        });
        e.resize(|_, _| {
            if !ortho {
                projection = get_projection(e.draw_size());
            }
        });
        if let Some(pos) = e.mouse_cursor_args() {
            cursor_pos = pos;
        }
        if let Some(Button::Keyboard(Key::C)) = e.press_args() {
            capture_cursor = !capture_cursor;
            e.set_capture_cursor(capture_cursor);
        }
        if let Some(Button::Keyboard(Key::O)) = e.press_args() {
            ortho = !ortho;
            if ortho {
                projection = Matrix::id();
            } else {
                projection = get_projection(e.draw_size());
            }
        }
        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            if !capture_cursor {
                let draw_size = e.draw_size();
                let draw_size = [draw_size.width, draw_size.height];
                let ray = Ray::from_2d(cursor_pos, draw_size, fov, near, far);
                let view_to_world = first_person.camera(0.0).orthogonal().inv();
                match view_to_world.ray(ray).ground_plane() {
                    None => info!("Click on the ground to add entity"),
                    Some(pos) => {
                        ground_pos = pos;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use piston_meta::*;

    #[test]
    fn entity_syntax() {
        let _ = load_syntax_data2("assets/entity/syntax.txt",
            "assets/entity/test-cube.txt");
    }
}
