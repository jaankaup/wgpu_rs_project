pub struct Array3D {
    array: Vec<f32>,
    dimension: (u32, u32, u32),
}

impl Array3D {
    pub fn init(x: u32, y: u32, z: u32, default_value: f32) -> Self {
        Self {
            array: vec![default_value ; (x*y*z) as usize],
            dimension: (x, y, z),
        }
    }

    /// Get the array index from 3d space indices. Panics if trying to index and value that are not
    /// in range.
    pub fn get_index(&self, x: u32, y: u32, z:u32) -> u32 {

        let result = x + y * self.dimension.0 + z * self.dimension.0 * self.dimension.1;
        assert!(x < self.dimension.0 && y < self.dimension.1 && z < self.dimension.2,
                "({}, {}, {}) not in range [(0, 0, 0), ({}, {}, {})[",
                x, y, z, self.dimension.0, self.dimension.1, self.dimension.2);
        //assert!(result < self.array.capacity() as u32,
        //        "Index out of bounds ({}, {}, {}) => {} >= {}",
        //        x, y, z, result, self.array.capacity());
        result
    }

    pub fn get_3d_index(&self, mut index: u32) -> (u32, u32, u32) {

        assert!(index < self.array.capacity() as u32, "{} < {}", index, self.array.capacity());

        let wh = self.dimension.0 * self.dimension.1;
        let mut x = index / wh;
        index -= x * wh; 
        let y = index / self.dimension.0;
        index -= y*self.dimension.0;
        let z = index;

        (x, y, z)
    }
}
