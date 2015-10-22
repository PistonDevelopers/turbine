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

pub mod math;
pub mod render;
pub mod logger;

/// The maximum number of entities.
pub const ENTITY_COUNT: usize = 10000;

bitflags!(
    /// Used to turn on/off components per entity.
    flags Mask: u32 {
        /// Entity is alive.
        const ALIVE         = 0b00000001,
        /// Entity is selected.
        const SELECT        = 0b00000010,
        /// Entity has an AABB.
        const AABB          = 0b00000100,
    }
);

/// Stores physical state.
pub struct Physics {
    /// The position.
    pub position: [Vec3; ENTITY_COUNT],
}

impl Physics {
    /// Gets next linear step.
    pub fn step(&mut self, prev: &Physics, current: &Physics) {
        use math::Vector;

        for i in 0..ENTITY_COUNT {
            // current + (current - prev) = 2 * current - prev.
            self.position[i] = current.position[i]
                .scale(2.0)
                .sub(prev.position[i]);
        }
    }
}

/// Stores the world data.
pub struct World {
    /// The active components per entity.
    pub mask: [Mask; ENTITY_COUNT],
    /// The initial state of physics.
    pub init: Box<Physics>,
    /// The previous state.
    pub prev: Box<Physics>,
    /// The current state.
    pub current: Box<Physics>,
    /// The next state.
    pub next: Box<Physics>,
    /// An AABB relative to position.
    pub aabb: [AABB; ENTITY_COUNT],
}

impl World {
    /// Swaps the physical state such that previous is now next.
    pub fn swap_physics(&mut self) {
        use std::mem::swap;

        swap(&mut self.prev, &mut self.current);
        swap(&mut self.current, &mut self.next);
    }

    /// Finds the first free entity slot.
    pub fn find_free_entity_slot(&self) -> Option<usize> {
        for i in 0..ENTITY_COUNT {
            if self.mask[i].is_empty() { return Some(i); }
        }
        None
    }

    /// Sets position with no velocity.
    pub fn set_position_with_no_velocity(&mut self, id: usize, pos: Vec3) {
        self.prev.position[id] = pos;
        self.current.position[id] = pos;
        self.next.position[id] = pos;
    }

    /// Adds a new entity.
    /// Marks the entity as alive and selects it.
    pub fn add_entity(&mut self, pos: Vec3) {
        let id = match self.find_free_entity_slot() {
            Some(id) => id,
            None => {
                warn!("There are no free entity slots");
                return;
            }
        };
        self.init.position[id] = pos;
        self.set_position_with_no_velocity(id, pos);
        let mask = &mut self.mask[id];
        mask.insert(ALIVE);
        mask.insert(SELECT);
    }
}

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

    let get_projection = |w: &PistonWindow<(), Sdl2Window>| {
        let draw_size = w.window.borrow().draw_size();
        CameraPerspective {
            fov: 90.0, near_clip: 0.1, far_clip: 1000.0,
            aspect_ratio: (draw_size.width as f32) / (draw_size.height as f32)
        }.projection()
    };

    let mut projection = get_projection(&window);
    let mut first_person = FirstPerson::new(
        [0.5, 0.5, 4.0],
        FirstPersonSettings::keyboard_wasd()
    );

    let mut debug_renderer = {
        let text_renderer = {
            gfx_text::new(window.factory.borrow().clone()).unwrap()
        };
        DebugRenderer::new(window.factory.borrow().clone(),
            text_renderer, 64).ok().unwrap()
    };

    let mut cursor_pos = [0.0, 0.0];
    let mut ground_pos = [0.0, 0.0, 0.0];

    for mut e in window {
        if capture_cursor {
            first_person.event(&e);
        }
        e.draw_3d(|stream| {
            let args = e.render_args().unwrap();
            let mvp = model_view_projection(
                Matrix::id(),
                first_person.camera(args.ext_dt).orthogonal(),
                projection
            );
            render::clear(stream);
            render::axes(&mut debug_renderer);
            debug_renderer.draw_marker(ground_pos, 0.1, [1.0, 0.0, 0.0, 1.0]);
            debug_renderer.render(stream, mvp).unwrap();
        });
        e.resize(|_, _| {
            projection = get_projection(&e);
        });
        if let Some(pos) = e.mouse_cursor_args() {
            cursor_pos = pos;

            if !capture_cursor {
                let draw_size = e.draw_size();
                let draw_size = [draw_size.width, draw_size.height];
                let cursor_pos = Vector::from_2d(cursor_pos, draw_size);
                let ray = Ray { pos: cursor_pos, dir: Vector::forward() };
                let mvp = model_view_projection(
                    Matrix::id(),
                    first_person.camera(0.0).orthogonal(),
                    projection
                );
                let ray = mvp.inv().ray(ray);
                match ray.ground_plane() {
                    None => info!("Click on the ground to add entity"),
                    Some(pos) => {
                        ground_pos = pos;
                        println!("TEST add position {:?}", pos);
                    }
                }
            }
        }
        if let Some(Button::Keyboard(Key::C)) = e.press_args() {
            capture_cursor = !capture_cursor;
            e.set_capture_cursor(capture_cursor);
        }
        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {

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
