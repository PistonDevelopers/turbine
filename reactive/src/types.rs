use *;

/// Spline for scalars.
pub type Spline1<T> = fns::Spline<ptr::Point1<T>, T>;
/// Spline for 2D.
pub type Spline2<T> = fns::Spline<ptr::Point2<T>, T>;
/// Spline for 3D.
pub type Spline3<T> = fns::Spline<ptr::Point3<T>, T>;
/// Spline for 4D.
pub type Spline4<T> = fns::Spline<ptr::Point4<T>, T>;

/// Spline reference for scalar.
pub type SplineRef1<T> = ptr::Spline<ptr::Point1<T>>;
/// Spline reference for 2D.
pub type SplineRef2<T> = ptr::Spline<ptr::Point2<T>>;
/// Spline reference for 3D.
pub type SplineRef3<T> = ptr::Spline<ptr::Point3<T>>;
/// Spline reference for 4D.
pub type SplineRef4<T> = ptr::Spline<ptr::Point4<T>>;

/// Surface for 1D.
pub type Surface1<T> = fns::Surface<ptr::Point1<T>, T>;
/// Surface for 2D.
pub type Surface2<T> = fns::Surface<ptr::Point2<T>, T>;
/// Surface for 3D.
pub type Surface3<T> = fns::Surface<ptr::Point3<T>, T>;
/// Surface for 4D.
pub type Surface4<T> = fns::Surface<ptr::Point4<T>, T>;

/// Surface reference for scalar.
pub type SurfaceRef1<T> = ptr::Surface<ptr::Point1<T>>;
/// Surface reference for 2D.
pub type SurfaceRef2<T> = ptr::Surface<ptr::Point2<T>>;
/// Surface reference for 3D.
pub type SurfaceRef3<T> = ptr::Surface<ptr::Point3<T>>;
/// Surface reference for 4D.
pub type SurfaceRef4<T> = ptr::Surface<ptr::Point4<T>>;

/// Color component type.
pub type ColorComponent = f32;
/// Color stored as normalized `[r, g, b, a]`.
pub type Color = [ColorComponent; 4];

/// Bone for 1D.
pub type Bone1<T> = fns::Bone<ptr::Point1<T>, T>;
/// Bone for 2D.
pub type Bone2<T> = fns::Bone<ptr::Point2<T>, T>;
/// Bone for 3D.
pub type Bone3<T> = fns::Bone<ptr::Point3<T>, T>;
/// Bone for 4D.
pub type Bone4<T> = fns::Bone<ptr::Point4<T>, T>;

/// Bone reference for 1D.
pub type BoneRef1<T> = ptr::Bone<ptr::Point1<T>>;
/// Bone reference for 2D.
pub type BoneRef2<T> = ptr::Bone<ptr::Point2<T>>;
/// Bone reference for 3D.
pub type BoneRef3<T> = ptr::Bone<ptr::Point3<T>>;
/// Bone reference for 4D.
pub type BoneRef4<T> = ptr::Bone<ptr::Point4<T>>;
