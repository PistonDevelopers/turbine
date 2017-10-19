//! Pointers to higher order structures.

use std::marker::PhantomData;

/// Points to bool.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Bool(usize);
/// Points to a scalar.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Point1<T>(pub(crate) usize, PhantomData<T>);
/// Points to a 2D point.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Point2<T>(pub(crate) usize, PhantomData<T>);
/// Points to a 3D point.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Point3<T>(pub(crate) usize, PhantomData<T>);
/// Points to a 4D point.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Point4<T>(pub(crate) usize, PhantomData<T>);
/// Points to a spline.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Spline<T>(usize, PhantomData<T>);
/// Points to a surface.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Surface<T>(usize, PhantomData<T>);
/// Points to a color.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Color(usize);
/// Points to a color spline.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ColorSpline(usize);
/// Points to a bone.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Bone<T>(usize, PhantomData<T>);

macro_rules! from_impl {
    ($point:ident) => {
        impl<T> From<usize> for $point<T> {
            fn from(val: usize) -> $point<T> {$point(val, PhantomData)}
        }
        impl<T> From<$point<T>> for usize {
            fn from(val: $point<T>) -> usize {val.0}
        }
    }
}

from_impl!{Point1}
from_impl!{Point2}
from_impl!{Point3}
from_impl!{Point4}
from_impl!{Spline}
from_impl!{Surface}
from_impl!{Bone}

impl From<usize> for Bool {
    fn from(val: usize) -> Bool {Bool(val)}
}
impl From<Bool> for usize {
    fn from(val: Bool) -> usize {val.0}
}

impl From<usize> for Color {
    fn from(val: usize) -> Color {Color(val)}
}
impl From<Color> for usize {
    fn from(val: Color) -> usize {val.0}
}

impl From<usize> for ColorSpline {
    fn from(val: usize) -> ColorSpline {ColorSpline(val)}
}
impl From<ColorSpline> for usize {
    fn from(val: ColorSpline) -> usize {val.0}
}
