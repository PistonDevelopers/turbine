use crate::*;

/// Stores intermediate data for physics simulation.
pub struct Physics<T> {
    /// Scalars connected to bones.
    pub pos1: Vec<(T, T)>,
    /// 2D points connected to bones.
    pub pos2: Vec<([T; 2], [T; 2])>,
    /// 3D points connected to bones.
    pub pos3: Vec<([T; 3], [T; 3])>,
    /// 4D points connected to bones.
    pub pos4: Vec<([T; 4], [T; 4])>,
    /// Scalar bones that keep constant distance.
    pub eq_bones1: Vec<(usize, usize, T, T)>,
    /// 2D bones that keep constant distance.
    pub eq_bones2: Vec<(usize, usize, T, T)>,
    /// 3D bones that keep constant distance.
    pub eq_bones3: Vec<(usize, usize, T, T)>,
    /// 4D bones that keep constant distance.
    pub eq_bones4: Vec<(usize, usize, T, T)>,
    /// Bones that keep a maximum distance.
    pub less_bones1: Vec<(usize, usize, T, T)>,
    /// 2D bones that keep maximum distance.
    pub less_bones2: Vec<(usize, usize, T, T)>,
    /// 3D bones that keep maximum distance.
    pub less_bones3: Vec<(usize, usize, T, T)>,
    /// 4D bones that keep maximum distance.
    pub less_bones4: Vec<(usize, usize, T, T)>,
    /// Scalar bones that keep minimum distance.
    pub more_bones1: Vec<(usize, usize, T, T)>,
    /// 2D bones that keep minimum distance.
    pub more_bones2: Vec<(usize, usize, T, T)>,
    /// 3D bones that keep minimum distance.
    pub more_bones3: Vec<(usize, usize, T, T)>,
    /// 4D bones that keep minimum distance.
    pub more_bones4: Vec<(usize, usize, T, T)>,
    /// Position map back to document for scalars.
    /// Sorted to enable binary search.
    pub map1: Vec<ptr::Point1<T>>,
    /// Position map back to document for 2D points.
    /// Sorted to enable binary search.
    pub map2: Vec<ptr::Point2<T>>,
    /// Position map back to document for 3D points.
    /// Sorted to enable binary search.
    pub map3: Vec<ptr::Point3<T>>,
    /// Position map back to document for 4D points.
    /// Sorted to enable binary search.
    pub map4: Vec<ptr::Point4<T>>,
}

impl<T> Physics<T> {
    /// Creates new physics from document.
    pub fn new(doc: &Reactor<T>, selection: &Selection<T>, env: &mut Environment<T>) -> Physics<T>
        where T: Float, f64: Cast<T>
    {
        use fns::Bone::*;

        let _05 = 0.5.cast();
        let mut pos1 = vec![];
        let mut pos2 = vec![];
        let mut pos3 = vec![];
        let mut pos4 = vec![];

        let mut eq_bones1 = vec![];
        let mut eq_bones2 = vec![];
        let mut eq_bones3 = vec![];
        let mut eq_bones4 = vec![];
        let mut less_bones1 = vec![];
        let mut less_bones2 = vec![];
        let mut less_bones3 = vec![];
        let mut less_bones4 = vec![];
        let mut more_bones1 = vec![];
        let mut more_bones2 = vec![];
        let mut more_bones3 = vec![];
        let mut more_bones4 = vec![];

        let mut map1 = vec![];
        let mut map2 = vec![];
        let mut map3 = vec![];
        let mut map4 = vec![];

        // By adding all points to the map back, remove duplicates and
        // sorting them, a unique id is obtained,
        // so one can avoid the need for hash maps.
        for b in &doc.bones1 {
            match *b {
                Eq(a, b, _) | Less(a, b, _) | More(a, b, _) => {
                    map1.push(a);
                    map1.push(b);
                }
            }
        }
        map1.sort_by_key(|id| id.0);
        map1.dedup();
        for i in 0..map1.len() {
            let pos = doc.eval1(map1[i], env);
            pos1.push((pos, pos));
        }
        for b in &doc.bones1 {
            match *b {
                Eq(a, b, dist) => {
                    let a_ind = map1.binary_search_by_key(&a.0, |id| id.0).unwrap();
                    let b_ind = map1.binary_search_by_key(&b.0, |id| id.0).unwrap();
                    let dist = doc.eval1(dist, env);
                    eq_bones1.push((a_ind, b_ind, dist, _05));
                }
                Less(a, b, dist) => {
                    let a_ind = map1.binary_search_by_key(&a.0, |id| id.0).unwrap();
                    let b_ind = map1.binary_search_by_key(&b.0, |id| id.0).unwrap();
                    let dist = doc.eval1(dist, env);
                    less_bones1.push((a_ind, b_ind, dist, _05));
                }
                More(a, b, dist) => {
                    let a_ind = map1.binary_search_by_key(&a.0, |id| id.0).unwrap();
                    let b_ind = map1.binary_search_by_key(&b.0, |id| id.0).unwrap();
                    let dist = doc.eval1(dist, env);
                    more_bones1.push((a_ind, b_ind, dist, _05));
                }
            }
        }

        for b in &doc.bones2 {
            match *b {
                Eq(a, b, _) | Less(a, b, _) | More(a, b, _) => {
                    map2.push(a);
                    map2.push(b);
                }
            }
        }
        map2.sort_by_key(|id| id.0);
        map2.dedup();
        for i in 0..map2.len() {
            let pos = doc.eval2(map2[i], env);
            pos2.push((pos, pos));
        }
        for b in &doc.bones2 {
            match *b {
                Eq(a, b, dist) => {
                    let a_ind = map2.binary_search_by_key(&a.0, |id| id.0).unwrap();
                    let b_ind = map2.binary_search_by_key(&b.0, |id| id.0).unwrap();
                    let dist = doc.eval1(dist, env);
                    eq_bones2.push((a_ind, b_ind, dist, _05));
                }
                Less(a, b, dist) => {
                    let a_ind = map2.binary_search_by_key(&a.0, |id| id.0).unwrap();
                    let b_ind = map2.binary_search_by_key(&b.0, |id| id.0).unwrap();
                    let dist = doc.eval1(dist, env);
                    less_bones2.push((a_ind, b_ind, dist, _05));
                }
                More(a, b, dist) => {
                    let a_ind = map2.binary_search_by_key(&a.0, |id| id.0).unwrap();
                    let b_ind = map2.binary_search_by_key(&b.0, |id| id.0).unwrap();
                    let dist = doc.eval1(dist, env);
                    more_bones2.push((a_ind, b_ind, dist, _05));
                }
            }
        }

        for b in &doc.bones3 {
            match *b {
                Eq(a, b, _) | Less(a, b, _) | More(a, b, _) => {
                    map3.push(a);
                    map3.push(b);
                }
            }
        }
        map3.sort_by_key(|id| id.0);
        map3.dedup();
        for i in 0..map3.len() {
            let pos = doc.eval3(map3[i], env);
            pos3.push((pos, pos));
        }
        for b in &doc.bones3 {
            match *b {
                Eq(a, b, dist) => {
                    let a_ind = map3.binary_search_by_key(&a.0, |id| id.0).unwrap();
                    let b_ind = map3.binary_search_by_key(&b.0, |id| id.0).unwrap();
                    let dist = doc.eval1(dist, env);
                    eq_bones3.push((a_ind, b_ind, dist, _05));
                }
                Less(a, b, dist) => {
                    let a_ind = map3.binary_search_by_key(&a.0, |id| id.0).unwrap();
                    let b_ind = map3.binary_search_by_key(&b.0, |id| id.0).unwrap();
                    let dist = doc.eval1(dist, env);
                    less_bones3.push((a_ind, b_ind, dist, _05));
                }
                More(a, b, dist) => {
                    let a_ind = map3.binary_search_by_key(&a.0, |id| id.0).unwrap();
                    let b_ind = map3.binary_search_by_key(&b.0, |id| id.0).unwrap();
                    let dist = doc.eval1(dist, env);
                    more_bones3.push((a_ind, b_ind, dist, _05));
                }
            }
        }

        for b in &doc.bones4 {
            match *b {
                Eq(a, b, _) | Less(a, b, _) | More(a, b, _) => {
                    map4.push(a);
                    map4.push(b);
                }
            }
        }
        map4.sort_by_key(|id| id.0);
        map4.dedup();
        for i in 0..map4.len() {
            let pos = doc.eval4(map4[i], env);
            pos4.push((pos, pos));
        }
        for b in &doc.bones4 {
            match *b {
                Eq(a, b, dist) => {
                    let a_ind = map4.binary_search_by_key(&a.0, |id| id.0).unwrap();
                    let b_ind = map4.binary_search_by_key(&b.0, |id| id.0).unwrap();
                    let dist = doc.eval1(dist, env);
                    eq_bones4.push((a_ind, b_ind, dist, _05));
                }
                Less(a, b, dist) => {
                    let a_ind = map4.binary_search_by_key(&a.0, |id| id.0).unwrap();
                    let b_ind = map4.binary_search_by_key(&b.0, |id| id.0).unwrap();
                    let dist = doc.eval1(dist, env);
                    less_bones4.push((a_ind, b_ind, dist, _05));
                }
                More(a, b, dist) => {
                    let a_ind = map4.binary_search_by_key(&a.0, |id| id.0).unwrap();
                    let b_ind = map4.binary_search_by_key(&b.0, |id| id.0).unwrap();
                    let dist = doc.eval1(dist, env);
                    more_bones4.push((a_ind, b_ind, dist, _05));
                }
            }
        }

        let mut locked1 = vec![false; pos1.len()];
        let mut locked2 = vec![false; pos2.len()];
        let mut locked3 = vec![false; pos3.len()];
        let mut locked4 = vec![false; pos4.len()];

        // Find locked state of points attached to bones.
        for &p in &selection.locked1 {
            if let Ok(ind) = map1.binary_search_by_key(&p.0, |id| id.0) {
                locked1[ind] = true;
            }
        }
        for &p in &selection.locked2 {
            if let Ok(ind) = map2.binary_search_by_key(&p.0, |id| id.0) {
                locked2[ind] = true;
            }
        }
        for &p in &selection.locked3 {
            if let Ok(ind) = map3.binary_search_by_key(&p.0, |id| id.0) {
                locked3[ind] = true;
            }
        }
        for &p in &selection.locked4 {
            if let Ok(ind) = map4.binary_search_by_key(&p.0, |id| id.0) {
                locked4[ind] = true;
            }
        }

        // A point is considered locked when it is computed by a function.
        for (i, &p) in map1.iter().enumerate() {
            if !locked1[i] {
                locked1[i] = if let fns::Point1::Data(_) = doc[p] {false} else {true};
            }
        }
        for (i, &p) in map2.iter().enumerate() {
            if !locked2[i] {
                locked2[i] = if let fns::Point2::Data(_) = doc[p] {false} else {true};
            }
        }
        for (i, &p) in map3.iter().enumerate() {
            if !locked3[i] {
                locked3[i] = if let fns::Point3::Data(_) = doc[p] {false} else {true};
            }
        }
        for (i, &p) in map4.iter().enumerate() {
            if !locked4[i] {
                locked4[i] = if let fns::Point4::Data(_) = doc[p] {false} else {true};
            }
        }

        let _0 = T::zero();
        let _1 = T::one();

        // Remove bones which both points are locked or pointing to same point.
        // Change the bone interaction weight such that it reflect the locked state.
        let remove = |bones: &mut Vec<(usize, usize, T, T)>, locked: &[bool]| {
            for i in (0..bones.len()).rev() {
                let a = bones[i].0;
                let b = bones[i].1;
                let locked_a = locked[a];
                let locked_b = locked[b];
                if locked_a && locked_a || a == b {
                    bones.swap_remove(i);
                } else if locked_a {
                    bones[i].3 = _0;
                } else if locked_b {
                    bones[i].3 = _1;
                }
            }
        };

        remove(&mut eq_bones1, &locked1);
        remove(&mut eq_bones2, &locked2);
        remove(&mut eq_bones3, &locked3);
        remove(&mut eq_bones4, &locked4);
        remove(&mut less_bones1, &locked1);
        remove(&mut less_bones2, &locked2);
        remove(&mut less_bones3, &locked3);
        remove(&mut less_bones4, &locked4);
        remove(&mut more_bones1, &locked1);
        remove(&mut more_bones2, &locked2);
        remove(&mut more_bones3, &locked3);
        remove(&mut more_bones4, &locked4);

        Physics {
            pos1,
            pos2,
            pos3,
            pos4,
            eq_bones1,
            eq_bones2,
            eq_bones3,
            eq_bones4,
            less_bones1,
            less_bones2,
            less_bones3,
            less_bones4,
            more_bones1,
            more_bones2,
            more_bones3,
            more_bones4,
            map1,
            map2,
            map3,
            map4,
        }
    }

    /// Simulates points.
    /// Returns a positive number representing the error squared from a satisfied solution.
    pub fn simulate(&mut self) -> T where T: Float, f64: Cast<T> {
        let _0 = T::zero();
        let _1 = T::one();
        let mut error = _0;
        for &(a_ind, b_ind, dist, w) in &self.eq_bones1 {
            let a = self.pos1[a_ind].0;
            let b = self.pos1[b_ind].0;

            let diff = a - b;
            let len = if diff < _0 {-diff} else {diff};
            let delta = diff.signum() * (dist - len);
            self.pos1[a_ind].1 += delta * w;
            self.pos1[b_ind].1 -= delta * (_1 - w);
            error += (dist - len) * (dist - len);
        }
        for &(a_ind, b_ind, dist, w) in &self.less_bones1 {
            let a = self.pos1[a_ind].0;
            let b = self.pos1[b_ind].0;

            let diff = a - b;
            let len = if diff < _0 {-diff} else {diff};
            if len > dist {
                let delta = len - dist;
                self.pos1[a_ind].1 += delta * w;
                self.pos1[b_ind].1 -= delta * (_1 - w);
                error += delta * delta;
            }
        }
        for &(a_ind, b_ind, dist, w) in &self.more_bones1 {
            let a = self.pos1[a_ind].0;
            let b = self.pos1[b_ind].0;

            let diff = a - b;
            let len = if diff < _0 {-diff} else {diff};
            if len < dist {
                let delta = dist - len;
                self.pos1[a_ind].1 += delta * w;
                self.pos1[b_ind].1 -= delta * (_1 - w);
                error += delta * delta;
            }
        }
        for &(a_ind, b_ind, dist, w) in &self.eq_bones2 {
            use vecmath::vec2_add as add;
            use vecmath::vec2_sub as sub;
            use vecmath::vec2_len as len;
            use vecmath::vec2_scale as scale;

            let a = self.pos2[a_ind].0;
            let b = self.pos2[b_ind].0;

            let diff = sub(a, b);
            let len = len(diff);
            if len > _0 {
                let delta = dist - len;
                let dir = scale(diff, _1 / len);
                self.pos2[a_ind].1 = add(self.pos2[a_ind].1, scale(dir, delta * w));
                self.pos2[b_ind].1 = sub(self.pos2[b_ind].1, scale(dir, delta * (_1 - w)));
                error += delta * delta;
            }
        }
        for &(a_ind, b_ind, dist, w) in &self.less_bones2 {
            use vecmath::vec2_add as add;
            use vecmath::vec2_sub as sub;
            use vecmath::vec2_len as len;
            use vecmath::vec2_scale as scale;

            let a = self.pos2[a_ind].0;
            let b = self.pos2[b_ind].0;

            let diff = sub(a, b);
            let len = len(diff);
            if len > dist {
                let delta = dist - len;
                let dir = scale(diff, _1 / len);
                self.pos2[a_ind].1 = add(self.pos2[a_ind].1, scale(dir, delta * w));
                self.pos2[b_ind].1 = sub(self.pos2[b_ind].1, scale(dir, delta * (_1 - w)));
                error += delta * delta;
            }
        }
        for &(a_ind, b_ind, dist, w) in &self.more_bones2 {
            use vecmath::vec2_add as add;
            use vecmath::vec2_sub as sub;
            use vecmath::vec2_len as len;
            use vecmath::vec2_scale as scale;

            let a = self.pos2[a_ind].0;
            let b = self.pos2[b_ind].0;

            let diff = sub(a, b);
            let len = len(diff);
            if len > _0 && len < dist {
                let delta = dist - len;
                let dir = scale(diff, _1 / len);
                self.pos2[a_ind].1 = add(self.pos2[a_ind].1, scale(dir, delta * w));
                self.pos2[b_ind].1 = sub(self.pos2[b_ind].1, scale(dir, delta * (_1 - w)));
                error += delta * delta;
            }
        }
        for &(a_ind, b_ind, dist, w) in &self.eq_bones3 {
            use vecmath::vec3_add as add;
            use vecmath::vec3_sub as sub;
            use vecmath::vec3_len as len;
            use vecmath::vec3_scale as scale;

            let a = self.pos3[a_ind].0;
            let b = self.pos3[b_ind].0;

            let diff = sub(a, b);
            let len = len(diff);
            if len > _0 {
                let delta = dist - len;
                let dir = scale(diff, _1 / len);
                self.pos3[a_ind].1 = add(self.pos3[a_ind].1, scale(dir, delta * w));
                self.pos3[b_ind].1 = sub(self.pos3[b_ind].1, scale(dir, delta * (_1 - w)));
                error += delta * delta;
            }
        }
        for &(a_ind, b_ind, dist, w) in &self.less_bones3 {
            use vecmath::vec3_add as add;
            use vecmath::vec3_sub as sub;
            use vecmath::vec3_len as len;
            use vecmath::vec3_scale as scale;

            let a = self.pos3[a_ind].0;
            let b = self.pos3[b_ind].0;

            let diff = sub(a, b);
            let len = len(diff);
            if len > dist {
                let delta = dist - len;
                let dir = scale(diff, _1 / len);
                self.pos3[a_ind].1 = add(self.pos3[a_ind].1, scale(dir, delta * w));
                self.pos3[b_ind].1 = sub(self.pos3[b_ind].1, scale(dir, delta * (_1 - w)));
                error += delta * delta;
            }
        }
        for &(a_ind, b_ind, dist, w) in &self.more_bones3 {
            use vecmath::vec3_add as add;
            use vecmath::vec3_sub as sub;
            use vecmath::vec3_len as len;
            use vecmath::vec3_scale as scale;

            let a = self.pos3[a_ind].0;
            let b = self.pos3[b_ind].0;

            let diff = sub(a, b);
            let len = len(diff);
            if len > _0 && len < dist {
                let delta = dist - len;
                let dir = scale(diff, _1 / len);
                self.pos3[a_ind].1 = add(self.pos3[a_ind].1, scale(dir, delta * w));
                self.pos3[b_ind].1 = sub(self.pos3[b_ind].1, scale(dir, delta * (_1 - w)));
                error += delta * delta;
            }
        }
        for &(a_ind, b_ind, dist, w) in &self.eq_bones4 {
            use vecmath::vec4_add as add;
            use vecmath::vec4_sub as sub;
            use vecmath::vec4_len as len;
            use vecmath::vec4_scale as scale;

            let a = self.pos4[a_ind].0;
            let b = self.pos4[b_ind].0;

            let diff = sub(a, b);
            let len = len(diff);
            if len > _0 {
                let delta = dist - len;
                let dir = scale(diff, _1 / len);
                self.pos4[a_ind].1 = add(self.pos4[a_ind].1, scale(dir, delta * w));
                self.pos4[b_ind].1 = sub(self.pos4[b_ind].1, scale(dir, delta * (_1 - w)));
                error += delta * delta;
            }
        }
        for &(a_ind, b_ind, dist, w) in &self.less_bones4 {
            use vecmath::vec4_add as add;
            use vecmath::vec4_sub as sub;
            use vecmath::vec4_len as len;
            use vecmath::vec4_scale as scale;

            let a = self.pos4[a_ind].0;
            let b = self.pos4[b_ind].0;

            let diff = sub(a, b);
            let len = len(diff);
            if len > dist {
                let delta = dist - len;
                let dir = scale(diff, _1 / len);
                self.pos4[a_ind].1 = add(self.pos4[a_ind].1, scale(dir, delta * w));
                self.pos4[b_ind].1 = sub(self.pos4[b_ind].1, scale(dir, delta * (_1 - w)));
                error += delta * delta;
            }
        }
        for &(a_ind, b_ind, dist, w) in &self.more_bones4 {
            use vecmath::vec4_add as add;
            use vecmath::vec4_sub as sub;
            use vecmath::vec4_len as len;
            use vecmath::vec4_scale as scale;

            let a = self.pos4[a_ind].0;
            let b = self.pos4[b_ind].0;

            let diff = sub(a, b);
            let len = len(diff);
            if len > _0 && len < dist {
                let delta = dist - len;
                let dir = scale(diff, _1 / len);
                self.pos4[a_ind].1 = add(self.pos4[a_ind].1, scale(dir, delta * w));
                self.pos4[b_ind].1 = sub(self.pos4[b_ind].1, scale(dir, delta * (_1 - w)));
                error += delta * delta;
            }
        }

        error
    }

    /// Commit changes to document.
    /// Writes data to document and updates previous position.
    pub fn commit(&mut self, doc: &mut Reactor<T>) where T: Copy {
        for i in 0..self.pos1.len() {
            if let fns::Point1::Data(ref mut p) = doc[self.map1[i]] {
                *p = self.pos1[i].1;
                self.pos1[i].0 = self.pos1[i].1;
            }
        }
        for i in 0..self.pos2.len() {
            if let fns::Point2::Data(ref mut p) = doc[self.map2[i]] {
                *p = self.pos2[i].1;
                self.pos2[i].0 = self.pos2[i].1;
            }
        }
        for i in 0..self.pos3.len() {
            if let fns::Point3::Data(ref mut p) = doc[self.map3[i]] {
                *p = self.pos3[i].1;
                self.pos3[i].0 = self.pos3[i].1;
            }
        }
        for i in 0..self.pos4.len() {
            if let fns::Point4::Data(ref mut p) = doc[self.map4[i]] {
                *p = self.pos4[i].1;
                self.pos4[i].0 = self.pos4[i].1;
            }
        }
    }
}
