//! # Design, Animate and Program Geometry
//!
//! With other words: It is a type safe, dynamic, functional reactive library with homotopy maps.
//!
//! This library uses ideas researched by Sven Nilsen of using homotopy maps to construct geometry.
//! Control points flags and ragdoll engine is based on Cutout Pro's Elemento.
//!
//! ### Motivation
//!
//! This libray is intended for design, animating and programming geometry.
//!
//! Function reactive programming is more fun when dynamic.
//! This means the user can modify the behavior at runtime.
//!
//! The use of homotopy maps is a promising technique for constructing geometry.
//! A homotopy map is a continuous function that maps from N-dimension normalized coordinates
//! to some M-dimensional space.
//! See https://github.com/pistondevelopers/construct for more information.
//!
//! The problem with dynamic frameworks for functional reactive programming
//! is that there often is a tradeoff with making it nice to use at compile time.
//! For example, when getting a reference to a spline,
//! it is easy to accidentally use it with another type, e.g. a surface.
//!
//! By separating memory by function types, it is possible to
//! make the API type safe in Rust.
//!
//! ### Features (work in progress)
//!
//! - Functional graph evaluation
//! - Environment with time and delta time (allows formal reasoning about animation and motion)
//! - Homotopy maps up to 4D
//! - Colors (converts sRGB to linear color space for internal use)
//! - Ragdoll physics
//! - Fine tuned control over caching
//! - Control point selection and locking
//!
//! ### Goals
//!
//! - Create a modernized version of Cutout Pro's Elemento core for use in Turbine
//! - Reusable in other game engines
//! - Design for editing in VR
//! - AI behavior trees (not decided yet)
//!
//! ### Design
//!
//! All data is stored in memory by their function type.
//! This means one function can reference another function using an index,
//! while keeping type safety at compile time.
//!
//! Objects are created by combining functions.

#![deny(missing_docs)]

extern crate vecmath;
extern crate read_color;
extern crate fnv;

use vecmath::traits::{Cast, Float};

pub use types::*;
pub use cache::Cache;
pub use selection::Selection;
pub use physics::Physics;

pub mod fns;
pub mod ptr;
pub mod color;

mod types;
mod cache;
mod selection;
mod physics;

// Impl methods for `Reactor`.
mod index;
mod eval;
mod add;

/// Stores data for an reactive document.
pub struct Reactor<T> {
    /// Stores bools.
    pub bools: Vec<fns::Bool<T>>,
    /// Stores scalars.
    pub points1: Vec<fns::Point1<T>>,
    /// Stores 2D points.
    pub points2: Vec<fns::Point2<T>>,
    /// Stores 3D points.
    pub points3: Vec<fns::Point3<T>>,
    /// Stores 4D points.
    pub points4: Vec<fns::Point4<T>>,
    /// Stores splines for scalars.
    pub splines1: Vec<Spline1<T>>,
    /// Stores splines for 2D points.
    pub splines2: Vec<Spline2<T>>,
    /// Stores splines for 3D points.
    pub splines3: Vec<Spline3<T>>,
    /// Stores splines for 4D points.
    pub splines4: Vec<Spline4<T>>,
    /// Stores surface for 1D.
    pub surfaces1: Vec<Surface1<T>>,
    /// Stores surfaces for 2D.
    pub surfaces2: Vec<Surface2<T>>,
    /// Stores surfaces for 3D.
    pub surfaces3: Vec<Surface3<T>>,
    /// Stores surface for 4D.
    pub surfaces4: Vec<Surface4<T>>,
    /// Stores colors.
    pub colors: Vec<fns::Color<T>>,
    /// Stores color splines.
    pub color_splines: Vec<fns::ColorSpline>,
    /// Stores bones between scalars.
    pub bones1: Vec<Bone1<T>>,
    /// Stores bones between 2D points.
    pub bones2: Vec<Bone2<T>>,
    /// Stores bones between 3D points.
    pub bones3: Vec<Bone3<T>>,
    /// Stores bones between 4D points.
    pub bones4: Vec<Bone4<T>>,
    /// Refers to time for reuse.
    /// This is also used to detect whether document is animated.
    pub time: Option<ptr::Point1<T>>,
    /// Refers to delta time.
    /// This is also used to detect whether document is moving.
    pub dt: Option<ptr::Point1<T>>,
}

impl<T> Reactor<T> {
    /// Creates a new `Reactor`.
    pub fn new() -> Reactor<T> {
        Reactor {
            bools: vec![],
            points1: vec![],
            points2: vec![],
            points3: vec![],
            points4: vec![],
            splines1: vec![],
            splines2: vec![],
            splines3: vec![],
            splines4: vec![],
            surfaces1: vec![],
            surfaces2: vec![],
            surfaces3: vec![],
            surfaces4: vec![],
            colors: vec![],
            color_splines: vec![],
            bones1: vec![],
            bones2: vec![],
            bones3: vec![],
            bones4: vec![],
            time: None,
            dt: None,
        }
    }
}

/// Used to set runtime behavior of external evaluation.
pub struct Environment<T> {
    /// Total time.
    pub time: f64,
    /// Delta time.
    pub dt: f64,
    /// Cache.
    pub cache: Cache<T>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use vecmath::vec2_len as len2;
    use vecmath::vec2_sub as sub2;
    use vecmath::vec4_len as len4;
    use vecmath::vec4_sub as sub4;

    #[test]
    fn shapes() {
        let mut doc: Reactor<f32> = Reactor::new();
        let ref mut env = Environment {time: 0.0, dt: 0.0, cache: Cache::new(&doc)};
        let a = doc.add2([0.0, 0.0]);
        let b = doc.add2([100.0, 0.0]);
        let line = doc.add_line2(a, b);

        if let fns::Spline::Line(a2, b2) = doc[line] {
            assert_eq!(a, a2);
            assert_eq!(b, b2);
        }

        assert_eq!(doc.eval_spline2(line, 0.0, env), doc.eval2(a, env));
        assert_eq!(doc.eval_spline2(line, 1.0, env), doc.eval2(b, env));

        let c = doc.add2([100.0; 2]);
        let qbez = doc.add_qbez2(a, b, c);

        if let fns::Spline::QuadraticBezier(a2, b2, c2) = doc[qbez] {
            assert_eq!(a, a2);
            assert_eq!(b, b2);
            assert_eq!(c, c2);
        }

        assert_eq!(doc.eval_spline2(qbez, 0.0, env), doc.eval2(a, env));
        assert_eq!(doc.eval_spline2(qbez, 1.0, env), doc.eval2(c, env));

        let d = doc.add2([200.0, 0.0]);
        let cbez = doc.add_cbez2(a, b, c, d);

        if let fns::Spline::CubicBezier(a2, b2, c2, d2) = doc[cbez] {
            assert_eq!(a, a2);
            assert_eq!(b, b2);
            assert_eq!(c, c2);
            assert_eq!(d, d2);
        }

        assert_eq!(doc.eval_spline2(cbez, 0.0, env), doc.eval2(a, env));
        assert_eq!(doc.eval_spline2(cbez, 1.0, env), doc.eval2(d, env));

        let r = doc.add1(10.0);
        let circle = doc.add_circle2(a, r);
        assert_eq!(doc.eval_surface2(circle, [0.0, 0.0], env), doc.eval2(a, env));
        assert_eq!(doc.eval_surface2(circle, [0.0, 1.0], env), [10.0, 0.0]);
        assert!(len2(sub2(doc.eval_surface2(circle, [0.25, 1.0], env), [0.0, 10.0])) < 0.001);
        assert!(len2(sub2(doc.eval_surface2(circle, [0.5, 1.0], env), [-10.0, 0.0])) < 0.001);
        assert!(len2(sub2(doc.eval_surface2(circle, [0.75, 1.0], env), [0.0, -10.0])) < 0.001);
    }

    #[test]
    fn colors() {
        let mut doc: Reactor<f32> = Reactor::new();
        let ref mut env = Environment {time: 0.0, dt: 0.0, cache: Cache::new(&doc)};
        let black = doc.add_color([0.0, 0.0, 0.0, 1.0]);
        let white = doc.add_color([1.0; 4]);

        if let fns::Color::Data(data) = doc[black] {
            assert_eq!(data, [0.0, 0.0, 0.0, 1.0]);
        }
        if let fns::Color::Data(data) = doc[white] {
            assert_eq!(data, [1.0; 4]);
        }

        let spline = doc.add_lerp_color(white, black);
        assert!(len4(sub4(doc.eval_color_spline(spline, 0.0, color::ColorSpace::SRGB, env),
                doc.eval_color(white, color::ColorSpace::SRGB, env))) < 0.001);
        assert!(len4(sub4(doc.eval_color_spline(spline, 1.0, color::ColorSpace::SRGB, env),
                doc.eval_color(black, color::ColorSpace::SRGB, env))) < 0.001);
    }

    #[test]
    fn linalg() {
        let mut doc: Reactor<f32> = Reactor::new();
        let ref mut env = Environment {time: 0.0, dt: 0.0, cache: Cache::new(&doc)};
        let x = doc.add3([1.0, 0.0, 0.0]);
        let y = doc.add3([0.0, 1.0, 0.0]);
        let z = doc.add3([0.0, 0.0, 1.0]);
        let cross = doc.add_cross3(x, y);

        assert_eq!(doc.eval3(cross, env), doc.eval3(z, env));

        let a = doc.add2([1.0, 0.0]);
        let dot = doc.add_dot2(a, a);
        assert_eq!(doc.eval1(dot, env), 1.0);
    }

    #[test]
    fn environment() {
        let mut doc: Reactor<f32> = Reactor::new();
        let ref mut env = Environment {time: 10.0, dt: 1.0, cache: Cache::new(&doc)};
        let time = doc.add_time();
        let dt = doc.add_dt();
        assert_eq!(doc.eval1(time, env), 10.0);
        assert_eq!(doc.eval1(dt, env), 1.0);
    }

    #[test]
    fn ref_count() {
        let mut doc: Reactor<f32> = Reactor::new();
        let a = doc.add1(1.0);
        let b = doc.add1(2.0);
        let c = doc.add_sum1(a, b);
        let d = doc.add_sum1(a, c);
        let e = doc.add_prod1(a, c);

        let ref mut env = Environment {time: 10.0, dt: 1.0, cache: Cache::new(&doc)};
        // `a` is a value, so it should not be stored in cache.
        assert!(!env.cache.points1.contains_key(&usize::from(a)));
        // `b` is only referenced once, so it should not be stored in cache.
        assert!(!env.cache.points1.contains_key(&usize::from(b)));
        // `c` is references from two places, so it should be stored in cache.
        assert!(env.cache.points1.contains_key(&usize::from(c)));
        // `d` is only referenced once, so it should not be stored in cache.
        assert!(!env.cache.points1.contains_key(&usize::from(d)));
        // `e` is only referenced once, so it should not be stored in cache.
        assert!(!env.cache.points1.contains_key(&usize::from(e)));
    }

    #[test]
    fn physics_1() {
        let mut doc: Reactor<f32> = Reactor::new();
        let a = doc.add1(0.0);
        let b = doc.add1(100.0);
        let c = doc.add1(50.0);
        let _bone = doc.add_eq_bone1(a, b, c);

        let selection = Selection::new();
        let ref mut env = Environment {time: 0.0, dt: 0.0, cache: Cache::new(&doc)};
        let mut physics = Physics::new(&doc, &selection, env);
        physics.simulate();

        assert_eq!(physics.pos1[0], (0.0, 25.0));
        assert_eq!(physics.pos1[1], (100.0, 75.0));

        physics.commit(&mut doc);
        assert_eq!(doc.eval1(a, env), 25.0);
        assert_eq!(doc.eval1(b, env), 75.0);
    }

    #[test]
    fn physics_2() {
        let mut doc: Reactor<f32> = Reactor::new();
        let a = doc.add2([0.0; 2]);
        let b = doc.add2([100.0, 0.0]);
        let c = doc.add1(50.0);
        let _bone = doc.add_eq_bone2(a, b, c);

        let selection = Selection::new();
        let ref mut env = Environment {time: 0.0, dt: 0.0, cache: Cache::new(&doc)};
        let mut physics = Physics::new(&doc, &selection, env);
        physics.simulate();

        assert_eq!(physics.pos2[0], ([0.0, 0.0], [25.0, 0.0]));
        assert_eq!(physics.pos2[1], ([100.0, 0.0], [75.0, 0.0]));

        physics.commit(&mut doc);
        assert_eq!(doc.eval2(a, env), [25.0, 0.0]);
        assert_eq!(doc.eval2(b, env), [75.0, 0.0]);

        assert_eq!(physics.pos2[0], ([25.0, 0.0], [25.0, 0.0]));
        assert_eq!(physics.pos2[1], ([75.0, 0.0], [75.0, 0.0]));
    }
}
