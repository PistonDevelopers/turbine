//! # Quad algorithms

use crate::{Quad, Triangle};

/// Convert from quad to two triangles.
pub fn quad_to_triangles(quad: Quad) -> (Triangle, Triangle) {
    ((quad[0], quad[1], quad[2]),
     (quad[2], quad[1], quad[3]))
}
