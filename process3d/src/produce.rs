//! Produce Pattern.

use crate::{Aabb, Chunk, Cube, Matrix4, Point, Quad, Triangle};

/// Implemented by virtual lists that produces chunks of data.
pub trait Produce<T> {
    /// The length of the virtual list.
    fn virtual_length(&self) -> usize;
    /// Produce a chunk at some offset.
    fn produce(&self, offset: usize) -> Chunk<T>;
}

/// Gets the initial chunk mask.
pub fn init_chunk_mask(len: usize, offset: usize) -> u64 {
    let n = (len - offset).min(64);
    1_u64.checked_shl(n as u32).unwrap_or(0).wrapping_sub(1)
}

/// Transforms chunk of triangles using a matrix.
pub struct TransformProducer<'a, T: ?Sized> {
    /// The matrix transform.
    pub matrix: Matrix4,
    /// The inner producer.
    pub inner: &'a T,
}

impl<'a, T> Produce<Triangle> for TransformProducer<'a, T>
    where T: Produce<Triangle> + ?Sized
{
    #[inline(always)]
    fn virtual_length(&self) -> usize {self.inner.virtual_length()}
    #[inline(always)]
    fn produce(&self, offset: usize) -> Chunk<Triangle> {
        let mut chunk = self.inner.produce(offset);
        crate::math::transform_chunk(&self.matrix, &mut chunk);
        chunk
    }
}

impl<T: Default + Copy> Produce<T> for [T] {
    #[inline(always)]
    fn virtual_length(&self) -> usize {self.len()}
    fn produce(&self, offset: usize) -> Chunk<T> {
        let list = &self[offset..];
        let mut chunk = [Default::default(); 64];
        let n = list.len().min(64);
        for i in 0..n {
            chunk[i] = list[i];
        }
        chunk
    }
}

impl<T> Produce<T> for Vec<T>
    where [T]: Produce<T>
{
    #[inline(always)]
    fn virtual_length(&self) -> usize {
        <[T] as Produce::<T>>::virtual_length(self)
    }
    #[inline(always)]
    fn produce(&self, offset: usize) -> Chunk<T> {
        <[T] as Produce::<T>>::produce(self, offset)
    }
}

impl Produce<Triangle> for [Quad] {
    #[inline(always)]
    fn virtual_length(&self) -> usize {self.len() * 2}
    fn produce(&self, offset: usize) -> Chunk<Triangle> {
        use crate::quad::quad_to_triangles;

        let j = offset / 2;
        let mut chunk = [Default::default(); 64];
        if offset % 2 == 0 {
            // Even case.
            let list = &self[j..];
            let n = list.len().min(32);
            for i in 0..n {
                let (a, b) = quad_to_triangles(list[i]);
                chunk[i * 2] = a;
                chunk[i * 2 + 1] = b;
            }
        } else {
            // Odd case.
            let list = &self[j..];
            if list.len() == 0 {return chunk};

            let (_, b) = quad_to_triangles(list[0]);
            chunk[0] = b;
            let n = (list.len() - 1).min(30);
            for i in 0..n {
                let (a, b) = quad_to_triangles(list[i]);
                chunk[i * 2 + 1] = a;
                chunk[i * 2 + 2] = b;
            }

            if list.len() >= 32 {
                let (a, _) = quad_to_triangles(list[31]);
                chunk[63] = a;
            }
        }
        chunk
    }
}

/// Converts into cubes.
pub trait IntoCube: Copy {
    /// Convert into cube.
    fn into_cube(self) -> Cube;
}

impl IntoCube for Cube {
    #[inline(always)]
    fn into_cube(self) -> Cube {self}
}

impl IntoCube for Aabb {
    #[inline(always)]
    fn into_cube(self) -> Cube {crate::cube::aabb_to_cube(self)}
}

impl IntoCube for Point {
    #[inline(always)]
    fn into_cube(self) -> Cube {crate::cube::aabb_to_cube((
        self,
        vecmath::vec3_add(self, [1.0; 3])
    ))}
}

macro_rules! into_cube_impl {
    ($($t:ty),*) => {
        $(
            impl IntoCube for Point<$t> {
                #[inline(always)]
                fn into_cube(self) -> Cube {
                    let pt = [self[0] as f32, self[1] as f32, self[2] as f32];
                    crate::cube::aabb_to_cube((
                        pt,
                        vecmath::vec3_add(pt, [1.0; 3])
                    ))
                }
            }
        )*
    };
}

into_cube_impl!(u8, u16, u32, u64, i8, i16, i32, i64);

impl<T: IntoCube> Produce<Triangle> for [T] {
    #[inline(always)]
    fn virtual_length(&self) -> usize {self.len() * 12}
    fn produce(&self, offset: usize) -> Chunk<Triangle> {
        use crate::cube::*;
        use crate::quad::quad_to_triangles;

        let j = offset / 12;
        let k = offset % 12;
        let mut chunk = [Default::default(); 64];

        let list = &self[j..];

        match k {
            0 => {
                // There are 5 whole cubes, taking up 60 triangles.
                let n = list.len().min(5);
                for i in 0..n {
                    let cube = &list[i].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                        cube_top(cube),
                        cube_bottom(cube),
                        cube_left(cube),
                        cube_right(cube),
                    ];
                    for (j, quad) in quads.iter().enumerate() {
                        let (a, b) = quad_to_triangles(*quad);
                        chunk[i * 12 + j * 2] = a;
                        chunk[i * 12 + j * 2 + 1] = b;
                    }
                }

                if list.len() > n {
                    // There is room for 4 more triangles.
                    let cube = &list[n].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                    ];

                    // Far.
                    let (a, b) = quad_to_triangles(quads[0]);
                    chunk[n * 12] = a;
                    chunk[n * 12 + 1] = b;
                    // Near.
                    let (a, b) = quad_to_triangles(quads[1]);
                    chunk[n * 12 + 2] = a;
                    chunk[n * 12 + 3] = b;
                }
            }
            1 => {
                if list.len() == 0 {return chunk};

                let cube = &list[0].into_cube();
                let quads = [
                    cube_far(cube),
                    cube_near(cube),
                    cube_top(cube),
                    cube_bottom(cube),
                    cube_left(cube),
                    cube_right(cube),
                ];
                // Far.
                let (_, b) = quad_to_triangles(quads[0]);
                chunk[0] = b;
                // Near.
                let (a, b) = quad_to_triangles(quads[1]);
                chunk[1] = a;
                chunk[2] = b;
                // Top.
                let (a, b) = quad_to_triangles(quads[2]);
                chunk[3] = a;
                chunk[4] = b;
                // Bottom.
                let (a, b) = quad_to_triangles(quads[3]);
                chunk[5] = a;
                chunk[6] = b;
                // Left.
                let (a, b) = quad_to_triangles(quads[4]);
                chunk[7] = a;
                chunk[8] = b;
                // Right.
                let (a, b) = quad_to_triangles(quads[5]);
                chunk[9] = a;
                chunk[10] = b;

                // There are 4 whole cubes, taking up 48 triangles.
                let n = (list.len() - 1).min(4);
                for i in 0..n {
                    let cube = &list[i + 1].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                        cube_top(cube),
                        cube_bottom(cube),
                        cube_left(cube),
                        cube_right(cube),
                    ];
                    for (j, quad) in quads.iter().enumerate() {
                        let (a, b) = quad_to_triangles(*quad);
                        chunk[i * 12 + 11 + j * 2] = a;
                        chunk[i * 12 + 11 + j * 2 + 1] = b;
                    }
                }

                if list.len() > n + 1 {
                    let cube = &list[n + 1].into_cube();
                    let quad = cube_far(cube);
                    // Far.
                    let (a, _) = quad_to_triangles(quad);
                    chunk[n * 12 + 11] = a;
                }
            }
            2 => {
                if list.len() == 0 {return chunk};

                let cube = &list[0].into_cube();
                let quads = [
                    cube_near(cube),
                    cube_top(cube),
                    cube_bottom(cube),
                    cube_left(cube),
                    cube_right(cube),
                ];
                // Near.
                let (a, b) = quad_to_triangles(quads[0]);
                chunk[0] = a;
                chunk[1] = b;
                // Top.
                let (a, b) = quad_to_triangles(quads[1]);
                chunk[2] = a;
                chunk[3] = b;
                // Bottom.
                let (a, b) = quad_to_triangles(quads[2]);
                chunk[4] = a;
                chunk[5] = b;
                // Left.
                let (a, b) = quad_to_triangles(quads[3]);
                chunk[6] = a;
                chunk[7] = b;
                // Right.
                let (a, b) = quad_to_triangles(quads[4]);
                chunk[8] = a;
                chunk[9] = b;

                // There are 4 whole cubes, taking up 48 triangles.
                let n = (list.len() - 1).min(4);
                for i in 0..n {
                    let cube = &list[i + 1].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                        cube_top(cube),
                        cube_bottom(cube),
                        cube_left(cube),
                        cube_right(cube),
                    ];
                    for (j, quad) in quads.iter().enumerate() {
                        let (a, b) = quad_to_triangles(*quad);
                        chunk[i * 12 + 10 + j * 2] = a;
                        chunk[i * 12 + 10 + j * 2 + 1] = b;
                    }
                }

                if list.len() > n + 1 {
                    let cube = &list[n + 1].into_cube();
                    let quad = cube_far(cube);
                    // Far.
                    let (a, b) = quad_to_triangles(quad);
                    chunk[n * 12 + 10] = a;
                    chunk[n * 12 + 10 + 1] = b;
                }
            }
            3 => {
                if list.len() == 0 {return chunk};

                let cube = &list[0].into_cube();
                let quads = [
                    cube_near(cube),
                    cube_top(cube),
                    cube_bottom(cube),
                    cube_left(cube),
                    cube_right(cube),
                ];
                // Near.
                let (_, b) = quad_to_triangles(quads[0]);
                chunk[0] = b;
                // Top.
                let (a, b) = quad_to_triangles(quads[1]);
                chunk[1] = a;
                chunk[2] = b;
                // Bottom.
                let (a, b) = quad_to_triangles(quads[2]);
                chunk[3] = a;
                chunk[4] = b;
                // Left.
                let (a, b) = quad_to_triangles(quads[3]);
                chunk[5] = a;
                chunk[6] = b;
                // Right.
                let (a, b) = quad_to_triangles(quads[4]);
                chunk[7] = a;
                chunk[8] = b;

                // There are 4 whole cubes, taking up 48 triangles.
                let n = (list.len() - 1).min(4);
                for i in 0..n {
                    let cube = &list[i + 1].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                        cube_top(cube),
                        cube_bottom(cube),
                        cube_left(cube),
                        cube_right(cube),
                    ];
                    for (j, quad) in quads.iter().enumerate() {
                        let (a, b) = quad_to_triangles(*quad);
                        chunk[i * 12 + 9 + j * 2] = a;
                        chunk[i * 12 + 9 + j * 2 + 1] = b;
                    }
                }

                if list.len() > n + 1 {
                    let cube = &list[n + 1].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                    ];
                    // Far.
                    let (a, b) = quad_to_triangles(quads[0]);
                    chunk[n * 12 + 9] = a;
                    chunk[n * 12 + 9 + 1] = b;
                    // Near.
                    let (a, _) = quad_to_triangles(quads[1]);
                    chunk[n * 12 + 9 + 2] = a;
                }
            }
            4 => {
                if list.len() == 0 {return chunk};

                let cube = &list[0].into_cube();
                let quads = [
                    cube_top(cube),
                    cube_bottom(cube),
                    cube_left(cube),
                    cube_right(cube),
                ];
                // Top.
                let (a, b) = quad_to_triangles(quads[0]);
                chunk[0] = a;
                chunk[1] = b;
                // Bottom.
                let (a, b) = quad_to_triangles(quads[1]);
                chunk[2] = a;
                chunk[3] = b;
                // Left.
                let (a, b) = quad_to_triangles(quads[2]);
                chunk[4] = a;
                chunk[5] = b;
                // Right.
                let (a, b) = quad_to_triangles(quads[3]);
                chunk[6] = a;
                chunk[7] = b;

                // There are 4 whole cubes, taking up 48 triangles.
                let n = (list.len() - 1).min(4);
                for i in 0..n {
                    let cube = &list[i + 1].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                        cube_top(cube),
                        cube_bottom(cube),
                        cube_left(cube),
                        cube_right(cube),
                    ];
                    for (j, quad) in quads.iter().enumerate() {
                        let (a, b) = quad_to_triangles(*quad);
                        chunk[i * 12 + 8 + j * 2] = a;
                        chunk[i * 12 + 8 + j * 2 + 1] = b;
                    }
                }

                if list.len() > n + 1 {
                    let cube = &list[n + 1].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                    ];
                    // Far.
                    let (a, b) = quad_to_triangles(quads[0]);
                    chunk[n * 12 + 8] = a;
                    chunk[n * 12 + 8 + 1] = b;
                    // Near.
                    let (a, b) = quad_to_triangles(quads[1]);
                    chunk[n * 12 + 8 + 2] = a;
                    chunk[n * 12 + 8 + 3] = b;
                }
            }
            5 => {
                if list.len() == 0 {return chunk};

                let cube = &list[0].into_cube();
                let quads = [
                    cube_top(cube),
                    cube_bottom(cube),
                    cube_left(cube),
                    cube_right(cube),
                ];
                // Top.
                let (_, b) = quad_to_triangles(quads[0]);
                chunk[0] = b;
                // Bottom.
                let (a, b) = quad_to_triangles(quads[1]);
                chunk[1] = a;
                chunk[2] = b;
                // Left.
                let (a, b) = quad_to_triangles(quads[2]);
                chunk[3] = a;
                chunk[4] = b;
                // Right.
                let (a, b) = quad_to_triangles(quads[3]);
                chunk[5] = a;
                chunk[6] = b;

                // There are 4 whole cubes, taking up 48 triangles.
                let n = (list.len() - 1).min(4);
                for i in 0..n {
                    let cube = &list[i + 1].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                        cube_top(cube),
                        cube_bottom(cube),
                        cube_left(cube),
                        cube_right(cube),
                    ];
                    for (j, quad) in quads.iter().enumerate() {
                        let (a, b) = quad_to_triangles(*quad);
                        chunk[i * 12 + 7 + j * 2] = a;
                        chunk[i * 12 + 7 + j * 2 + 1] = b;
                    }
                }

                if list.len() > n + 1 {
                    let cube = &list[n + 1].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                        cube_top(cube),
                    ];
                    // Far.
                    let (a, b) = quad_to_triangles(quads[0]);
                    chunk[n * 12 + 7] = a;
                    chunk[n * 12 + 7 + 1] = b;
                    // Near.
                    let (a, b) = quad_to_triangles(quads[1]);
                    chunk[n * 12 + 7 + 2] = a;
                    chunk[n * 12 + 7 + 3] = b;
                    // Top.
                    let (a, _) = quad_to_triangles(quads[2]);
                    chunk[n * 12 + 7 + 4] = a;
                }
            }
            6 => {
                if list.len() == 0 {return chunk};

                let cube = &list[0].into_cube();
                let quads = [
                    cube_bottom(cube),
                    cube_left(cube),
                    cube_right(cube),
                ];
                // Bottom.
                let (a, b) = quad_to_triangles(quads[0]);
                chunk[0] = a;
                chunk[1] = b;
                // Left.
                let (a, b) = quad_to_triangles(quads[1]);
                chunk[2] = a;
                chunk[3] = b;
                // Right.
                let (a, b) = quad_to_triangles(quads[2]);
                chunk[4] = a;
                chunk[5] = b;

                // There are 4 whole cubes, taking up 48 triangles.
                let n = (list.len() - 1).min(4);
                for i in 0..n {
                    let cube = &list[i + 1].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                        cube_top(cube),
                        cube_bottom(cube),
                        cube_left(cube),
                        cube_right(cube),
                    ];
                    for (j, quad) in quads.iter().enumerate() {
                        let (a, b) = quad_to_triangles(*quad);
                        chunk[i * 12 + 6 + j * 2] = a;
                        chunk[i * 12 + 6 + j * 2 + 1] = b;
                    }
                }

                if list.len() > n + 1 {
                    let cube = &list[n + 1].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                        cube_top(cube),
                    ];
                    // Far.
                    let (a, b) = quad_to_triangles(quads[0]);
                    chunk[n * 12 + 6] = a;
                    chunk[n * 12 + 6 + 1] = b;
                    // Near.
                    let (a, b) = quad_to_triangles(quads[1]);
                    chunk[n * 12 + 6 + 2] = a;
                    chunk[n * 12 + 6 + 3] = b;
                    // Top.
                    let (a, b) = quad_to_triangles(quads[2]);
                    chunk[n * 12 + 6 + 4] = a;
                    chunk[n * 12 + 6 + 5] = b;
                }
            }
            7 => {
                if list.len() == 0 {return chunk};

                let cube = &list[0].into_cube();
                let quads = [
                    cube_bottom(cube),
                    cube_left(cube),
                    cube_right(cube),
                ];
                // Bottom.
                let (_, b) = quad_to_triangles(quads[0]);
                chunk[0] = b;
                // Left.
                let (a, b) = quad_to_triangles(quads[1]);
                chunk[1] = a;
                chunk[2] = b;
                // Right.
                let (a, b) = quad_to_triangles(quads[2]);
                chunk[3] = a;
                chunk[4] = b;

                // There are 4 whole cubes, taking up 48 triangles.
                let n = (list.len() - 1).min(4);
                for i in 0..n {
                    let cube = &list[i + 1].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                        cube_top(cube),
                        cube_bottom(cube),
                        cube_left(cube),
                        cube_right(cube),
                    ];
                    for (j, quad) in quads.iter().enumerate() {
                        let (a, b) = quad_to_triangles(*quad);
                        chunk[i * 12 + 5 + j * 2] = a;
                        chunk[i * 12 + 5 + j * 2 + 1] = b;
                    }
                }

                if list.len() > n + 1 {
                    let cube = &list[n + 1].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                        cube_top(cube),
                        cube_bottom(cube),
                    ];
                    // Far.
                    let (a, b) = quad_to_triangles(quads[0]);
                    chunk[n * 12 + 5] = a;
                    chunk[n * 12 + 5 + 1] = b;
                    // Near.
                    let (a, b) = quad_to_triangles(quads[1]);
                    chunk[n * 12 + 5 + 2] = a;
                    chunk[n * 12 + 5 + 3] = b;
                    // Top.
                    let (a, b) = quad_to_triangles(quads[2]);
                    chunk[n * 12 + 5 + 4] = a;
                    chunk[n * 12 + 5 + 5] = b;
                    // Bottom.
                    let (a, _) = quad_to_triangles(quads[3]);
                    chunk[n * 12 + 5 + 6] = a;
                }
            }
            8 => {
                if list.len() == 0 {return chunk};

                let cube = &list[0].into_cube();
                let quads = [
                    cube_left(cube),
                    cube_right(cube),
                ];
                // Left.
                let (a, b) = quad_to_triangles(quads[0]);
                chunk[0] = a;
                chunk[1] = b;
                // Right.
                let (a, b) = quad_to_triangles(quads[1]);
                chunk[2] = a;
                chunk[3] = b;

                // There are 4 whole cubes, taking up 48 triangles.
                let n = (list.len() - 1).min(4);
                for i in 0..n {
                    let cube = &list[i + 1].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                        cube_top(cube),
                        cube_bottom(cube),
                        cube_left(cube),
                        cube_right(cube),
                    ];
                    for (j, quad) in quads.iter().enumerate() {
                        let (a, b) = quad_to_triangles(*quad);
                        chunk[i * 12 + 4 + j * 2] = a;
                        chunk[i * 12 + 4 + j * 2 + 1] = b;
                    }
                }

                if list.len() > n + 1 {
                    let cube = &list[n + 1].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                        cube_top(cube),
                        cube_bottom(cube),
                    ];
                    // Far.
                    let (a, b) = quad_to_triangles(quads[0]);
                    chunk[n * 12 + 4] = a;
                    chunk[n * 12 + 4 + 1] = b;
                    // Near.
                    let (a, b) = quad_to_triangles(quads[1]);
                    chunk[n * 12 + 4 + 2] = a;
                    chunk[n * 12 + 4 + 3] = b;
                    // Top.
                    let (a, b) = quad_to_triangles(quads[2]);
                    chunk[n * 12 + 4 + 4] = a;
                    chunk[n * 12 + 4 + 5] = b;
                    // Bottom.
                    let (a, b) = quad_to_triangles(quads[3]);
                    chunk[n * 12 + 4 + 6] = a;
                    chunk[n * 12 + 4 + 7] = b;
                }
            }
            9 => {
                if list.len() == 0 {return chunk};

                let cube = &list[0].into_cube();
                let quads = [
                    cube_left(cube),
                    cube_right(cube),
                ];
                // Left.
                let (_, b) = quad_to_triangles(quads[0]);
                chunk[0] = b;
                // Right.
                let (a, b) = quad_to_triangles(quads[1]);
                chunk[1] = a;
                chunk[2] = b;

                // There are 4 whole cubes, taking up 48 triangles.
                let n = (list.len() - 1).min(4);
                for i in 0..n {
                    let cube = &list[i + 1].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                        cube_top(cube),
                        cube_bottom(cube),
                        cube_left(cube),
                        cube_right(cube),
                    ];
                    for (j, quad) in quads.iter().enumerate() {
                        let (a, b) = quad_to_triangles(*quad);
                        chunk[i * 12 + 3 + j * 2] = a;
                        chunk[i * 12 + 3 + j * 2 + 1] = b;
                    }
                }

                if list.len() > n + 1 {
                    let cube = &list[n + 1].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                        cube_top(cube),
                        cube_bottom(cube),
                        cube_left(cube),
                    ];
                    // Far.
                    let (a, b) = quad_to_triangles(quads[0]);
                    chunk[n * 12 + 3] = a;
                    chunk[n * 12 + 3 + 1] = b;
                    // Near.
                    let (a, b) = quad_to_triangles(quads[1]);
                    chunk[n * 12 + 3 + 2] = a;
                    chunk[n * 12 + 3 + 3] = b;
                    // Top.
                    let (a, b) = quad_to_triangles(quads[2]);
                    chunk[n * 12 + 3 + 4] = a;
                    chunk[n * 12 + 3 + 5] = b;
                    // Bottom.
                    let (a, b) = quad_to_triangles(quads[3]);
                    chunk[n * 12 + 3 + 6] = a;
                    chunk[n * 12 + 3 + 7] = b;
                    // Left.
                    let (a, _) = quad_to_triangles(quads[4]);
                    chunk[n * 12 + 3 + 8] = a;
                }
            }
            10 => {
                if list.len() == 0 {return chunk};

                let cube = &list[0].into_cube();
                let quad = cube_right(cube);
                // Right.
                let (a, b) = quad_to_triangles(quad);
                chunk[0] = a;
                chunk[1] = b;

                // There are 4 whole cubes, taking up 48 triangles.
                let n = (list.len() - 1).min(4);
                for i in 0..n {
                    let cube = &list[i + 1].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                        cube_top(cube),
                        cube_bottom(cube),
                        cube_left(cube),
                        cube_right(cube),
                    ];
                    for (j, quad) in quads.iter().enumerate() {
                        let (a, b) = quad_to_triangles(*quad);
                        chunk[i * 12 + 2 + j * 2] = a;
                        chunk[i * 12 + 2 + j * 2 + 1] = b;
                    }
                }

                if list.len() > n + 1 {
                    let cube = &list[n + 1].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                        cube_top(cube),
                        cube_bottom(cube),
                        cube_left(cube),
                    ];
                    // Far.
                    let (a, b) = quad_to_triangles(quads[0]);
                    chunk[n * 12 + 2] = a;
                    chunk[n * 12 + 2 + 1] = b;
                    // Near.
                    let (a, b) = quad_to_triangles(quads[1]);
                    chunk[n * 12 + 2 + 2] = a;
                    chunk[n * 12 + 2 + 3] = b;
                    // Top.
                    let (a, b) = quad_to_triangles(quads[2]);
                    chunk[n * 12 + 2 + 4] = a;
                    chunk[n * 12 + 2 + 5] = b;
                    // Bottom.
                    let (a, b) = quad_to_triangles(quads[3]);
                    chunk[n * 12 + 2 + 6] = a;
                    chunk[n * 12 + 2 + 7] = b;
                    // Left.
                    let (a, b) = quad_to_triangles(quads[4]);
                    chunk[n * 12 + 2 + 8] = a;
                    chunk[n * 12 + 2 + 9] = b;
                }
            }
            _ => {
                if list.len() == 0 {return chunk};

                let cube = &list[0].into_cube();
                let quad = cube_right(cube);
                // Right.
                let (_, b) = quad_to_triangles(quad);
                chunk[0] = b;

                // There are 4 whole cubes, taking up 48 triangles.
                let n = (list.len() - 1).min(4);
                for i in 0..n {
                    let cube = &list[i + 1].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                        cube_top(cube),
                        cube_bottom(cube),
                        cube_left(cube),
                        cube_right(cube),
                    ];
                    for (j, quad) in quads.iter().enumerate() {
                        let (a, b) = quad_to_triangles(*quad);
                        chunk[i * 12 + 1 + j * 2] = a;
                        chunk[i * 12 + 1 + j * 2 + 1] = b;
                    }
                }

                if list.len() > n + 1 {
                    let cube = &list[n + 1].into_cube();
                    let quads = [
                        cube_far(cube),
                        cube_near(cube),
                        cube_top(cube),
                        cube_bottom(cube),
                        cube_left(cube),
                        cube_right(cube),
                    ];
                    // Far.
                    let (a, b) = quad_to_triangles(quads[0]);
                    chunk[n * 12 + 1] = a;
                    chunk[n * 12 + 1 + 1] = b;
                    // Near.
                    let (a, b) = quad_to_triangles(quads[1]);
                    chunk[n * 12 + 1 + 2] = a;
                    chunk[n * 12 + 1 + 3] = b;
                    // Top.
                    let (a, b) = quad_to_triangles(quads[2]);
                    chunk[n * 12 + 1 + 4] = a;
                    chunk[n * 12 + 1 + 5] = b;
                    // Bottom.
                    let (a, b) = quad_to_triangles(quads[3]);
                    chunk[n * 12 + 1 + 6] = a;
                    chunk[n * 12 + 1 + 7] = b;
                    // Left.
                    let (a, b) = quad_to_triangles(quads[4]);
                    chunk[n * 12 + 1 + 8] = a;
                    chunk[n * 12 + 1 + 9] = b;
                    // Right.
                    let (a, _) = quad_to_triangles(quads[5]);
                    chunk[n * 12 + 1 + 10] = a;
                }
            }
        }
        chunk
    }
}
