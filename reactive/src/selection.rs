use *;

/// Stores selection state.
pub struct Selection<T> {
    /// Selected scalars.
    pub selected1: Vec<ptr::Point1<T>>,
    /// Selected 2D vectors.
    pub selected2: Vec<ptr::Point2<T>>,
    /// Selected 3D vectors.
    pub selected3: Vec<ptr::Point3<T>>,
    /// Selected 4D vectors.
    pub selected4: Vec<ptr::Point3<T>>,
    /// Locked scalars.
    pub locked1: Vec<ptr::Point1<T>>,
    /// Locked 2D vectors.
    pub locked2: Vec<ptr::Point2<T>>,
    /// Locked 3D vectors.
    pub locked3: Vec<ptr::Point3<T>>,
    /// Locked 4D vectors.
    pub locked4: Vec<ptr::Point4<T>>,
}

impl<T> Selection<T> {
    /// Creates a new empty selection.
    pub fn new() -> Selection<T> {
        Selection {
            selected1: vec![],
            selected2: vec![],
            selected3: vec![],
            selected4: vec![],
            locked1: vec![],
            locked2: vec![],
            locked3: vec![],
            locked4: vec![],
        }
    }
}
