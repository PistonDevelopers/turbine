use *;

impl<T> Reactor<T> {
    /// Evaluate bool.
    pub fn eval_bool(&self, id: ptr::Bool, env: &mut Environment<T>) -> bool
        where T: Float, f64: Cast<T>
    {
        use fns::Bool::*;

        match self[id] {
            Data(a) => a,
            Not(a) => !self.eval_bool(a, env),
            And(a, b) => {
                let a = self.eval_bool(a, env);
                let b = self.eval_bool(b, env);
                a && b
            }
            Or(a, b) => {
                let a = self.eval_bool(a, env);
                let b = self.eval_bool(b, env);
                a || b
            }
            Xor(a, b) => {
                let a = self.eval_bool(a, env);
                let b = self.eval_bool(b, env);
                a ^ b
            }
            EqBool(a, b) => {
                let a = self.eval_bool(a, env);
                let b = self.eval_bool(b, env);
                a == b
            }
            Eq1(a, b) => {
                let a = self.eval1(a, env);
                let b = self.eval1(b, env);
                a == b
            }
            Eq2(a, b) => {
                let a = self.eval2(a, env);
                let b = self.eval2(b, env);
                a == b
            }
            Eq3(a, b) => {
                let a = self.eval3(a, env);
                let b = self.eval3(b, env);
                a == b
            }
            Eq4(a, b) => {
                let a = self.eval4(a, env);
                let b = self.eval4(b, env);
                a == b
            }
            Less(a, b) => {
                let a = self.eval1(a, env);
                let b = self.eval1(b, env);
                a < b
            }
            GreaterOrEqual(a, b) => {
                let a = self.eval1(a, env);
                let b = self.eval1(b, env);
                a >= b
            }
        }
    }

    /// Evaluate scalar.
    pub fn eval1(&self, id: ptr::Point1<T>, env: &mut Environment<T>) -> T
        where T: Float,
              f64: Cast<T>
    {
        use fns::Point1::*;

        if let Some(&Some(val)) = env.cache.points1.get(&usize::from(id)) {
            return val;
        }

        let val = match self[id] {
            Data(a) => a,
            Time1(f, t) => self.eval_spline1(f, self.eval1(t, env), env),
            Time2(f, t) => self.eval_surface1(f, self.eval2(t, env), env),
            Sum(a, b) => {
                let a = self.eval1(a, env);
                let b = self.eval1(b, env);
                a + b
            }
            Diff(a, b) => {
                let a = self.eval1(a, env);
                let b = self.eval1(b, env);
                a - b
            }
            Prod(a, b) => {
                let a = self.eval1(a, env);
                let b = self.eval1(b, env);
                a * b
            }
            Div(a, b) => {
                let a = self.eval1(a, env);
                let b = self.eval1(b, env);
                a / b
            }
            Dot2(a, b) => {
                use vecmath::vec2_dot as dot;

                let a = self.eval2(a, env);
                let b = self.eval2(b, env);
                dot(a, b)
            }
            Dot3(a, b) => {
                use vecmath::vec3_dot as dot;

                let a = self.eval3(a, env);
                let b = self.eval3(b, env);
                dot(a, b)
            }
            Dot4(a, b) => {
                use vecmath::vec4_dot as dot;

                let a = self.eval4(a, env);
                let b = self.eval4(b, env);
                dot(a, b)
            }
            Cross(a, b) => {
                use vecmath::vec2_dot as dot;

                let a = self.eval2(a, env);
                let b = self.eval2(b, env);
                dot(a, b)
            }
            Abs(a) => {
                let a = self.eval1(a, env);
                let _0 = T::zero();
                if a < _0 {-a} else {a}
            }
            Len2(a) => vecmath::vec2_len(self.eval2(a, env)),
            Len3(a) => vecmath::vec3_len(self.eval3(a, env)),
            Len4(a) => vecmath::vec4_len(self.eval4(a, env)),
            Neg(a) => -self.eval1(a, env),
            Sign(a) => self.eval1(a, env).signum(),
            Sin(a) => self.eval1(a, env).sin(),
            Cos(a) => self.eval1(a, env).cos(),
            Tan(a) => self.eval1(a, env).tan(),
            Asin(a) => self.eval1(a, env).asin(),
            Acos(a) => self.eval1(a, env).acos(),
            Atan(a) => self.eval1(a, env).atan(),
            Atan2(a, b) => self.eval1(a, env).atan2(self.eval1(b, env)),
            Sinh(a) => self.eval1(a, env).sinh(),
            Cosh(a) => self.eval1(a, env).cosh(),
            Tanh(a) => self.eval1(a, env).tanh(),
            Asinh(a) => self.eval1(a, env).asinh(),
            Acosh(a) => self.eval1(a, env).acosh(),
            Atanh(a) => self.eval1(a, env).atanh(),
            Sqrt(a) => self.eval1(a, env).sqrt(),
            Max(a, b) => self.eval1(a, env).max(self.eval1(b, env)),
            Min(a, b) => self.eval1(a, env).min(self.eval1(b, env)),
            DegToRad(a) => self.eval1(a, env).deg_to_rad(),
            RadToDeg(a) => self.eval1(a, env).rad_to_deg(),
            Time => env.time.cast(),
            DeltaTime => env.dt.cast(),
        };

        if let Some(entry) = env.cache.points1.get_mut(&usize::from(id)) {
            *entry = Some(val);
        }
        val
    }

    /// Evaluate 2D point.
    pub fn eval2(&self, id: ptr::Point2<T>, env: &mut Environment<T>) -> [T; 2]
        where T: Float,
              f64: Cast<T>
    {
        use fns::Point2::*;

        if let Some(&Some(val)) = env.cache.points2.get(&usize::from(id)) {
            return val;
        }

        let val = match self[id] {
            Data(a) => a,
            Time1(f, t) => self.eval_spline2(f, self.eval1(t, env), env),
            Time2(f, t) => self.eval_surface2(f, self.eval2(t, env), env),
            Sum(a, b) => {
                use vecmath::vec2_add as add;

                let a = self.eval2(a, env);
                let b = self.eval2(b, env);
                add(a, b)
            }
            Prod(a, b) => {
                let a = self.eval2(a, env);
                let b = self.eval2(b, env);
                [a[0] * b[0], a[1] * b[1]]
            }
            Diff(a, b) => {
                use vecmath::vec2_sub as sub;

                let a = self.eval2(a, env);
                let b = self.eval2(b, env);
                sub(a, b)
            }
            Max(a, b) => {
                let a = self.eval2(a, env);
                let b = self.eval2(b, env);
                [a[0].max(b[0]), a[1].max(b[1])]
            }
            Min(a, b) => {
                let a = self.eval2(a, env);
                let b = self.eval2(b, env);
                [a[0].min(b[0]), a[1].min(b[1])]
            }
        };

        if let Some(entry) = env.cache.points2.get_mut(&usize::from(id)) {
            *entry = Some(val);
        }
        val
    }

    /// Evaluate 3D point.
    pub fn eval3(&self, id: ptr::Point3<T>, env: &mut Environment<T>) -> [T; 3]
        where T: Float,
              f64: Cast<T>
    {
        use fns::Point3::*;

        if let Some(&Some(val)) = env.cache.points3.get(&usize::from(id)) {
            return val;
        }

        let val = match self[id] {
            Data(a) => a,
            Time1(f, t) => self.eval_spline3(f, self.eval1(t, env), env),
            Time2(f, t) => self.eval_surface3(f, self.eval2(t, env), env),
            Sum(a, b) => {
                use vecmath::vec3_add as add;

                let a = self.eval3(a, env);
                let b = self.eval3(b, env);
                add(a, b)
            }
            Prod(a, b) => {
                let a = self.eval3(a, env);
                let b = self.eval3(b, env);
                [a[0] * b[0], a[1] * b[1], a[2] * b[2]]
            }
            Diff(a, b) => {
                use vecmath::vec3_sub as sub;

                let a = self.eval3(a, env);
                let b = self.eval3(b, env);
                sub(a, b)
            }
            Cross(a, b) => {
                use vecmath::vec3_cross as cross;

                let a = self.eval3(a, env);
                let b = self.eval3(b, env);
                cross(a, b)
            }
            Max(a, b) => {
                let a = self.eval3(a, env);
                let b = self.eval3(b, env);
                [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2])]
            }
            Min(a, b) => {
                let a = self.eval3(a, env);
                let b = self.eval3(b, env);
                [a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2])]
            }
        };

        if let Some(entry) = env.cache.points3.get_mut(&usize::from(id)) {
            *entry = Some(val);
        }
        val
    }

    /// Evaluate 4D point.
    pub fn eval4(&self, id: ptr::Point4<T>, env: &mut Environment<T>) -> [T; 4]
        where T: Float,
              f64: Cast<T>
    {
        use fns::Point4::*;

        if let Some(&Some(val)) = env.cache.points4.get(&usize::from(id)) {
            return val;
        }

        let val = match self[id] {
            Data(a) => a,
            Time1(f, t) => self.eval_spline4(f, self.eval1(t, env), env),
            Time2(f, t) => self.eval_surface4(f, self.eval2(t, env), env),
            Sum(a, b) => {
                use vecmath::vec4_add as add;

                let a = self.eval4(a, env);
                let b = self.eval4(b, env);
                add(a, b)
            }
            Prod(a, b) => {
                let a = self.eval4(a, env);
                let b = self.eval4(b, env);
                [a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]]
            }
            Diff(a, b) => {
                use vecmath::vec4_sub as sub;

                let a = self.eval4(a, env);
                let b = self.eval4(b, env);
                sub(a, b)
            }
            Max(a, b) => {
                let a = self.eval4(a, env);
                let b = self.eval4(b, env);
                [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2]), a[3].max(b[3])]
            }
            Min(a, b) => {
                let a = self.eval4(a, env);
                let b = self.eval4(b, env);
                [a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2]), a[3].min(b[3])]
            }
        };

        if let Some(entry) = env.cache.points4.get_mut(&usize::from(id)) {
            *entry = Some(val);
        }
        val
    }

    /// Evaluate 1D spline with an argument.
    pub fn eval_spline1(&self, id: SplineRef1<T>, arg: T, env: &mut Environment<T>) -> T
        where T: Float,
              f64: Cast<T>
    {
        use fns::Spline::*;

        match self[id] {
            Line(a, b) => {
                let a = self.eval1(a, env);
                let b = self.eval1(b, env);
                let _1 = T::one();
                a * (_1 - arg) + b * arg
            }
            QuadraticBezier(a, b, c) => {
                let a = self.eval1(a, env);
                let b = self.eval1(b, env);
                let c = self.eval1(c, env);
                let _1 = T::one();
                let ab = a * (_1 - arg) + b * arg;
                let bc = b * (_1 - arg) + c * arg;
                ab * (_1 - arg) + bc * arg
            }
            CubicBezier(a, b, c, d) => {
                let a = self.eval1(a, env);
                let b = self.eval1(b, env);
                let c = self.eval1(c, env);
                let d = self.eval1(d, env);
                let _1 = T::one();
                let ab = a * (_1 - arg) + b * arg;
                let cd = c * (_1 - arg) + d * arg;
                ab * (_1 - arg) + cd * arg
            }
            Segment(f, a, b) => {
                let a = self.eval1(a, env);
                let b = self.eval1(b, env);
                let _1 = T::one();
                let t = a * (_1 - arg) + b * arg;
                self.eval_spline1(f, t, env)
            }
            OnSurface(f, a, b) => {
                use vecmath::vec2_add as add;
                use vecmath::vec2_scale as scale;

                let a = self.eval2(a, env);
                let b = self.eval2(b, env);
                let _1 = T::one();
                let t = add(scale(a, _1 - arg), scale(b, arg));
                self.eval_surface1(f, t, env)
            }
            Contour(f) => {
                let _025: T = 0.25.cast();
                let _4: T = 4.0.cast();
                let _0: T = 0.0.cast();
                let _05: T = 0.5.cast();
                let _1: T = 1.0.cast();
                let _075 = 0.75.cast();
                if arg < _025 {self.eval_surface1(f, [_4 * arg, _0], env)}
                else if arg < _05 {self.eval_surface1(f, [_1, _4 * (arg - _025)], env)}
                else if arg < _075 {self.eval_surface1(f, [_1 - _4 * (arg - _05), _1], env)}
                else {self.eval_surface1(f, [_0, _1 - _4 * (arg - _075)], env)}
            }
        }
    }

    /// Evaluate 2D spline with an argument.
    pub fn eval_spline2(&self, id: SplineRef2<T>, arg: T, env: &mut Environment<T>) -> [T; 2]
        where T: Float,
              f64: Cast<T>
    {
        use fns::Spline::*;
        use vecmath::vec2_add as add;
        use vecmath::vec2_scale as scale;

        match self[id] {
            Line(a, b) => {
                let a = self.eval2(a, env);
                let b = self.eval2(b, env);
                let _1 = T::one();
                add(scale(a, _1 - arg), scale(b, arg))
            }
            QuadraticBezier(a, b, c) => {
                let a = self.eval2(a, env);
                let b = self.eval2(b, env);
                let c = self.eval2(c, env);
                let _1 = T::one();
                let ab = add(scale(a, _1 - arg), scale(b, arg));
                let bc = add(scale(b, _1 - arg), scale(c, arg));
                add(scale(ab, _1 - arg), scale(bc, arg))
            }
            CubicBezier(a, b, c, d) => {
                let a = self.eval2(a, env);
                let b = self.eval2(b, env);
                let c = self.eval2(c, env);
                let d = self.eval2(d, env);
                let _1 = T::one();
                let ab = add(scale(a, _1 - arg), scale(b, arg));
                let cd = add(scale(c, _1 - arg), scale(d, arg));
                add(scale(ab, _1 - arg), scale(cd, arg))
            }
            Segment(f, a, b) => {
                let a = self.eval1(a, env);
                let b = self.eval1(b, env);
                let _1 = T::one();
                let t = a * (_1 - arg) + b * arg;
                self.eval_spline2(f, t, env)
            }
            OnSurface(f, a, b) => {
                use vecmath::vec2_add as add;
                use vecmath::vec2_scale as scale;

                let a = self.eval2(a, env);
                let b = self.eval2(b, env);
                let _1 = T::one();
                let t = add(scale(a, _1 - arg), scale(b, arg));
                self.eval_surface2(f, t, env)
            }
            Contour(f) => {
                let _025: T = 0.25.cast();
                let _4: T = 4.0.cast();
                let _0: T = 0.0.cast();
                let _05: T = 0.5.cast();
                let _1: T = 1.0.cast();
                let _075 = 0.75.cast();
                if arg < _025 {self.eval_surface2(f, [_4 * arg, _0], env)}
                else if arg < _05 {self.eval_surface2(f, [_1, _4 * (arg - _025)], env)}
                else if arg < _075 {self.eval_surface2(f, [_1 - _4 * (arg - _05), _1], env)}
                else {self.eval_surface2(f, [_0, _1 - _4 * (arg - _075)], env)}
            }
        }
    }

    /// Evaluate 3D spline with an argument.
    pub fn eval_spline3(&self, id: SplineRef3<T>, arg: T, env: &mut Environment<T>) -> [T; 3]
        where T: Float,
              f64: Cast<T>
    {
        use fns::Spline::*;
        use vecmath::vec3_add as add;
        use vecmath::vec3_scale as scale;

        match self[id] {
            Line(a, b) => {
                let a = self.eval3(a, env);
                let b = self.eval3(b, env);
                let _1 = T::one();
                add(scale(a, _1 - arg), scale(b, arg))
            }
            QuadraticBezier(a, b, c) => {
                let a = self.eval3(a, env);
                let b = self.eval3(b, env);
                let c = self.eval3(c, env);
                let _1 = T::one();
                let ab = add(scale(a, _1 - arg), scale(b, arg));
                let bc = add(scale(b, _1 - arg), scale(c, arg));
                add(scale(ab, _1 - arg), scale(bc, arg))
            }
            CubicBezier(a, b, c, d) => {
                let a = self.eval3(a, env);
                let b = self.eval3(b, env);
                let c = self.eval3(c, env);
                let d = self.eval3(d, env);
                let _1 = T::one();
                let ab = add(scale(a, _1 - arg), scale(b, arg));
                let cd = add(scale(c, _1 - arg), scale(d, arg));
                add(scale(ab, _1 - arg), scale(cd, arg))
            }
            Segment(f, a, b) => {
                let a = self.eval1(a, env);
                let b = self.eval1(b, env);
                let _1 = T::one();
                let t = a * (_1 - arg) + b * arg;
                self.eval_spline3(f, t, env)
            }
            OnSurface(f, a, b) => {
                use vecmath::vec2_add as add;
                use vecmath::vec2_scale as scale;

                let a = self.eval2(a, env);
                let b = self.eval2(b, env);
                let _1 = T::one();
                let t = add(scale(a, _1 - arg), scale(b, arg));
                self.eval_surface3(f, t, env)
            }
            Contour(f) => {
                let _025: T = 0.25.cast();
                let _4: T = 4.0.cast();
                let _0: T = 0.0.cast();
                let _05: T = 0.5.cast();
                let _1: T = 1.0.cast();
                let _075 = 0.75.cast();
                if arg < _025 {self.eval_surface3(f, [_4 * arg, _0], env)}
                else if arg < _05 {self.eval_surface3(f, [_1, _4 * (arg - _025)], env)}
                else if arg < _075 {self.eval_surface3(f, [_1 - _4 * (arg - _05), _1], env)}
                else {self.eval_surface3(f, [_0, _1 - _4 * (arg - _075)], env)}
            }
        }
    }

    /// Evaluate 4D spline with an argument.
    pub fn eval_spline4(&self, id: SplineRef4<T>, arg: T, env: &mut Environment<T>) -> [T; 4]
        where T: Float,
              f64: Cast<T>
    {
        use fns::Spline::*;
        use vecmath::vec4_add as add;
        use vecmath::vec4_scale as scale;

        match self[id] {
            Line(a, b) => {
                let a = self.eval4(a, env);
                let b = self.eval4(b, env);
                let _1 = T::one();
                add(scale(a, _1 - arg), scale(b, arg))
            }
            QuadraticBezier(a, b, c) => {
                let a = self.eval4(a, env);
                let b = self.eval4(b, env);
                let c = self.eval4(c, env);
                let _1 = T::one();
                let ab = add(scale(a, _1 - arg), scale(b, arg));
                let bc = add(scale(b, _1 - arg), scale(c, arg));
                add(scale(ab, _1 - arg), scale(bc, arg))
            }
            CubicBezier(a, b, c, d) => {
                let a = self.eval4(a, env);
                let b = self.eval4(b, env);
                let c = self.eval4(c, env);
                let d = self.eval4(d, env);
                let _1 = T::one();
                let ab = add(scale(a, _1 - arg), scale(b, arg));
                let cd = add(scale(c, _1 - arg), scale(d, arg));
                add(scale(ab, _1 - arg), scale(cd, arg))
            }
            Segment(f, a, b) => {
                let a = self.eval1(a, env);
                let b = self.eval1(b, env);
                let _1 = T::one();
                let t = a * (_1 - arg) + b * arg;
                self.eval_spline4(f, t, env)
            }
            OnSurface(f, a, b) => {
                use vecmath::vec2_add as add;
                use vecmath::vec2_scale as scale;

                let a = self.eval2(a, env);
                let b = self.eval2(b, env);
                let _1 = T::one();
                let t = add(scale(a, _1 - arg), scale(b, arg));
                self.eval_surface4(f, t, env)
            }
            Contour(f) => {
                let _025: T = 0.25.cast();
                let _4: T = 4.0.cast();
                let _0: T = 0.0.cast();
                let _05: T = 0.5.cast();
                let _1: T = 1.0.cast();
                let _075 = 0.75.cast();
                if arg < _025 {self.eval_surface4(f, [_4 * arg, _0], env)}
                else if arg < _05 {self.eval_surface4(f, [_1, _4 * (arg - _025)], env)}
                else if arg < _075 {self.eval_surface4(f, [_1 - _4 * (arg - _05), _1], env)}
                else {self.eval_surface4(f, [_0, _1 - _4 * (arg - _075)], env)}
            }
        }
    }

    /// Evaluate 1D surface with an argument.
    pub fn eval_surface1(
        &self,
        id: SurfaceRef1<T>,
        arg: [T; 2],
        env: &mut Environment<T>
    ) -> T
        where T: Float,
              f64: Cast<T>
    {
        use fns::Surface::*;

        match self[id] {
            Rect(ref rect) => {
                let a = self.eval1(rect[0], env);
                let b = self.eval1(rect[1], env);
                let c = self.eval1(rect[2], env);
                let d = self.eval1(rect[3], env);
                let _1 = T::one();
                let ab = a * (_1 - arg[0]) + b * arg[0];
                let cd = c * (_1 - arg[0]) + d * arg[0];
                ab * (_1 - arg[1]) + cd * arg[1]
            }
            Lerp(ab, cd) => {
                let ab = self.eval_spline1(ab, arg[0], env);
                let cd = self.eval_spline1(cd, arg[1], env);
                let _1 = T::one();
                ab * (_1 - arg[1]) + cd * arg[1]
            }
            CurvedQuad {smooth, ab, cd, ac, bd} => {
                let smooth = self.eval1(smooth, env);
                let _1 = T::one();
                let _0 = T::zero();
                let _05: T = 0.5.cast();
                let _4: T = 4.0.cast();

                let abx = self.eval_spline1(ab, arg[1], env);
                let cdx = self.eval_spline1(cd, arg[1], env);
                let acx = self.eval_spline1(ac, arg[0], env);
                let bdx = self.eval_spline1(bd, arg[0], env);

                let w0 = _4 * (arg[0] - _05) * (arg[0] - _05) + smooth;
                let w1 = _4 * (arg[1] - _05) * (arg[1] - _05) + smooth;
                // Normalize weights.
                let (w0, w1) = (w0 / (w0 + w1), w1 / (w0 + w1));

                let a = abx + (cdx - abx) * arg[0];
                let b = acx + (bdx - acx) * arg[1];
                if w0 == _1 {a}
                else if w1 == _1 {b}
                else if (w0 + w1) == _0 {
                    (a + b) * _05
                }
                else {
                    a * w0 + b * w1
                }
            }
            Circle(center, radius) => {
                let center = self.eval1(center, env);
                let radius = self.eval1(radius, env);
                let two_pi = 6.283185307179586.cast();
                let angle = arg[0] * two_pi;
                center + radius * arg[1] * angle.sin()
            }
        }
    }

    /// Evaluate 2D surface with an argument.
    pub fn eval_surface2(
        &self,
        id: SurfaceRef2<T>,
        arg: [T; 2], env: &mut Environment<T>
    ) -> [T; 2]
        where T: Float,
              f64: Cast<T>
    {
        use fns::Surface::*;
        use vecmath::vec2_add as add;
        use vecmath::vec2_sub as sub;
        use vecmath::vec2_scale as scale;

        match self[id] {
            Rect(ref rect) => {
                let a = self.eval2(rect[0], env);
                let b = self.eval2(rect[1], env);
                let c = self.eval2(rect[2], env);
                let d = self.eval2(rect[3], env);
                let _1 = T::one();
                let ab = add(scale(a, _1 - arg[0]), scale(b, arg[0]));
                let cd = add(scale(c, _1 - arg[0]), scale(d, arg[0]));
                add(scale(ab, _1 - arg[1]), scale(cd, arg[1]))
            }
            Lerp(ab, cd) => {
                let ab = self.eval_spline2(ab, arg[0], env);
                let cd = self.eval_spline2(cd, arg[1], env);
                let _1 = T::one();
                add(scale(ab, _1 - arg[1]), scale(cd, arg[1]))
            }
            CurvedQuad {smooth, ab, cd, ac, bd} => {
                let smooth = self.eval1(smooth, env);
                let _1 = T::one();
                let _0 = T::zero();
                let _05: T = 0.5.cast();
                let _4: T = 4.0.cast();

                let abx = self.eval_spline2(ab, arg[1], env);
                let cdx = self.eval_spline2(cd, arg[1], env);
                let acx = self.eval_spline2(ac, arg[0], env);
                let bdx = self.eval_spline2(bd, arg[0], env);

                let w0 = _4 * (arg[0] - _05) * (arg[0] - _05) + smooth;
                let w1 = _4 * (arg[1] - _05) * (arg[1] - _05) + smooth;
                // Normalize weights.
                let (w0, w1) = (w0 / (w0 + w1), w1 / (w0 + w1));

                let a = add(abx, scale(sub(cdx, abx), arg[0]));
                let b = add(acx, scale(sub(bdx, acx), arg[1]));
                if w0 == _1 {a}
                else if w1 == _1 {b}
                else if (w0 + w1) == _0 {
                    scale(add(a, b), _05)
                }
                else {
                    add(scale(a, w0), scale(b, w1))
                }
            }
            Circle(center, radius) => {
                let center = self.eval2(center, env);
                let radius = self.eval1(radius, env);
                let two_pi = 6.283185307179586.cast();
                let angle = arg[0] * two_pi;
                [
                    center[0] + radius * arg[1] * angle.cos(),
                    center[1] + radius * arg[1] * angle.sin()
                ]
            }
        }
    }

    /// Evaluate 3D surface with an argument.
    pub fn eval_surface3(
        &self,
        id: SurfaceRef3<T>,
        arg: [T; 2],
        env: &mut Environment<T>
    ) -> [T; 3]
        where T: Float,
              f64: Cast<T>
    {
        use fns::Surface::*;
        use vecmath::vec3_add as add;
        use vecmath::vec3_sub as sub;
        use vecmath::vec3_scale as scale;

        match self[id] {
            Rect(ref rect) => {
                let a = self.eval3(rect[0], env);
                let b = self.eval3(rect[1], env);
                let c = self.eval3(rect[2], env);
                let d = self.eval3(rect[3], env);
                let _1 = T::one();
                let ab = add(scale(a, _1 - arg[0]), scale(b, arg[0]));
                let cd = add(scale(c, _1 - arg[0]), scale(d, arg[0]));
                add(scale(ab, _1 - arg[1]), scale(cd, arg[1]))
            }
            Lerp(ab, cd) => {
                let ab = self.eval_spline3(ab, arg[0], env);
                let cd = self.eval_spline3(cd, arg[1], env);
                let _1 = T::one();
                add(scale(ab, _1 - arg[1]), scale(cd, arg[1]))
            }
            CurvedQuad {smooth, ab, cd, ac, bd} => {
                let smooth = self.eval1(smooth, env);
                let _1 = T::one();
                let _0 = T::zero();
                let _05: T = 0.5.cast();
                let _4: T = 4.0.cast();

                let abx = self.eval_spline3(ab, arg[1], env);
                let cdx = self.eval_spline3(cd, arg[1], env);
                let acx = self.eval_spline3(ac, arg[0], env);
                let bdx = self.eval_spline3(bd, arg[0], env);

                let w0 = _4 * (arg[0] - _05) * (arg[0] - _05) + smooth;
                let w1 = _4 * (arg[1] - _05) * (arg[1] - _05) + smooth;
                // Normalize weights.
                let (w0, w1) = (w0 / (w0 + w1), w1 / (w0 + w1));

                let a = add(abx, scale(sub(cdx, abx), arg[0]));
                let b = add(acx, scale(sub(bdx, acx), arg[1]));
                if w0 == _1 {a}
                else if w1 == _1 {b}
                else if (w0 + w1) == _0 {
                    scale(add(a, b), _05)
                }
                else {
                    add(scale(a, w0), scale(b, w1))
                }
            }
            Circle(center, radius) => {
                let center = self.eval3(center, env);
                let radius = self.eval1(radius, env);
                let two_pi = 6.283185307179586.cast();
                let angle = arg[0] * two_pi;
                [
                    center[0] + radius * arg[1] * angle.cos(),
                    center[1] + radius * arg[1] * angle.sin(),
                    center[2]
                ]
            }
        }
    }

    /// Evaluate 4D surface with an argument.
    pub fn eval_surface4(
        &self,
        id: SurfaceRef4<T>,
        arg: [T; 2],
        env: &mut Environment<T>
    ) -> [T; 4]
        where T: Float,
              f64: Cast<T>
    {
        use fns::Surface::*;
        use vecmath::vec4_add as add;
        use vecmath::vec4_sub as sub;
        use vecmath::vec4_scale as scale;

        match self[id] {
            Rect(ref rect) => {
                let a = self.eval4(rect[0], env);
                let b = self.eval4(rect[1], env);
                let c = self.eval4(rect[2], env);
                let d = self.eval4(rect[3], env);
                let _1 = T::one();
                let ab = add(scale(a, _1 - arg[0]), scale(b, arg[0]));
                let cd = add(scale(c, _1 - arg[0]), scale(d, arg[0]));
                add(scale(ab, _1 - arg[1]), scale(cd, arg[1]))
            }
            Lerp(ab, cd) => {
                let ab = self.eval_spline4(ab, arg[0], env);
                let cd = self.eval_spline4(cd, arg[1], env);
                let _1 = T::one();
                add(scale(ab, _1 - arg[1]), scale(cd, arg[1]))
            }
            CurvedQuad {smooth, ab, cd, ac, bd} => {
                let smooth = self.eval1(smooth, env);
                let _1 = T::one();
                let _0 = T::zero();
                let _05: T = 0.5.cast();
                let _4: T = 4.0.cast();

                let abx = self.eval_spline4(ab, arg[1], env);
                let cdx = self.eval_spline4(cd, arg[1], env);
                let acx = self.eval_spline4(ac, arg[0], env);
                let bdx = self.eval_spline4(bd, arg[0], env);

                let w0 = _4 * (arg[0] - _05) * (arg[0] - _05) + smooth;
                let w1 = _4 * (arg[1] - _05) * (arg[1] - _05) + smooth;
                // Normalize weights.
                let (w0, w1) = (w0 / (w0 + w1), w1 / (w0 + w1));

                let a = add(abx, scale(sub(cdx, abx), arg[0]));
                let b = add(acx, scale(sub(bdx, acx), arg[1]));
                if w0 == _1 {a}
                else if w1 == _1 {b}
                else if (w0 + w1) == _0 {
                    scale(add(a, b), _05)
                }
                else {
                    add(scale(a, w0), scale(b, w1))
                }
            }
            Circle(center, radius) => {
                let center = self.eval4(center, env);
                let radius = self.eval1(radius, env);
                let two_pi = 6.283185307179586.cast();
                let angle = arg[0] * two_pi;
                [
                    center[0] + radius * arg[1] * angle.cos(),
                    center[1] + radius * arg[1] * angle.sin(),
                    center[2],
                    center[3]
                ]
            }
        }
    }

    /// Evaluates color.
    pub fn eval_color(
        &self,
        id: ptr::Color,
        space: color::ColorSpace,
        env: &mut Environment<T>
    ) -> Color
        where T: Float + Cast<f32>, f64: Cast<T>
    {
        use color::gamma_linear_to_srgb;
        use fns::Color::*;
        use color::ColorSpace::*;

        match self[id] {
            Data(data) => match space {
                Linear => data,
                SRGB => gamma_linear_to_srgb(data)
            },
            Time1(f, t) => {
                let t = self.eval1(t, env);
                self.eval_color_spline(f, t, space, env)
            }
        }
    }

    /// Evaluates color spline.
    pub fn eval_color_spline(
        &self,
        id: ptr::ColorSpline,
        arg: T,
        space: color::ColorSpace,
        env: &mut Environment<T>
    ) -> Color
        where T: Float + Cast<f32>, f64: Cast<T>
    {
        use color::gamma_linear_to_srgb;
        use color::ColorSpace::*;
        use fns::ColorSpline::*;
        use vecmath::vec4_add as add;
        use vecmath::vec4_scale as scale;

        let t: f32 = arg.cast();
        match self[id] {
            Lerp(a, b) => {
                let a = self.eval_color(a, Linear, env);
                let b = self.eval_color(b, Linear, env);
                let ab = add(scale(a, 1.0 - t), scale(b, t));
                match space {
                    Linear => ab,
                    SRGB => gamma_linear_to_srgb(ab)
                }
            }
        }
    }
}
