use *;

use std::ops::{Index, IndexMut};

impl<T> Index<ptr::Bool> for Reactor<T> {
    type Output = fns::Bool<T>;
    fn index(&self, id: ptr::Bool) -> &fns::Bool<T> {
        &self.bools[usize::from(id)]
    }
}

impl<T> Index<ptr::Point1<T>> for Reactor<T> {
    type Output = fns::Point1<T>;
    fn index(&self, id: ptr::Point1<T>) -> &fns::Point1<T> {
        &self.points1[usize::from(id)]
    }
}

impl<T> Index<ptr::Point2<T>> for Reactor<T> {
    type Output = fns::Point2<T>;
    fn index(&self, id: ptr::Point2<T>) -> &fns::Point2<T> {
        &self.points2[usize::from(id)]
    }
}

impl<T> Index<ptr::Point3<T>> for Reactor<T> {
    type Output = fns::Point3<T>;
    fn index(&self, id: ptr::Point3<T>) -> &fns::Point3<T> {
        &self.points3[usize::from(id)]
    }
}

impl<T> Index<ptr::Point4<T>> for Reactor<T> {
    type Output = fns::Point4<T>;
    fn index(&self, id: ptr::Point4<T>) -> &fns::Point4<T> {
        &self.points4[usize::from(id)]
    }
}

impl<T> Index<SplineRef1<T>> for Reactor<T> {
    type Output = Spline1<T>;
    fn index(&self, id: SplineRef1<T>) -> &Spline1<T> {
        &self.splines1[usize::from(id)]
    }
}

impl<T> Index<SplineRef2<T>> for Reactor<T> {
    type Output = Spline2<T>;
    fn index(&self, id: SplineRef2<T>) -> &Spline2<T> {
        &self.splines2[usize::from(id)]
    }
}

impl<T> Index<SplineRef3<T>> for Reactor<T> {
    type Output = Spline3<T>;
    fn index(&self, id: SplineRef3<T>) -> &Spline3<T> {
        &self.splines3[usize::from(id)]
    }
}

impl<T> Index<SplineRef4<T>> for Reactor<T> {
    type Output = Spline4<T>;
    fn index(&self, id: SplineRef4<T>) -> &Spline4<T> {
        &self.splines4[usize::from(id)]
    }
}

impl<T> Index<SurfaceRef1<T>> for Reactor<T> {
    type Output = Surface1<T>;
    fn index(&self, id: SurfaceRef1<T>) -> &Surface1<T> {
        &self.surfaces1[usize::from(id)]
    }
}

impl<T> Index<SurfaceRef2<T>> for Reactor<T> {
    type Output = Surface2<T>;
    fn index(&self, id: SurfaceRef2<T>) -> &Surface2<T> {
        &self.surfaces2[usize::from(id)]
    }
}

impl<T> Index<SurfaceRef3<T>> for Reactor<T> {
    type Output = Surface3<T>;
    fn index(&self, id: SurfaceRef3<T>) -> &Surface3<T> {
        &self.surfaces3[usize::from(id)]
    }
}

impl<T> Index<SurfaceRef4<T>> for Reactor<T> {
    type Output = Surface4<T>;
    fn index(&self, id: SurfaceRef4<T>) -> &Surface4<T> {
        &self.surfaces4[usize::from(id)]
    }
}

impl<T> IndexMut<ptr::Bool> for Reactor<T> {
    fn index_mut(&mut self, id: ptr::Bool) -> &mut fns::Bool<T> {
        &mut self.bools[usize::from(id)]
    }
}

impl<T> IndexMut<ptr::Point1<T>> for Reactor<T> {
    fn index_mut(&mut self, id: ptr::Point1<T>) -> &mut fns::Point1<T> {
        &mut self.points1[usize::from(id)]
    }
}

impl<T> IndexMut<ptr::Point2<T>> for Reactor<T> {
    fn index_mut(&mut self, id: ptr::Point2<T>) -> &mut fns::Point2<T> {
        &mut self.points2[usize::from(id)]
    }
}

impl<T> IndexMut<ptr::Point3<T>> for Reactor<T> {
    fn index_mut(&mut self, id: ptr::Point3<T>) -> &mut fns::Point3<T> {
        &mut self.points3[usize::from(id)]
    }
}

impl<T> IndexMut<ptr::Point4<T>> for Reactor<T> {
    fn index_mut(&mut self, id: ptr::Point4<T>) -> &mut fns::Point4<T> {
        &mut self.points4[usize::from(id)]
    }
}

impl<T> IndexMut<SplineRef1<T>> for Reactor<T> {
    fn index_mut(&mut self, id: SplineRef1<T>) -> &mut Spline1<T> {
        &mut self.splines1[usize::from(id)]
    }
}

impl<T> IndexMut<SplineRef2<T>> for Reactor<T> {
    fn index_mut(&mut self, id: SplineRef2<T>) -> &mut Spline2<T> {
        &mut self.splines2[usize::from(id)]
    }
}

impl<T> IndexMut<SplineRef3<T>> for Reactor<T> {
    fn index_mut(&mut self, id: SplineRef3<T>) -> &mut Spline3<T> {
        &mut self.splines3[usize::from(id)]
    }
}

impl<T> IndexMut<SplineRef4<T>> for Reactor<T> {
    fn index_mut(&mut self, id: SplineRef4<T>) -> &mut Spline4<T> {
        &mut self.splines4[usize::from(id)]
    }
}

impl<T> IndexMut<SurfaceRef1<T>> for Reactor<T> {
    fn index_mut(&mut self, id: SurfaceRef1<T>) -> &mut Surface1<T> {
        &mut self.surfaces1[usize::from(id)]
    }
}

impl<T> IndexMut<SurfaceRef2<T>> for Reactor<T> {
    fn index_mut(&mut self, id: SurfaceRef2<T>) -> &mut Surface2<T> {
        &mut self.surfaces2[usize::from(id)]
    }
}

impl<T> IndexMut<SurfaceRef3<T>> for Reactor<T> {
    fn index_mut(&mut self, id: SurfaceRef3<T>) -> &mut Surface3<T> {
        &mut self.surfaces3[usize::from(id)]
    }
}

impl<T> IndexMut<SurfaceRef4<T>> for Reactor<T> {
    fn index_mut(&mut self, id: SurfaceRef4<T>) -> &mut Surface4<T> {
        &mut self.surfaces4[usize::from(id)]
    }
}

impl<T> Index<ptr::Color> for Reactor<T> {
    type Output = fns::Color<T>;
    fn index(&self, id: ptr::Color) -> &fns::Color<T> {
        &self.colors[usize::from(id)]
    }
}

impl<T> IndexMut<ptr::Color> for Reactor<T> {
    fn index_mut(&mut self, id: ptr::Color) -> &mut fns::Color<T> {
        &mut self.colors[usize::from(id)]
    }
}

impl<T> Index<ptr::ColorSpline> for Reactor<T> {
    type Output = fns::ColorSpline;
    fn index(&self, id: ptr::ColorSpline) -> &fns::ColorSpline {
        &self.color_splines[usize::from(id)]
    }
}

impl<T> IndexMut<ptr::ColorSpline> for Reactor<T> {
    fn index_mut(&mut self, id: ptr::ColorSpline) -> &mut fns::ColorSpline {
        &mut self.color_splines[usize::from(id)]
    }
}

impl<T> Index<BoneRef1<T>> for Reactor<T> {
    type Output = Bone1<T>;
    fn index(&self, id: BoneRef1<T>) -> &Bone1<T> {
        &self.bones1[usize::from(id)]
    }
}

impl<T> Index<BoneRef2<T>> for Reactor<T> {
    type Output = Bone2<T>;
    fn index(&self, id: BoneRef2<T>) -> &Bone2<T> {
        &self.bones2[usize::from(id)]
    }
}

impl<T> Index<BoneRef3<T>> for Reactor<T> {
    type Output = Bone3<T>;
    fn index(&self, id: BoneRef3<T>) -> &Bone3<T> {
        &self.bones3[usize::from(id)]
    }
}

impl<T> Index<BoneRef4<T>> for Reactor<T> {
    type Output = Bone4<T>;
    fn index(&self, id: BoneRef4<T>) -> &Bone4<T> {
        &self.bones4[usize::from(id)]
    }
}

impl<T> IndexMut<BoneRef1<T>> for Reactor<T> {
    fn index_mut(&mut self, id: BoneRef1<T>) -> &mut Bone1<T> {
        &mut self.bones1[usize::from(id)]
    }
}

impl<T> IndexMut<BoneRef2<T>> for Reactor<T> {
    fn index_mut(&mut self, id: BoneRef2<T>) -> &mut Bone2<T> {
        &mut self.bones2[usize::from(id)]
    }
}

impl<T> IndexMut<BoneRef3<T>> for Reactor<T> {
    fn index_mut(&mut self, id: BoneRef3<T>) -> &mut Bone3<T> {
        &mut self.bones3[usize::from(id)]
    }
}

impl<T> IndexMut<BoneRef4<T>> for Reactor<T> {
    fn index_mut(&mut self, id: BoneRef4<T>) -> &mut Bone4<T> {
        &mut self.bones4[usize::from(id)]
    }
}
