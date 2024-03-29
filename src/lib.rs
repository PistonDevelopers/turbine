#![deny(missing_docs)]

//! A 3D game engine with built-in editor.

#[macro_use]
extern crate bitflags;
extern crate camera_controllers;
extern crate gfx;
extern crate gfx_debug_draw;
extern crate gfx_device_gl;
extern crate gfx_text;
extern crate piston_meta;
extern crate piston_window;
extern crate sdl2_window;
extern crate vecmath;
#[macro_use]
extern crate log;
extern crate range;
#[macro_use]
extern crate conrod;
extern crate find_folder;

pub use math::Mat4;
pub use math::Ray;
pub use math::Vec3;
pub use math::AABB;

pub use world::World;

pub mod data;
pub mod logger;
pub mod math;
pub mod render;
pub mod world;

widget_ids! {
    #[allow(missing_docs)]
    pub struct Ids {
        refresh,
    }
}

/// Starts Turbine pointing it to a project folder.
pub fn start(project_folder: &str) {
    use camera_controllers::*;
    use piston_window::*;
    use sdl2_window::Sdl2Window;
    // use conrod::{ Ui, Theme };
    // use gfx_debug_draw::DebugRenderer;
    use crate::math::Matrix;

    println!(
        "
~~~~~~~~   TURBINE   ~~~~~~~~\n\
=============================\n\
Camera navigation (on/off): C\n\
Camera control: WASD\n\
"
    );

    let mut window: PistonWindow<Sdl2Window> = WindowSettings::new("Turbine", [1024, 768])
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
            fov,
            near_clip: near,
            far_clip: far,
            aspect_ratio: (draw_size.width as f32) / (draw_size.height as f32),
        }
        .projection()
    };

    let mut projection = get_projection(window.draw_size());
    let mut first_person = FirstPerson::new([0.5, 0.5, 4.0], FirstPersonSettings::keyboard_wasd());

    // TODO: Update debug renderer.
    /*
    let mut debug_renderer = {
        let text_renderer =
            gfx_text::new(window.factory.borrow().clone()).unwrap();
        DebugRenderer::new(window.factory.borrow().clone(),
            text_renderer, 64).ok().unwrap()
    };
    */

    let mut cursor_pos = [0.0, 0.0];
    let mut ground_pos = [0.0, 0.0, 0.0];
    let mut ortho = false;

    // TODO: Update Conrod.
    /*
    let mut ui = {
        let assets = find_folder::Search::ParentsThenKids(3, 3)
            .for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        let mut theme = Theme::default();
        theme.font_size_medium = 12;
        let glyph_cache = Glyphs::new(&font_path, window.factory.borrow().clone());
        Ui::new(glyph_cache.unwrap(), theme)
    };
    */

    let mut world = World::new();

    {
        // Create folders if they do not exists.
        data::create_folders(project_folder).unwrap();

        // Load entities.
        let files = data::entities::files(project_folder).unwrap();
        data::entities::load(&mut world, &files).unwrap();
    }

    while let Some(e) = window.next() {
        if capture_cursor {
            first_person.event(&e);
        }

        window.draw_3d(&e, |window| {
            let draw_size = window.draw_size();
            let draw_size = [draw_size.width, draw_size.height];
            let args = e.render_args().unwrap();
            let camera = first_person.camera(args.ext_dt);
            let mvp = model_view_projection(Matrix::id(), camera.orthogonal(), projection);

            window
                .encoder
                .clear(&window.output_color, [0.3, 0.3, 0.3, 1.0]);
            window.encoder.clear_depth(&window.output_stencil, 1.0);
            window.encoder.clear_stencil(&window.output_stencil, 0);

            // TODO: Update debug renderer.
            /*
            render::entity_current_positions(&world, &mut debug_renderer);

            render::axes(&camera, projection, draw_size, &mut debug_renderer);
            let yellow = [1.0, 1.0, 0.0, 1.0];
            debug_renderer.draw_marker(ground_pos, 0.1, yellow);
            debug_renderer.render(stream, mvp).unwrap();
            */
        });
        if capture_cursor {
            // Send render events to make Conrod update window size.
            if e.render_args().is_some() {
                // TODO: Update Conrod.
                // ui.handle_event(&e);
            }
        } else {
            // use conrod::*;

            // TODO: Update Conrod.
            // ui.handle_event(&e);
            /*
            e.update(|_| ui.set_widgets(|ui| {
                Button::new()
                    .color(color::BLUE)
                    .top_left()
                    .w_h(60.0, 30.0)
                    .label("refresh")
                    .react(|| {})
                    .set(REFRESH, ui);
            }));
            */
            window.draw_2d(&e, |c, g| {
                // TODO: Update Conrod.
                // ui.draw(c, g);
            });
        }
        e.resize(|_, _| {
            if !ortho {
                projection = get_projection(window.draw_size());
            }
        });
        if let Some(pos) = e.mouse_cursor_args() {
            cursor_pos = pos;
        }
        if let Some(Button::Keyboard(Key::C)) = e.press_args() {
            capture_cursor = !capture_cursor;
            window.set_capture_cursor(capture_cursor);
        }
        if let Some(Button::Keyboard(Key::O)) = e.press_args() {
            ortho = !ortho;
            if ortho {
                projection = Matrix::id();
            } else {
                projection = get_projection(window.draw_size());
            }
        }
        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            if !capture_cursor {
                let draw_size = window.draw_size();
                let draw_size = [draw_size.width, draw_size.height];
                let ray = Ray::from_2d(cursor_pos, draw_size, fov, near, far);
                let view_to_world = first_person.camera(0.0).orthogonal().inv();
                match view_to_world.ray(ray).ground_plane() {
                    None => info!("Click on the ground to add entity"),
                    Some(pos) => {
                        ground_pos = pos;
                        world.add_entity(pos);
                    }
                }
            }
        }
    }

    {
        // Save entities data.
        let entities_folder = data::entities::folder(project_folder);
        data::entities::save(&world, entities_folder).unwrap();
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use piston_meta::*;

    #[test]
    fn entity_syntax() {
        let _ = load_syntax_data("assets/entity/syntax.txt", "assets/entity/test-cube.txt");
    }
}
