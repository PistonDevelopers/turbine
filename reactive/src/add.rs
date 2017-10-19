use *;

impl<T> Reactor<T> {
    /// Adds new bool.
    pub fn add_bool(&mut self, val: bool) -> ptr::Bool {
        let n = self.bools.len();
        self.bools.push(fns::Bool::Data(val));
        n.into()
    }

    /// Adds new logical NOT.
    pub fn add_not(&mut self, id: ptr::Bool) -> ptr::Bool {
        let n = self.bools.len();
        self.bools.push(fns::Bool::Not(id));
        n.into()
    }

    /// Adds new logical AND.
    pub fn add_and(&mut self, a: ptr::Bool, b: ptr::Bool) -> ptr::Bool {
        let n = self.bools.len();
        self.bools.push(fns::Bool::And(a, b));
        n.into()
    }

    /// Adds new logical OR.
    pub fn add_or(&mut self, a: ptr::Bool, b: ptr::Bool) -> ptr::Bool {
        let n = self.bools.len();
        self.bools.push(fns::Bool::Or(a, b));
        n.into()
    }

    /// Adds new logical XOR.
    pub fn add_xor(&mut self, a: ptr::Bool, b: ptr::Bool) -> ptr::Bool {
        let n = self.bools.len();
        self.bools.push(fns::Bool::Xor(a, b));
        n.into()
    }

    /// Adds new logical EQ.
    pub fn add_eq_bool(&mut self, a: ptr::Bool, b: ptr::Bool) -> ptr::Bool {
        let n = self.bools.len();
        self.bools.push(fns::Bool::EqBool(a, b));
        n.into()
    }

    /// Adds new check number equality check.
    pub fn add_eq1(&mut self, a: ptr::Point1<T>, b: ptr::Point1<T>) -> ptr::Bool {
        let n = self.bools.len();
        self.bools.push(fns::Bool::Eq1(a, b));
        n.into()
    }

    /// Adds new 2D vector equality check.
    pub fn add_eq2(&mut self, a: ptr::Point2<T>, b: ptr::Point2<T>) -> ptr::Bool {
        let n = self.bools.len();
        self.bools.push(fns::Bool::Eq2(a, b));
        n.into()
    }

    /// Adds new 3D vector equality check.
    pub fn add_eq3(&mut self, a: ptr::Point3<T>, b: ptr::Point3<T>) -> ptr::Bool {
        let n = self.bools.len();
        self.bools.push(fns::Bool::Eq3(a, b));
        n.into()
    }

    /// Adds new 4D vector equality check.
    pub fn add_eq4(&mut self, a: ptr::Point4<T>, b: ptr::Point4<T>) -> ptr::Bool {
        let n = self.bools.len();
        self.bools.push(fns::Bool::Eq4(a, b));
        n.into()
    }

    /// Adds new less comparison.
    pub fn add_lt(&mut self, a: ptr::Point1<T>, b: ptr::Point1<T>) -> ptr::Bool {
        let n = self.bools.len();
        self.bools.push(fns::Bool::Less(a, b));
        n.into()
    }

    /// Adds new more comparison.
    /// Uses `less` by swapping arguments.
    pub fn add_gt(&mut self, a: ptr::Point1<T>, b: ptr::Point1<T>) -> ptr::Bool {
        let n = self.bools.len();
        self.bools.push(fns::Bool::Less(b, a));
        n.into()
    }

    /// Adds new greater or equal comparison.
    pub fn add_ge(&mut self, a: ptr::Point1<T>, b: ptr::Point1<T>) -> ptr::Bool {
        let n = self.bools.len();
        self.bools.push(fns::Bool::GreaterOrEqual(a, b));
        n.into()
    }

    /// Adds new less or equal comparison.
    /// Uses `greater or equal` by swapping arguments.
    pub fn add_le(&mut self, a: ptr::Point1<T>, b: ptr::Point1<T>) -> ptr::Bool {
        let n = self.bools.len();
        self.bools.push(fns::Bool::GreaterOrEqual(b, a));
        n.into()
    }

    /// Adds new scalar.
    pub fn add1(&mut self, scalar: T) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Data(scalar));
        n.into()
    }

    /// Adds new 2D point.
    pub fn add2(&mut self, pos: [T; 2]) -> ptr::Point2<T> {
        let n = self.points2.len();
        self.points2.push(fns::Point2::Data(pos));
        n.into()
    }

    /// Adds new 3D point.
    pub fn add3(&mut self, pos: [T; 3]) -> ptr::Point3<T> {
        let n = self.points3.len();
        self.points3.push(fns::Point3::Data(pos));
        n.into()
    }

    /// Adds new 4D point.
    pub fn add4(&mut self, pos: [T; 4]) -> ptr::Point4<T> {
        let n = self.points4.len();
        self.points4.push(fns::Point4::Data(pos));
        n.into()
    }

    /// Adds new 1D point from a line.
    pub fn add_line_point1(&mut self, f: SplineRef1<T>, t: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Time1(f, t));
        n.into()
    }

    /// Adds new 2D point from a line.
    pub fn add_line_point2(&mut self, f: SplineRef2<T>, t: ptr::Point1<T>) -> ptr::Point2<T> {
        let n = self.points2.len();
        self.points2.push(fns::Point2::Time1(f, t));
        n.into()
    }

    /// Adds new 3D point from a line.
    pub fn add_line_point3(&mut self, f: SplineRef3<T>, t: ptr::Point1<T>) -> ptr::Point3<T> {
        let n = self.points3.len();
        self.points3.push(fns::Point3::Time1(f, t));
        n.into()
    }

    /// Adds new 4D point from a line.
    pub fn add_line_point4(&mut self, f: SplineRef4<T>, t: ptr::Point1<T>) -> ptr::Point4<T> {
        let n = self.points4.len();
        self.points4.push(fns::Point4::Time1(f, t));
        n.into()
    }

    /// Adds new 1D point from a surface.
    pub fn add_surface_point1(&mut self, f: SurfaceRef1<T>, t: ptr::Point2<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Time2(f, t));
        n.into()
    }

    /// Adds new 2D point from a surface.
    pub fn add_surface_point2(&mut self, f: SurfaceRef2<T>, t: ptr::Point2<T>) -> ptr::Point2<T> {
        let n = self.points2.len();
        self.points2.push(fns::Point2::Time2(f, t));
        n.into()
    }

    /// Adds new 3D point from a surface.
    pub fn add_surface_point3(&mut self, f: SurfaceRef3<T>, t: ptr::Point2<T>) -> ptr::Point3<T> {
        let n = self.points3.len();
        self.points3.push(fns::Point3::Time2(f, t));
        n.into()
    }

    /// Adds new 4D point from a surface.
    pub fn add_surface_point4(&mut self, f: SurfaceRef4<T>, t: ptr::Point2<T>) -> ptr::Point4<T> {
        let n = self.points4.len();
        self.points4.push(fns::Point4::Time2(f, t));
        n.into()
    }

    /// Adds sum of two numbers.
    pub fn add_sum1(&mut self, a: ptr::Point1<T>, b: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Sum(a, b));
        n.into()
    }

    /// Adds sum of two 2D vectors.
    pub fn add_sum2(&mut self, a: ptr::Point2<T>, b: ptr::Point2<T>) -> ptr::Point2<T> {
        let n = self.points2.len();
        self.points2.push(fns::Point2::Sum(a, b));
        n.into()
    }

    /// Adds sum of two 3D vectors.
    pub fn add_sum3(&mut self, a: ptr::Point3<T>, b: ptr::Point3<T>) -> ptr::Point3<T> {
        let n = self.points3.len();
        self.points3.push(fns::Point3::Sum(a, b));
        n.into()
    }

    /// Adds sum of two 4D vectors.
    pub fn add_sum4(&mut self, a: ptr::Point4<T>, b: ptr::Point4<T>) -> ptr::Point4<T> {
        let n = self.points4.len();
        self.points4.push(fns::Point4::Sum(a, b));
        n.into()
    }

    /// Adds difference between two numbers.
    pub fn add_diff1(&mut self, a: ptr::Point1<T>, b: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Diff(a, b));
        n.into()
    }

    /// Adds difference between two 2D vectors.
    pub fn add_diff2(&mut self, a: ptr::Point2<T>, b: ptr::Point2<T>) -> ptr::Point2<T> {
        let n = self.points2.len();
        self.points2.push(fns::Point2::Diff(a, b));
        n.into()
    }

    /// Adds difference between two 3D vectors.
    pub fn add_diff3(&mut self, a: ptr::Point3<T>, b: ptr::Point3<T>) -> ptr::Point3<T> {
        let n = self.points3.len();
        self.points3.push(fns::Point3::Diff(a, b));
        n.into()
    }

    /// Adds difference between two 4D vectors.
    pub fn add_diff4(&mut self, a: ptr::Point4<T>, b: ptr::Point4<T>) -> ptr::Point4<T> {
        let n = self.points4.len();
        self.points4.push(fns::Point4::Diff(a, b));
        n.into()
    }

    /// Adds product of two numbers.
    pub fn add_prod1(&mut self, a: ptr::Point1<T>, b: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Prod(a, b));
        n.into()
    }

    /// Adds division of two numbers.
    pub fn add_div(&mut self, a: ptr::Point1<T>, b: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Div(a, b));
        n.into()
    }

    /// Adds product component wise of two 2D vectors.
    pub fn add_prod2(&mut self, a: ptr::Point2<T>, b: ptr::Point2<T>) -> ptr::Point2<T> {
        let n = self.points2.len();
        self.points2.push(fns::Point2::Prod(a, b));
        n.into()
    }

    /// Adds product component wise of two 3D vectors.
    pub fn add_prod3(&mut self, a: ptr::Point3<T>, b: ptr::Point3<T>) -> ptr::Point3<T> {
        let n = self.points3.len();
        self.points3.push(fns::Point3::Prod(a, b));
        n.into()
    }

    /// Adds product component wise of two 4D vectors.
    pub fn add_prod4(&mut self, a: ptr::Point4<T>, b: ptr::Point4<T>) -> ptr::Point4<T> {
        let n = self.points4.len();
        self.points4.push(fns::Point4::Prod(a, b));
        n.into()
    }

    /// Dot product of two 2D vectors.
    pub fn add_dot2(&mut self, a: ptr::Point2<T>, b: ptr::Point2<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Dot2(a, b));
        n.into()
    }

    /// Dot product of two 3D vectors.
    pub fn add_dot3(&mut self, a: ptr::Point3<T>, b: ptr::Point3<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Dot3(a, b));
        n.into()
    }

    /// Dot product of two 4D vectors.
    pub fn add_dot4(&mut self, a: ptr::Point4<T>, b: ptr::Point4<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Dot4(a, b));
        n.into()
    }

    /// Cross product of two 2D vectors.
    pub fn add_cross2(&mut self, a: ptr::Point2<T>, b: ptr::Point2<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Cross(a, b));
        n.into()
    }

    /// Cross product of two 3D vectors.
    pub fn add_cross3(&mut self, a: ptr::Point3<T>, b: ptr::Point3<T>) -> ptr::Point3<T> {
        let n = self.points3.len();
        self.points3.push(fns::Point3::Cross(a, b));
        n.into()
    }

    /// Adds abolute value of number.
    pub fn add_abs(&mut self, a: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Abs(a));
        n.into()
    }

    /// Adds length of 2D vector.
    pub fn add_len2(&mut self, a: ptr::Point2<T>) -> ptr::Point2<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Len2(a));
        n.into()
    }

    /// Adds length of 3D vector.
    pub fn add_len3(&mut self, a: ptr::Point3<T>) -> ptr::Point3<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Len3(a));
        n.into()
    }

    /// Adds length of 4D vector.
    pub fn add_len4(&mut self, a: ptr::Point4<T>) -> ptr::Point4<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Len4(a));
        n.into()
    }

    /// Adds negative of number.
    pub fn add_neg(&mut self, a: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Neg(a));
        n.into()
    }

    /// Adds sign of number.
    pub fn add_sign(&mut self, a: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Sign(a));
        n.into()
    }

    /// Adds sine of angle.
    pub fn add_sin(&mut self, a: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Sin(a));
        n.into()
    }

    /// Adds cosine of angle.
    pub fn add_cos(&mut self, a: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Cos(a));
        n.into()
    }

    /// Adds tangent of angle.
    pub fn add_tan(&mut self, a: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Tan(a));
        n.into()
    }

    /// Adds inverse sine of angle.
    pub fn add_asin(&mut self, a: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Asin(a));
        n.into()
    }

    /// Adds inverse cosine of angle.
    pub fn add_acos(&mut self, a: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Acos(a));
        n.into()
    }

    /// Adds inverse tangent of angle.
    pub fn add_atan(&mut self, a: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Atan(a));
        n.into()
    }

    /// Adds inverse tangent of `y, x`.
    pub fn add_atan2(&mut self, a: ptr::Point1<T>, b: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Atan2(a, b));
        n.into()
    }

    /// Adds hyperbolic sine of angle.
    pub fn add_sinh(&mut self, a: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Sinh(a));
        n.into()
    }

    /// Adds hyperbolic cosine of angle.
    pub fn add_cosh(&mut self, a: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Cosh(a));
        n.into()
    }

    /// Adds hyperbolic tangent of angle.
    pub fn add_tanh(&mut self, a: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Tanh(a));
        n.into()
    }

    /// Adds inverse hyperbolic sine of angle.
    pub fn add_asinh(&mut self, a: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Asinh(a));
        n.into()
    }

    /// Adds inverse hyperbolic cosine of angle.
    pub fn add_acosh(&mut self, a: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Acosh(a));
        n.into()
    }

    /// Adds inverse hyperbolic tangent of angle.
    pub fn add_atanh(&mut self, a: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Atanh(a));
        n.into()
    }

    /// Adds maximum of two numbers.
    pub fn add_max1(&mut self, a: ptr::Point1<T>, b: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Max(a, b));
        n.into()
    }

    /// Adds maximum of two 2D vectors.
    pub fn add_max2(&mut self, a: ptr::Point2<T>, b: ptr::Point2<T>) -> ptr::Point2<T> {
        let n = self.points2.len();
        self.points2.push(fns::Point2::Max(a, b));
        n.into()
    }

    /// Adds maximum of two 3D vectors.
    pub fn add_max3(&mut self, a: ptr::Point3<T>, b: ptr::Point3<T>) -> ptr::Point3<T> {
        let n = self.points3.len();
        self.points3.push(fns::Point3::Max(a, b));
        n.into()
    }

    /// Adds maximum of two 4D vectors.
    pub fn add_max4(&mut self, a: ptr::Point4<T>, b: ptr::Point4<T>) -> ptr::Point4<T> {
        let n = self.points4.len();
        self.points4.push(fns::Point4::Max(a, b));
        n.into()
    }

    /// Adds minimum of two numbers.
    pub fn add_min1(&mut self, a: ptr::Point1<T>, b: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Min(a, b));
        n.into()
    }

    /// Adds minimum of two 2D vectors.
    pub fn add_min2(&mut self, a: ptr::Point2<T>, b: ptr::Point2<T>) -> ptr::Point2<T> {
        let n = self.points2.len();
        self.points2.push(fns::Point2::Min(a, b));
        n.into()
    }

    /// Adds minimum of two 3D vectors.
    pub fn add_min3(&mut self, a: ptr::Point3<T>, b: ptr::Point3<T>) -> ptr::Point3<T> {
        let n = self.points3.len();
        self.points3.push(fns::Point3::Min(a, b));
        n.into()
    }

    /// Adds minimum of two 4D vectors.
    pub fn add_min4(&mut self, a: ptr::Point4<T>, b: ptr::Point4<T>) -> ptr::Point4<T> {
        let n = self.points4.len();
        self.points4.push(fns::Point4::Min(a, b));
        n.into()
    }

    /// Adds square root of number.
    pub fn add_sqrt(&mut self, a: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::Sqrt(a));
        n.into()
    }

    /// Adds conversion from degrees to radians.
    pub fn add_deg_to_rad(&mut self, a: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::DegToRad(a));
        n.into()
    }

    /// Adds conversion from radians to degrees.
    pub fn add_rad_to_deg(&mut self, a: ptr::Point1<T>) -> ptr::Point1<T> {
        let n = self.points1.len();
        self.points1.push(fns::Point1::RadToDeg(a));
        n.into()
    }

    /// Adds time.
    pub fn add_time(&mut self) -> ptr::Point1<T> where T: Copy {
        if let Some(id) = self.time {
            id
        } else {
            let n = self.points1.len();
            self.points1.push(fns::Point1::Time);
            self.time = Some(n.into());
            n.into()
        }
    }

    /// Adds delta time.
    pub fn add_dt(&mut self) -> ptr::Point1<T> where T: Copy {
        if let Some(id) = self.dt {
            id
        } else {
            let n = self.points1.len();
            self.points1.push(fns::Point1::DeltaTime);
            self.dt = Some(n.into());
            n.into()
        }
    }

    /// Adds new 1D line.
    pub fn add_line1(&mut self, a: ptr::Point1<T>, b: ptr::Point1<T>) -> SplineRef1<T> {
        let n = self.splines1.len();
        self.splines1.push(fns::Spline::Line(a, b));
        n.into()
    }

    /// Adds new 2D line.
    pub fn add_line2(&mut self, a: ptr::Point2<T>, b: ptr::Point2<T>) -> SplineRef2<T> {
        let n = self.splines2.len();
        self.splines2.push(fns::Spline::Line(a, b));
        n.into()
    }

    /// Adds new 3D line.
    pub fn add_line3(&mut self, a: ptr::Point3<T>, b: ptr::Point3<T>) -> SplineRef3<T> {
        let n = self.splines3.len();
        self.splines3.push(fns::Spline::Line(a, b));
        n.into()
    }

    /// Adds new 4D line.
    pub fn add_line4(&mut self, a: ptr::Point4<T>, b: ptr::Point4<T>) -> SplineRef4<T> {
        let n = self.splines4.len();
        self.splines4.push(fns::Spline::Line(a, b));
        n.into()
    }

    /// Adds new 1D quadratic bezier spline.
    pub fn add_qbez1(
        &mut self,
        a: ptr::Point1<T>,
        b: ptr::Point1<T>,
        c: ptr::Point1<T>
    ) -> SplineRef1<T> {
        let n = self.splines1.len();
        self.splines1.push(fns::Spline::QuadraticBezier(a, b, c));
        n.into()
    }

    /// Adds new 2D quadratic bezier spline.
    pub fn add_qbez2(
        &mut self,
        a: ptr::Point2<T>,
        b: ptr::Point2<T>,
        c: ptr::Point2<T>
    ) -> SplineRef2<T> {
        let n = self.splines2.len();
        self.splines2.push(fns::Spline::QuadraticBezier(a, b, c));
        n.into()
    }

    /// Adds new 3D quadratic bezier spline.
    pub fn add_qbez3(
        &mut self,
        a: ptr::Point3<T>,
        b: ptr::Point3<T>,
        c: ptr::Point3<T>
    ) -> SplineRef3<T> {
        let n = self.splines3.len();
        self.splines3.push(fns::Spline::QuadraticBezier(a, b, c));
        n.into()
    }

    /// Adds new 4D quadratic bezier spline.
    pub fn add_qbez4(
        &mut self,
        a: ptr::Point4<T>,
        b: ptr::Point4<T>,
        c: ptr::Point4<T>
    ) -> SplineRef4<T> {
        let n = self.splines4.len();
        self.splines4.push(fns::Spline::QuadraticBezier(a, b, c));
        n.into()
    }

    /// Adds new 1D cubic bezier spline.
    pub fn add_cbez1(
        &mut self,
        a: ptr::Point1<T>,
        b: ptr::Point1<T>,
        c: ptr::Point1<T>,
        d: ptr::Point1<T>
    ) -> SplineRef1<T> {
        let n = self.splines1.len();
        self.splines1.push(fns::Spline::CubicBezier(a, b, c, d));
        n.into()
    }

    /// Adds new 2D cubic bezier spline.
    pub fn add_cbez2(
        &mut self,
        a: ptr::Point2<T>,
        b: ptr::Point2<T>,
        c: ptr::Point2<T>,
        d: ptr::Point2<T>
    ) -> SplineRef2<T> {
        let n = self.splines2.len();
        self.splines2.push(fns::Spline::CubicBezier(a, b, c, d));
        n.into()
    }

    /// Adds new 3D cubic bezier spline.
    pub fn add_cbez3(
        &mut self,
        a: ptr::Point3<T>,
        b: ptr::Point3<T>,
        c: ptr::Point3<T>,
        d: ptr::Point3<T>
    ) -> SplineRef3<T> {
        let n = self.splines3.len();
        self.splines3.push(fns::Spline::CubicBezier(a, b, c, d));
        n.into()
    }

    /// Adds new 4D cubic bezier spline.
    pub fn add_cbez4(
        &mut self,
        a: ptr::Point4<T>,
        b: ptr::Point4<T>,
        c: ptr::Point4<T>,
        d: ptr::Point4<T>
    ) -> SplineRef4<T> {
        let n = self.splines4.len();
        self.splines4.push(fns::Spline::CubicBezier(a, b, c, d));
        n.into()
    }

    /// Adds new 1D segment from line.
    pub fn add_line_segment1(
        &mut self,
        f: SplineRef1<T>,
        start: ptr::Point1<T>,
        end: ptr::Point1<T>
    ) -> SplineRef1<T> {
        let n = self.splines1.len();
        self.splines1.push(fns::Spline::Segment(f, start, end));
        n.into()
    }

    /// Adds new 2D segment from line.
    pub fn add_line_segment2(
        &mut self,
        f: SplineRef2<T>,
        start: ptr::Point1<T>,
        end: ptr::Point1<T>
    ) -> SplineRef2<T> {
        let n = self.splines2.len();
        self.splines2.push(fns::Spline::Segment(f, start, end));
        n.into()
    }

    /// Adds new 3D segment from line.
    pub fn add_line_segment3(
        &mut self,
        f: SplineRef3<T>,
        start: ptr::Point1<T>,
        end: ptr::Point1<T>
    ) -> SplineRef3<T> {
        let n = self.splines3.len();
        self.splines3.push(fns::Spline::Segment(f, start, end));
        n.into()
    }

    /// Adds new 4D segment from line.
    pub fn add_line_segment4(
        &mut self,
        f: SplineRef4<T>,
        start: ptr::Point1<T>,
        end: ptr::Point1<T>
    ) -> SplineRef4<T> {
        let n = self.splines4.len();
        self.splines4.push(fns::Spline::Segment(f, start, end));
        n.into()
    }

    /// Adds new 1D line on surface.
    pub fn add_line_on_surface1(
        &mut self,
        f: SurfaceRef1<T>,
        start: ptr::Point2<T>,
        end: ptr::Point2<T>
    ) -> SplineRef1<T> {
        let n = self.splines1.len();
        self.splines1.push(fns::Spline::OnSurface(f, start, end));
        n.into()
    }

    /// Adds new 2D line on surface.
    pub fn add_line_on_surface2(
        &mut self,
        f: SurfaceRef2<T>,
        start: ptr::Point2<T>,
        end: ptr::Point2<T>
    ) -> SplineRef2<T> {
        let n = self.splines2.len();
        self.splines2.push(fns::Spline::OnSurface(f, start, end));
        n.into()
    }

    /// Adds new 3D line on surface.
    pub fn add_line_on_surface3(
        &mut self,
        f: SurfaceRef3<T>,
        start: ptr::Point2<T>,
        end: ptr::Point2<T>
    ) -> SplineRef3<T> {
        let n = self.splines3.len();
        self.splines3.push(fns::Spline::OnSurface(f, start, end));
        n.into()
    }

    /// Adds new 4D line on surface.
    pub fn add_line_on_surface4(
        &mut self,
        f: SurfaceRef4<T>,
        start: ptr::Point2<T>,
        end: ptr::Point2<T>
    ) -> SplineRef4<T> {
        let n = self.splines4.len();
        self.splines4.push(fns::Spline::OnSurface(f, start, end));
        n.into()
    }

    /// Adds new 1D contour of surface.
    pub fn add_contour1(&mut self, f: SurfaceRef1<T>) -> SplineRef1<T> {
        let n = self.splines1.len();
        self.splines1.push(fns::Spline::Contour(f));
        n.into()
    }

    /// Adds new 2D contour of surface.
    pub fn add_contour2(&mut self, f: SurfaceRef2<T>) -> SplineRef2<T> {
        let n = self.splines2.len();
        self.splines2.push(fns::Spline::Contour(f));
        n.into()
    }

    /// Adds new 3D contour of surface.
    pub fn add_contour3(&mut self, f: SurfaceRef3<T>) -> SplineRef3<T> {
        let n = self.splines3.len();
        self.splines3.push(fns::Spline::Contour(f));
        n.into()
    }

    /// Adds new 4D contour of surface.
    pub fn add_contour4(&mut self, f: SurfaceRef4<T>) -> SplineRef4<T> {
        let n = self.splines4.len();
        self.splines4.push(fns::Spline::Contour(f));
        n.into()
    }

    /// Adds new 1D rectangle.
    pub fn add_rect1(&mut self, val: [ptr::Point1<T>; 4]) -> SurfaceRef1<T> {
        let n = self.surfaces1.len();
        self.surfaces1.push(fns::Surface::Rect(val));
        n.into()
    }

    /// Adds new 2D rectangle.
    pub fn add_rect2(&mut self, val: [ptr::Point2<T>; 4]) -> SurfaceRef2<T> {
        let n = self.surfaces2.len();
        self.surfaces2.push(fns::Surface::Rect(val));
        n.into()
    }

    /// Adds new 3D rectangle.
    pub fn add_rect3(&mut self, val: [ptr::Point3<T>; 4]) -> SurfaceRef3<T> {
        let n = self.surfaces3.len();
        self.surfaces3.push(fns::Surface::Rect(val));
        n.into()
    }

    /// Adds new 4D rectangle.
    pub fn add_rect4(&mut self, val: [ptr::Point4<T>; 4]) -> SurfaceRef4<T> {
        let n = self.surfaces4.len();
        self.surfaces4.push(fns::Surface::Rect(val));
        n.into()
    }

    /// Adds new 1D surface lerp.
    pub fn add_lerp_surface1(&mut self, a: SplineRef1<T>, b: SplineRef1<T>) -> SurfaceRef1<T> {
        let n = self.surfaces1.len();
        self.surfaces1.push(fns::Surface::Lerp(a, b));
        n.into()
    }

    /// Adds new 2D surface lerp.
    pub fn add_lerp_surface2(&mut self, a: SplineRef2<T>, b: SplineRef2<T>) -> SurfaceRef2<T> {
        let n = self.surfaces2.len();
        self.surfaces2.push(fns::Surface::Lerp(a, b));
        n.into()
    }

    /// Adds new 3D surface lerp.
    pub fn add_lerp_surface3(&mut self, a: SplineRef3<T>, b: SplineRef3<T>) -> SurfaceRef3<T> {
        let n = self.surfaces3.len();
        self.surfaces3.push(fns::Surface::Lerp(a, b));
        n.into()
    }

    /// Adds new 4D surface lerp.
    pub fn add_lerp_surface4(&mut self, a: SplineRef4<T>, b: SplineRef4<T>) -> SurfaceRef4<T> {
        let n = self.surfaces4.len();
        self.surfaces4.push(fns::Surface::Lerp(a, b));
        n.into()
    }

    /// Adds new 1D curved quad.
    pub fn add_cquad1(
        &mut self,
        smooth: ptr::Point1<T>,
        ab: SplineRef1<T>,
        cd: SplineRef1<T>,
        ac: SplineRef1<T>,
        bd: SplineRef1<T>
    ) -> SurfaceRef1<T> {
        let n = self.surfaces1.len();
        self.surfaces1.push(fns::Surface::CurvedQuad {smooth, ab, cd, ac, bd});
        n.into()
    }

    /// Adds new 2D curved quad.
    pub fn add_cquad2(
        &mut self,
        smooth: ptr::Point1<T>,
        ab: SplineRef2<T>,
        cd: SplineRef2<T>,
        ac: SplineRef2<T>,
        bd: SplineRef2<T>
    ) -> SurfaceRef2<T> {
        let n = self.surfaces2.len();
        self.surfaces2.push(fns::Surface::CurvedQuad {smooth, ab, cd, ac, bd});
        n.into()
    }

    /// Adds new 3D curved quad.
    pub fn add_cquad3(
        &mut self,
        smooth: ptr::Point1<T>,
        ab: SplineRef3<T>,
        cd: SplineRef3<T>,
        ac: SplineRef3<T>,
        bd: SplineRef3<T>
    ) -> SurfaceRef3<T> {
        let n = self.surfaces3.len();
        self.surfaces3.push(fns::Surface::CurvedQuad {smooth, ab, cd, ac, bd});
        n.into()
    }

    /// Adds new 4D curved quad.
    pub fn add_cquad4(
        &mut self,
        smooth: ptr::Point1<T>,
        ab: SplineRef4<T>,
        cd: SplineRef4<T>,
        ac: SplineRef4<T>,
        bd: SplineRef4<T>
    ) -> SurfaceRef4<T> {
        let n = self.surfaces4.len();
        self.surfaces4.push(fns::Surface::CurvedQuad {smooth, ab, cd, ac, bd});
        n.into()
    }

    /// Adds new circle in 1D (sine wave).
    pub fn add_circle1(
        &mut self,
        center: ptr::Point1<T>,
        radius: ptr::Point1<T>
    ) -> SurfaceRef1<T> {
        let n = self.surfaces1.len();
        self.surfaces1.push(fns::Surface::Circle(center, radius));
        n.into()
    }

    /// Adds new 2D circle.
    pub fn add_circle2(
        &mut self,
        center: ptr::Point2<T>,
        radius: ptr::Point1<T>
    ) -> SurfaceRef2<T> {
        let n = self.surfaces2.len();
        self.surfaces2.push(fns::Surface::Circle(center, radius));
        n.into()
    }

    /// Adds new 3D circle.
    pub fn add_circle3(
        &mut self,
        center: ptr::Point3<T>,
        radius: ptr::Point1<T>
    ) -> SurfaceRef3<T> {
        let n = self.surfaces3.len();
        self.surfaces3.push(fns::Surface::Circle(center, radius));
        n.into()
    }

    /// Adds new 4D circle.
    pub fn add_circle4(
        &mut self,
        center: ptr::Point4<T>,
        radius: ptr::Point1<T>
    ) -> SurfaceRef4<T> {
        let n = self.surfaces4.len();
        self.surfaces4.push(fns::Surface::Circle(center, radius));
        n.into()
    }

    /// Adds new color.
    /// Converts from sRGB to linear color space for easier manipulation internally.
    pub fn add_color(&mut self, color: Color) -> ptr::Color {
        use color::gamma_srgb_to_linear;

        let n = self.colors.len();
        self.colors.push(fns::Color::Data(gamma_srgb_to_linear(color)));
        n.into()
    }

    /// Adds new lerp color.
    pub fn add_lerp_color(&mut self, a: ptr::Color, b: ptr::Color) -> ptr::ColorSpline {
        let n = self.color_splines.len();
        self.color_splines.push(fns::ColorSpline::Lerp(a, b));
        n.into()
    }

    /// Adds new eq bone between scalars.
    pub fn add_eq_bone1(
        &mut self,
        a: ptr::Point1<T>,
        b: ptr::Point1<T>,
        dist: ptr::Point1<T>
    ) -> BoneRef1<T> {
        let n = self.bones1.len();
        self.bones1.push(fns::Bone::Eq(a, b, dist));
        n.into()
    }

    /// Adds new eq bone between 2D points.
    pub fn add_eq_bone2(
        &mut self,
        a: ptr::Point2<T>,
        b: ptr::Point2<T>,
        dist: ptr::Point1<T>
    ) -> BoneRef2<T> {
        let n = self.bones2.len();
        self.bones2.push(fns::Bone::Eq(a, b, dist));
        n.into()
    }

    /// Adds new eq bone between 3D points.
    pub fn add_eq_bone3(
        &mut self,
        a: ptr::Point3<T>,
        b: ptr::Point3<T>,
        dist: ptr::Point1<T>
    ) -> BoneRef3<T> {
        let n = self.bones3.len();
        self.bones3.push(fns::Bone::Eq(a, b, dist));
        n.into()
    }

    /// Adds new eq bone between 4D points.
    pub fn add_eq_bone4(
        &mut self,
        a: ptr::Point4<T>,
        b: ptr::Point4<T>,
        dist: ptr::Point1<T>
    ) -> BoneRef4<T> {
        let n = self.bones4.len();
        self.bones4.push(fns::Bone::Eq(a, b, dist));
        n.into()
    }

    /// Adds new less bone between scalars.
    pub fn add_less_bone1(
        &mut self,
        a: ptr::Point1<T>,
        b: ptr::Point1<T>,
        dist: ptr::Point1<T>
    ) -> BoneRef1<T> {
        let n = self.bones1.len();
        self.bones1.push(fns::Bone::Less(a, b, dist));
        n.into()
    }

    /// Adds new less bone between 2D vectors.
    pub fn add_less_bone2(
        &mut self,
        a: ptr::Point2<T>,
        b: ptr::Point2<T>,
        dist: ptr::Point1<T>
    ) -> BoneRef2<T> {
        let n = self.bones2.len();
        self.bones2.push(fns::Bone::Less(a, b, dist));
        n.into()
    }

    /// Adds new less bone between 3D vectors.
    pub fn add_less_bone3(
        &mut self,
        a: ptr::Point3<T>,
        b: ptr::Point3<T>,
        dist: ptr::Point1<T>
    ) -> BoneRef3<T> {
        let n = self.bones3.len();
        self.bones3.push(fns::Bone::Less(a, b, dist));
        n.into()
    }

    /// Adds new less bone between 4D vectors.
    pub fn add_less_bone4(
        &mut self,
        a: ptr::Point4<T>,
        b: ptr::Point4<T>,
        dist: ptr::Point1<T>
    ) -> BoneRef4<T> {
        let n = self.bones4.len();
        self.bones4.push(fns::Bone::Less(a, b, dist));
        n.into()
    }

    /// Adds new more bone between scalars.
    pub fn add_more_bone1(
        &mut self,
        a: ptr::Point1<T>,
        b: ptr::Point1<T>,
        dist: ptr::Point1<T>
    ) -> BoneRef1<T> {
        let n = self.bones1.len();
        self.bones1.push(fns::Bone::More(a, b, dist));
        n.into()
    }

    /// Adds new more bone between 2D vectors.
    pub fn add_more_bone2(
        &mut self,
        a: ptr::Point2<T>,
        b: ptr::Point2<T>,
        dist: ptr::Point1<T>
    ) -> BoneRef2<T> {
        let n = self.bones2.len();
        self.bones2.push(fns::Bone::More(a, b, dist));
        n.into()
    }

    /// Adds new more bone between 3D vectors.
    pub fn add_more_bone3(
        &mut self,
        a: ptr::Point3<T>,
        b: ptr::Point3<T>,
        dist: ptr::Point1<T>
    ) -> BoneRef3<T> {
        let n = self.bones3.len();
        self.bones3.push(fns::Bone::More(a, b, dist));
        n.into()
    }

    /// Adds new more bone between 4D vectors.
    pub fn add_more_bone4(
        &mut self,
        a: ptr::Point4<T>,
        b: ptr::Point4<T>,
        dist: ptr::Point1<T>
    ) -> BoneRef4<T> {
        let n = self.bones4.len();
        self.bones4.push(fns::Bone::More(a, b, dist));
        n.into()
    }
}
