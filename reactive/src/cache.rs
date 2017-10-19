use *;

use fnv::FnvHashMap as HashMap;

/// Used to store temporary values of shared functions to speed up computation.
/// By default, all function outputs that are referenced more than once are cached.
/// To signal that some function output should be cached, `None` is stored in a hash map.
/// When performing the first computation of that output, `Some(value)` is stored in the cache.
pub struct Cache<T> {
    /// Stores temporary values of scalars.
    pub points1: HashMap<usize, Option<T>>,
    /// Stores temporary values of 2D vectors.
    pub points2: HashMap<usize, Option<[T; 2]>>,
    /// Stores temporary values of 3D vectors.
    pub points3: HashMap<usize, Option<[T; 3]>>,
    /// Stores temporary values of 4D vectors.
    pub points4: HashMap<usize, Option<[T; 4]>>,
}

impl<T> Cache<T> {
    /// Creates a new cache for a document.
    ///
    /// Analyzes the graph to find function outputs that are referenced more than once.
    ///
    /// If new data and functions are added after the cache is created,
    /// then these will not be automatically cached, but the behavior should be the same.
    pub fn new(doc: &Reactor<T>) -> Cache<T> where T: Copy {
        let mut cache = Cache {
            points1: HashMap::default(),
            points2: HashMap::default(),
            points3: HashMap::default(),
            points4: HashMap::default(),
        };

        let ref_count = RefCount::new(doc);

        // Go through references and add non-data ones to the cache for later use.
        let add1 = |id: ptr::Point1<T>| if let fns::Point1::Data(_) = doc[id] {false} else {true};
        let add2 = |id: ptr::Point2<T>| if let fns::Point2::Data(_) = doc[id] {false} else {true};
        let add3 = |id: ptr::Point3<T>| if let fns::Point3::Data(_) = doc[id] {false} else {true};
        let add4 = |id: ptr::Point4<T>| if let fns::Point4::Data(_) = doc[id] {false} else {true};

        // Add all variables that are an output of a function and used more than once.
        for (i, &r) in ref_count.points1.iter().enumerate() {
            if r > 1 && add1(i.into()) {cache.points1.insert(i, None);}
        }
        for (i, &r) in ref_count.points2.iter().enumerate() {
            if r > 1 && add2(i.into()) {cache.points1.insert(i, None);}
        }
        for (i, &r) in ref_count.points3.iter().enumerate() {
            if r > 1 && add3(i.into()) {cache.points2.insert(i, None);}
        }
        for (i, &r) in ref_count.points4.iter().enumerate() {
            if r > 1 && add4(i.into()) {cache.points4.insert(i, None);}
        }

        cache
    }

    /// Reset cache such that new temporary values can be calculated.
    pub fn clear(&mut self) {
        for value in self.points1.values_mut() {*value = None;}
        for value in self.points2.values_mut() {*value = None;}
        for value in self.points3.values_mut() {*value = None;}
        for value in self.points4.values_mut() {*value = None;}
    }
}

pub struct RefCount {
    points1: Vec<u32>,
    points2: Vec<u32>,
    points3: Vec<u32>,
    points4: Vec<u32>,
}

impl RefCount {
    pub fn new<T>(doc: &Reactor<T>) -> RefCount where T: Copy {
        // Create a table to count the number of references.
        let mut ref_count_points1 = vec![0; doc.points1.len()];
        let mut ref_count_points2 = vec![0; doc.points2.len()];
        let mut ref_count_points3 = vec![0; doc.points3.len()];
        let mut ref_count_points4 = vec![0; doc.points4.len()];

        {
            let mut inc_point1 = |id: ptr::Point1<T>| ref_count_points1[usize::from(id)] += 1;
            let mut inc_point2 = |id: ptr::Point2<T>| ref_count_points2[usize::from(id)] += 1;
            let mut inc_point3 = |id: ptr::Point3<T>| ref_count_points3[usize::from(id)] += 1;
            let mut inc_point4 = |id: ptr::Point4<T>| ref_count_points4[usize::from(id)] += 1;

            for b in &doc.bools {
                use fns::Bool::*;

                match *b {
                    Data(_) => {}
                    Not(_) => {}
                    And(_, _) => {}
                    Or(_, _) => {}
                    Xor(_, _) => {}
                    EqBool(_, _) => {}
                    Eq1(a, b) => {
                        inc_point1(a);
                        inc_point1(b);
                    }
                    Eq2(a, b) => {
                        inc_point2(a);
                        inc_point2(b);
                    }
                    Eq3(a, b) => {
                        inc_point3(a);
                        inc_point3(b);
                    }
                    Eq4(a, b) => {
                        inc_point4(a);
                        inc_point4(b);
                    }
                    Less(a, b) => {
                        inc_point1(a);
                        inc_point1(b);
                    }
                    GreaterOrEqual(a, b) => {
                        inc_point1(a);
                        inc_point1(b);
                    }
                }
            }

            for p in &doc.points1 {
                use fns::Point1::*;

                match *p {
                    Data(_) => {}
                    Time1(_, a) => inc_point1(a),
                    Time2(_, a) => inc_point2(a),
                    Sum(a, b) => {
                        inc_point1(a);
                        inc_point1(b);
                    }
                    Diff(a, b) => {
                        inc_point1(a);
                        inc_point1(b);
                    }
                    Prod(a, b) => {
                        inc_point1(a);
                        inc_point1(b);
                    }
                    Div(a, b) => {
                        inc_point1(a);
                        inc_point1(b);
                    }
                    Dot2(a, b) => {
                        inc_point2(a);
                        inc_point2(b);
                    }
                    Dot3(a, b) => {
                        inc_point3(a);
                        inc_point3(b);
                    }
                    Dot4(a, b) => {
                        inc_point4(a);
                        inc_point4(b);
                    }
                    Cross(a, b) => {
                        inc_point2(a);
                        inc_point2(b);
                    }
                    Abs(a) => inc_point1(a),
                    Len2(a) => inc_point2(a),
                    Len3(a) => inc_point3(a),
                    Len4(a) => inc_point4(a),
                    Neg(a) => inc_point1(a),
                    Sign(a) => inc_point1(a),
                    Sin(a) => inc_point1(a),
                    Cos(a) => inc_point1(a),
                    Tan(a) => inc_point1(a),
                    Asin(a) => inc_point1(a),
                    Acos(a) => inc_point1(a),
                    Atan(a) => inc_point1(a),
                    Atan2(a, b) => {
                        inc_point1(a);
                        inc_point1(b);
                    }
                    Sinh(a) => inc_point1(a),
                    Cosh(a) => inc_point1(a),
                    Tanh(a) => inc_point1(a),
                    Asinh(a) => inc_point1(a),
                    Acosh(a) => inc_point1(a),
                    Atanh(a) => inc_point1(a),
                    Sqrt(a) => inc_point1(a),
                    Max(a, b) => {
                        inc_point1(a);
                        inc_point1(b);
                    }
                    Min(a, b) => {
                        inc_point1(a);
                        inc_point1(b);
                    }
                    DegToRad(a) => inc_point1(a),
                    RadToDeg(a) => inc_point1(a),
                    Time | DeltaTime => {}
                }
            }

            for p in &doc.points2 {
                use fns::Point2::*;

                match *p {
                    Data(_) => {}
                    Time1(_, a) => inc_point1(a),
                    Time2(_, a) => inc_point2(a),
                    Sum(a, b) => {
                        inc_point2(a);
                        inc_point2(b);
                    }
                    Prod(a, b) => {
                        inc_point2(a);
                        inc_point2(b);
                    }
                    Diff(a, b) => {
                        inc_point2(a);
                        inc_point2(b);
                    }
                    Max(a, b) => {
                        inc_point2(a);
                        inc_point2(b);
                    }
                    Min(a, b) => {
                        inc_point2(a);
                        inc_point2(b);
                    }
                }
            }

            for p in &doc.points3 {
                use fns::Point3::*;

                match *p {
                    Data(_) => {}
                    Time1(_, a) => inc_point1(a),
                    Time2(_, a) => inc_point2(a),
                    Sum(a, b) => {
                        inc_point3(a);
                        inc_point3(b);
                    }
                    Prod(a, b) => {
                        inc_point3(a);
                        inc_point3(b);
                    }
                    Diff(a, b) => {
                        inc_point3(a);
                        inc_point3(b);
                    }
                    Cross(a, b) => {
                        inc_point3(a);
                        inc_point3(b);
                    }
                    Max(a, b) => {
                        inc_point3(a);
                        inc_point3(b);
                    }
                    Min(a, b) => {
                        inc_point3(a);
                        inc_point3(b);
                    }
                }
            }

            for p in &doc.points4 {
                use fns::Point4::*;

                match *p {
                    Data(_) => {}
                    Time1(_, a) => inc_point1(a),
                    Time2(_, a) => inc_point2(a),
                    Sum(a, b) => {
                        inc_point4(a);
                        inc_point4(b);
                    }
                    Prod(a, b) => {
                        inc_point4(a);
                        inc_point4(b);
                    }
                    Diff(a, b) => {
                        inc_point4(a);
                        inc_point4(b);
                    }
                    Max(a, b) => {
                        inc_point4(a);
                        inc_point4(b);
                    }
                    Min(a, b) => {
                        inc_point4(a);
                        inc_point4(b);
                    }
                }
            }

            for f in &doc.splines1 {
                use fns::Spline::*;

                match *f {
                    Line(a, b) => {
                        inc_point1(a);
                        inc_point1(b);
                    }
                    QuadraticBezier(a, b, c) => {
                        inc_point1(a);
                        inc_point1(b);
                        inc_point1(c);
                    }
                    CubicBezier(a, b, c, d) => {
                        inc_point1(a);
                        inc_point1(b);
                        inc_point1(c);
                        inc_point1(d);
                    }
                    Segment(_, a, b) => {
                        inc_point1(a);
                        inc_point1(b);
                    }
                    OnSurface(_, a, b) => {
                        inc_point2(a);
                        inc_point2(b);
                    }
                    Contour(_) => {}
                }
            }

            for f in &doc.splines2 {
                use fns::Spline::*;

                match *f {
                    Line(a, b) => {
                        inc_point2(a);
                        inc_point2(b);
                    }
                    QuadraticBezier(a, b, c) => {
                        inc_point2(a);
                        inc_point2(b);
                        inc_point2(c);
                    }
                    CubicBezier(a, b, c, d) => {
                        inc_point2(a);
                        inc_point2(b);
                        inc_point2(c);
                        inc_point2(d);
                    }
                    Segment(_, a, b) => {
                        inc_point1(a);
                        inc_point1(b);
                    }
                    OnSurface(_, a, b) => {
                        inc_point2(a);
                        inc_point2(b);
                    }
                    Contour(_) => {}
                }
            }

            for f in &doc.splines3 {
                use fns::Spline::*;

                match *f {
                    Line(a, b) => {
                        inc_point3(a);
                        inc_point3(b);
                    }
                    QuadraticBezier(a, b, c) => {
                        inc_point3(a);
                        inc_point3(b);
                        inc_point3(c);
                    }
                    CubicBezier(a, b, c, d) => {
                        inc_point3(a);
                        inc_point3(b);
                        inc_point3(c);
                        inc_point3(d);
                    }
                    Segment(_, a, b) => {
                        inc_point1(a);
                        inc_point1(b);
                    }
                    OnSurface(_, a, b) => {
                        inc_point2(a);
                        inc_point2(b);
                    }
                    Contour(_) => {}
                }
            }

            for f in &doc.splines4 {
                use fns::Spline::*;

                match *f {
                    Line(a, b) => {
                        inc_point4(a);
                        inc_point4(b);
                    }
                    QuadraticBezier(a, b, c) => {
                        inc_point4(a);
                        inc_point4(b);
                        inc_point4(c);
                    }
                    CubicBezier(a, b, c, d) => {
                        inc_point4(a);
                        inc_point4(b);
                        inc_point4(c);
                        inc_point4(d);
                    }
                    Segment(_, a, b) => {
                        inc_point1(a);
                        inc_point1(b);
                    }
                    OnSurface(_, a, b) => {
                        inc_point2(a);
                        inc_point2(b);
                    }
                    Contour(_) => {}
                }
            }

            for f in &doc.surfaces1 {
                use fns::Surface::*;

                match *f {
                    Rect(rect) => {
                        for &p in &rect[..] {
                            inc_point1(p)
                        }
                    }
                    Lerp(_, _) => {}
                    CurvedQuad { smooth, .. } => inc_point1(smooth),
                    Circle(center, radius) => {
                        inc_point1(center);
                        inc_point1(radius);
                    }
                }
            }

            for f in &doc.surfaces2 {
                use fns::Surface::*;

                match *f {
                    Rect(rect) => {
                        for &p in &rect[..] {
                            inc_point2(p)
                        }
                    }
                    Lerp(_, _) => {}
                    CurvedQuad { smooth, .. } => inc_point1(smooth),
                    Circle(center, radius) => {
                        inc_point2(center);
                        inc_point1(radius);
                    }
                }
            }

            for f in &doc.surfaces3 {
                use fns::Surface::*;

                match *f {
                    Rect(rect) => {
                        for &p in &rect[..] {
                            inc_point3(p)
                        }
                    }
                    Lerp(_, _) => {}
                    CurvedQuad { smooth, .. } => inc_point1(smooth),
                    Circle(center, radius) => {
                        inc_point3(center);
                        inc_point1(radius);
                    }
                }
            }

            for f in &doc.surfaces4 {
                use fns::Surface::*;

                match *f {
                    Rect(rect) => {
                        for &p in &rect[..] {
                            inc_point4(p)
                        }
                    }
                    Lerp(_, _) => {}
                    CurvedQuad { smooth, .. } => inc_point1(smooth),
                    Circle(center, radius) => {
                        inc_point4(center);
                        inc_point1(radius);
                    }
                }
            }

            for c in &doc.colors {
                use fns::Color::*;

                match *c {
                    Data(_) => {}
                    Time1(_, a) => inc_point1(a),
                }
            }
        }

        RefCount {
            points1: ref_count_points1,
            points2: ref_count_points2,
            points3: ref_count_points3,
            points4: ref_count_points4,
        }
    }
}
