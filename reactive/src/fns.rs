//! Higher order structures for functions (including data).
//!
//! ### Design
//!
//! `Data` variants stores the actual data.
//!
//! `Time` variants evaluates a higher homotopy function.
//! For example, a line in 3D has homotopy type `t -> point3`.
//! This is a function that can be evaluated using a scalar.
//! Since the function returns a 3D point, this particular `Time` variant
//! needs to be stored in the enum for `Point3`.

use crate::*;
use ptr;

/// Boolean values.
#[derive(Copy, Clone, Debug)]
pub enum Bool<T> {
    /// Data bool.
    Data(bool),
    /// Logical NOT.
    Not(ptr::Bool),
    /// Logical AND.
    And(ptr::Bool, ptr::Bool),
    /// Logical OR.
    Or(ptr::Bool, ptr::Bool),
    /// Logical XOR.
    Xor(ptr::Bool, ptr::Bool),
    /// Logical EQ.
    EqBool(ptr::Bool, ptr::Bool),
    /// Two numbers are equal.
    Eq1(ptr::Point1<T>, ptr::Point1<T>),
    /// Two 2D vectors are equal.
    Eq2(ptr::Point2<T>, ptr::Point2<T>),
    /// Two 3D vectors are equal.
    Eq3(ptr::Point3<T>, ptr::Point3<T>),
    /// Two 4D vectors are equal.
    Eq4(ptr::Point4<T>, ptr::Point4<T>),
    /// Less.
    Less(ptr::Point1<T>, ptr::Point1<T>),
    /// Greater or equal.
    GreaterOrEqual(ptr::Point1<T>, ptr::Point1<T>),
}

/// Scalar.
#[derive(Copy, Clone, Debug)]
pub enum Point1<T> {
    /// Data scalar.
    Data(T),
    /// `f(t)` stored as `f, t`.
    Time1(SplineRef1<T>, ptr::Point1<T>),
    /// `f([t; 2])` stored as `f, [t; 2]`.
    Time2(SurfaceRef1<T>, ptr::Point2<T>),
    /// Sums two numbers.
    Sum(ptr::Point1<T>, ptr::Point1<T>),
    /// Difference between two numbers.
    Diff(ptr::Point1<T>, ptr::Point1<T>),
    /// Product of two numbers.
    Prod(ptr::Point1<T>, ptr::Point1<T>),
    /// Divides one number with another.
    Div(ptr::Point1<T>, ptr::Point1<T>),
    /// Dot product of two 2D vectors.
    Dot2(ptr::Point2<T>, ptr::Point2<T>),
    /// Dot product of two 3D vectors.
    Dot3(ptr::Point3<T>, ptr::Point3<T>),
    /// Dot product of two 4D vectors.
    Dot4(ptr::Point4<T>, ptr::Point4<T>),
    /// Cross product between two 2D vectors.
    Cross(ptr::Point2<T>, ptr::Point2<T>),
    /// Absolute value of scalar.
    Abs(ptr::Point1<T>),
    /// Length of 2D vector.
    Len2(ptr::Point2<T>),
    /// Length of 3D vector.
    Len3(ptr::Point3<T>),
    /// Length of 4D vector.
    Len4(ptr::Point4<T>),
    /// Negative value of number.
    Neg(ptr::Point1<T>),
    /// Sign value of number.
    Sign(ptr::Point1<T>),
    /// Sine of angle.
    Sin(ptr::Point1<T>),
    /// Cosine of angle.
    Cos(ptr::Point1<T>),
    /// Tangent of angle.
    Tan(ptr::Point1<T>),
    /// Inverse sine of angle.
    Asin(ptr::Point1<T>),
    /// Inverse cosine of angle.
    Acos(ptr::Point1<T>),
    /// Inverse tangent of angle.
    Atan(ptr::Point1<T>),
    /// Inverse tangent of `y, x`.
    Atan2(ptr::Point1<T>, ptr::Point1<T>),
    /// Hyperbolic sine of angle.
    Sinh(ptr::Point1<T>),
    /// Hyperbolic cosine of angle.
    Cosh(ptr::Point1<T>),
    /// Hyperbolic tangent of angle.
    Tanh(ptr::Point1<T>),
    /// Inverse hyperbolic sine of angle.
    Asinh(ptr::Point1<T>),
    /// Inverse hyperbolic cosine of angle.
    Acosh(ptr::Point1<T>),
    /// Inverse hyperbolic tangent of angle.
    Atanh(ptr::Point1<T>),
    /// Square root of number.
    Sqrt(ptr::Point1<T>),
    /// Maximum of two numbers.
    Max(ptr::Point1<T>, ptr::Point1<T>),
    /// Minimum of two numbers.
    Min(ptr::Point1<T>, ptr::Point1<T>),
    /// Convert degrees to radians.
    DegToRad(ptr::Point1<T>),
    /// Convert radians to degrees.
    RadToDeg(ptr::Point1<T>),
    /// Returns the time from environment.
    Time,
    /// Returns delta time from environment.
    DeltaTime,
}

/// 2D point.
#[derive(Copy, Clone, Debug)]
pub enum Point2<T> {
    /// Data point.
    Data([T; 2]),
    /// `f(t)` stored as `f, t`.
    Time1(SplineRef2<T>, ptr::Point1<T>),
    /// `f([t; 2])` stored as `f, [t; 2]`.
    Time2(SurfaceRef2<T>, ptr::Point2<T>),
    /// Sums two 2D vectors.
    Sum(ptr::Point2<T>, ptr::Point2<T>),
    /// Product component wise of two 2D vectors.
    Prod(ptr::Point2<T>, ptr::Point2<T>),
    /// Difference between two 2D vectors.
    Diff(ptr::Point2<T>, ptr::Point2<T>),
    /// Maximum component wise of two 2D vectors.
    Max(ptr::Point2<T>, ptr::Point2<T>),
    /// Minimum component wise of two 2D vectors.
    Min(ptr::Point2<T>, ptr::Point2<T>),
}

/// 3D point.
#[derive(Copy, Clone, Debug)]
pub enum Point3<T> {
    /// Data point.
    Data([T; 3]),
    /// `f(t)` stored as `f, t`.
    Time1(SplineRef3<T>, ptr::Point1<T>),
    /// `f([t; 2])` stored as `f, [t; 2]`.
    Time2(SurfaceRef3<T>, ptr::Point2<T>),
    /// Sums two 3D vectors.
    Sum(ptr::Point3<T>, ptr::Point3<T>),
    /// Product component wise of two 3D vectors.
    Prod(ptr::Point3<T>, ptr::Point3<T>),
    /// Difference between 3D vectors.
    Diff(ptr::Point3<T>, ptr::Point3<T>),
    /// Cross product of two vectors.
    Cross(ptr::Point3<T>, ptr::Point3<T>),
    /// Maximum component wise of two 3D vectors.
    Max(ptr::Point3<T>, ptr::Point3<T>),
    /// Minimum component wise of two 3D vectors.
    Min(ptr::Point3<T>, ptr::Point3<T>),
}

/// 4D point.
#[derive(Copy, Clone, Debug)]
pub enum Point4<T> {
    /// Data point.
    Data([T; 4]),
    /// `f(t)` stored as `f, t`.
    Time1(SplineRef4<T>, ptr::Point1<T>),
    /// `f([t; 2])` stored as `f, [t; 2]`.
    Time2(SurfaceRef4<T>, ptr::Point2<T>),
    /// Sums two 4D vectors.
    Sum(ptr::Point4<T>, ptr::Point4<T>),
    /// Product component wise of two 4D vectors.
    Prod(ptr::Point4<T>, ptr::Point4<T>),
    /// Difference between two 3D vectors.
    Diff(ptr::Point4<T>, ptr::Point4<T>),
    /// Maximum component wise of two 4D vectors.
    Max(ptr::Point4<T>, ptr::Point4<T>),
    /// Minimum component wise of two 4D vectors.
    Min(ptr::Point4<T>, ptr::Point4<T>),
}

/// A spline is a function of type `t -> point`.
#[derive(Copy, Clone, Debug)]
pub enum Spline<T, U> {
    /// Linear interpolation between two points.
    Line(T, T),
    /// Quadratic bezier spline.
    QuadraticBezier(T, T, T),
    /// Cubic bezier spline.
    CubicBezier(T, T, T, T),
    /// Represents a segment of another spline controlled by an input spline.
    Segment(ptr::Spline<T>, ptr::Point1<U>, ptr::Point1<U>),
    /// Represents an intersecting line on a surface controlled by `start, end`.
    OnSurface(ptr::Surface<T>, ptr::Point2<U>, ptr::Point2<U>),
    /// Gets the contour line of a surface.
    ///
    /// ```ignore
    /// 0.0-0.25: [0.0, 0.0] -> [1.0, 0.0]
    /// 0.25-0.5: [1.0, 0.0] -> [1.0, 1.0]
    /// 0.5-0.75: [1.0, 1.0] -> [0.0, 1.0]
    /// 0.75-1.0: [0.0, 1.0] -> [0.0, 0.0]
    /// ```
    Contour(ptr::Surface<T>),
}

/// A surface is a function of type `[t; 2] -> point`.
#[derive(Copy, Clone, Debug)]
pub enum Surface<T, U> {
    /// Bilinear interpolation between 4 points.
    ///
    /// ```ignore
    /// 0 ----- 1
    /// |       |
    /// |       |
    /// 2 ----- 3
    /// ```
    Rect([T; 4]),
    /// Linear interpolation between two splines stored as `a, b`.
    ///
    /// ```ignore
    /// a -->-- a2
    /// |       |
    /// |       |
    /// b -->-- b2
    /// ```
    Lerp(ptr::Spline<T>, ptr::Spline<T>),
    /// Curved quad.
    ///
    /// ```ignore
    /// a -->-- b
    /// |       |
    /// v       v
    /// |       |
    /// c -->-- d
    /// ```
    CurvedQuad {
        /// Smooth factor.
        smooth: ptr::Point1<U>,
        /// AB spline.
        ab: ptr::Spline<T>,
        /// CD spline.
        cd: ptr::Spline<T>,
        /// AC spline.
        ac: ptr::Spline<T>,
        /// BD spline.
        bd: ptr::Spline<T>,
    },
    /// Circle stored as `center, radius`.
    /// In 1D the circle behaves like a sine wave.
    /// In 3D the circle is creates in the xy plane at z coordinate.
    Circle(T, ptr::Point1<U>),
}

/// A color is separate from points because of format and color space conversion.
#[derive(Copy, Clone, Debug)]
pub enum Color<T> {
    /// Color data.
    Data(types::Color),
    /// Get color from a color spline.
    Time1(ptr::ColorSpline, ptr::Point1<T>),
}

/// A color spline.
#[derive(Copy, Clone, Debug)]
pub enum ColorSpline {
    /// Linear interpolation between two colors.
    Lerp(ptr::Color, ptr::Color),
}

/// Bones.
#[derive(Copy, Clone, Debug)]
pub enum Bone<T, U> {
    /// Keep distance.
    Eq(T, T, ptr::Point1<U>),
    /// Keep less or equal distance.
    Less(T, T, ptr::Point1<U>),
    /// Keep larger or equal distance.
    More(T, T, ptr::Point1<U>),
}
