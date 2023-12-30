pub struct ComplexObjectVertex {
    pub xyz: [f32; 3],
    pub uv: [f32; 2],
}

impl ComplexObjectVertex {
    #[inline] pub fn new(xyz: [f32; 3], uv: [f32; 2]) -> Self { Self {xyz, uv} }

    #[inline]
    pub fn x(&self, rotation_index: usize) -> f32 {
        if rotation_index > 1 {self.xyz[2]} else {self.xyz[0]}
    }

    #[inline] pub fn y(&self) -> f32 {self.xyz[1]}

    #[inline]
    pub fn z(&self, rotation_index: usize) -> f32 {
        if rotation_index > 1 {self.xyz[0]} else {self.xyz[2]}
    }

    #[inline] pub fn u(&self) -> f32 {self.uv[0]}
    #[inline] pub fn v(&self) -> f32 {self.uv[1]}
}

impl From<([f32; 3], [f32; 2])> for ComplexObjectVertex {
    fn from(value: ([f32; 3], [f32; 2])) -> Self {
        Self { xyz: value.0, uv: value.1 }
    }
}

pub struct ComplexObjectGroup(pub [ComplexObjectVertex; 4]);

impl ComplexObjectGroup {
    const VERTEX_ORDER: [[usize; 4]; 4] = [
        [0, 1, 2, 3],
        [2, 3, 0, 1],
        [3, 2, 1, 0],
        [1, 0, 3, 2]
    ];

    #[inline]
    pub fn get(&self, rotation_index: usize, index: usize) -> &ComplexObjectVertex {
        &self.0[Self::VERTEX_ORDER[rotation_index][index]]
    }

    #[inline]
    pub fn x(&self, rotation_index: usize, index: usize) -> f32 {
        self.0[Self::VERTEX_ORDER[rotation_index][index]].x(rotation_index)
    }

    #[inline]
    pub fn y(&self, rotation_index: usize, index: usize) -> f32 {
        self.0[Self::VERTEX_ORDER[rotation_index][index]].y()
    }

    #[inline]
    pub fn z(&self, rotation_index: usize, index: usize) -> f32 {
        self.0[Self::VERTEX_ORDER[rotation_index][index]].z(rotation_index)
    }


    pub fn sum_position(&self, x: f32, y: f32, z: f32, rotation_index: usize, index: usize) -> [f32; 3] {
        [self.x(rotation_index, index) + x,
         self.y(rotation_index, index) + y,
         self.z(rotation_index, index) + z]
    }

    #[inline] pub fn u(&self, index: usize) -> f32 {self.0[index].u()}
    #[inline] pub fn v(&self, index: usize) -> f32 {self.0[index].v()}
    #[inline] pub fn uv(&self, index: usize) -> [f32; 2] {[self.0[index].u(), self.0[index].v()]}
}

impl From<[ComplexObjectVertex; 4]> for ComplexObjectGroup {
    fn from(value: [ComplexObjectVertex; 4]) -> Self {
        Self(value)
    }
}

pub struct ComplexObjectSide {
    pub texture_layer: u32,
    pub vertex_groups: Vec<ComplexObjectGroup>,
}

impl ComplexObjectSide {
    #[inline]
    pub fn new(texture_layer: u32, vertex_groups: Vec<ComplexObjectGroup>) -> Self {
        Self { texture_layer, vertex_groups }
    }
}

pub enum ComplexObjectParts {
    // nx = 0, px = 1, ny = 2, py = 3, nz = 4, pz = 5
    Block([Option<ComplexObjectSide>; 6]),
    TransportBelt([Option<ComplexObjectSide>; 6]),
}

pub struct ComplexObject {
    pub parts: Vec<ComplexObjectParts>
}

impl ComplexObject {
    #[inline]
    pub fn new(parts: Vec<ComplexObjectParts>) -> Self {
        Self { parts }
    }
}

// I'll rewrite this using files someday
pub fn new_transport_belt() -> ComplexObject {
    ComplexObject::new(vec![
        ComplexObjectParts::Block([
            // Negative x
            Some(ComplexObjectSide::new(9, vec![
                [([0.0, 0.0,   0.0], [0.0, 0.0]).into(), 
                 ([0.0, 0.25,  0.0], [0.0, 0.25]).into(),
                 ([0.0, 0.25,  1.0], [1.0, 0.25]).into(),
                 ([0.0, 0.0,   1.0], [1.0, 0.0]).into()].into(),
                [([0.875, 0.125, 0.0], [0.0, 0.125]).into(),
                 ([0.875, 0.25,  0.0], [0.0, 0.25]).into(),
                 ([0.875, 0.25,  1.0], [1.0, 0.25]).into(),
                 ([0.875, 0.125, 1.0], [1.0, 0.125]).into()].into()
            ])),
            // Positive x
            Some(ComplexObjectSide::new(9, vec![
                [([1.0, 0.0,   0.0], [0.0, 0.0]).into(), 
                 ([1.0, 0.25,  0.0], [0.0, 0.25]).into(),
                 ([1.0, 0.25,  1.0], [1.0, 0.25]).into(),
                 ([1.0, 0.0,   1.0], [1.0, 0.0]).into()].into(),
                [([0.125, 0.125, 0.0], [0.0, 0.125]).into(),
                 ([0.125, 0.25,  0.0], [0.0, 0.25]).into(),
                 ([0.125, 0.25,  1.0], [1.0, 0.25]).into(),
                 ([0.125, 0.125, 1.0], [1.0, 0.125]).into()].into()
            ])),
            // Negative y
            Some(ComplexObjectSide::new(9, vec![
                [([0.0, 0.0, 0.0], [0.0, 0.0]).into(), 
                 ([0.0, 0.0, 1.0], [0.0, 1.0]).into(),
                 ([1.0, 0.0, 1.0], [1.0, 1.0]).into(),
                 ([1.0, 0.0, 0.0], [1.0, 0.0]).into()].into()
            ])),
            // Positive y
            Some(ComplexObjectSide::new(9, vec![
                [([0.0,   0.25,   0.0], [0.0, 0.0]).into(), 
                 ([0.0,   0.25,   1.0], [0.0, 1.0]).into(),
                 ([0.125, 0.25,   1.0], [0.125, 1.0]).into(),
                 ([0.125, 0.25,   0.0], [0.125, 0.0]).into()].into(),
                [([0.875, 0.25,  0.0], [0.875, 0.0]).into(),
                 ([0.875, 0.25,  1.0], [0.875, 1.0]).into(),
                 ([1.0,   0.25,  1.0], [1.0, 1.0]).into(),
                 ([1.0,   0.25,  0.0], [1.0, 0.0]).into()].into()
            ])),
            // Negative z
            Some(ComplexObjectSide::new(9, vec![
                [([0.0, 0.0,   0.0], [0.0, 0.0]).into(), 
                 ([1.0, 0.0,   0.0], [0.0, 1.0]).into(),
                 ([1.0, 0.125, 0.0], [0.125, 1.0]).into(),
                 ([0.0, 0.125, 0.0], [0.125, 0.0]).into()].into(),
                [([0.0,   0.125, 0.0], [0.125, 0.0]).into(),
                 ([0.125, 0.125, 0.0], [0.125, 0.125]).into(),
                 ([0.125, 0.25,  0.0], [0.25, 0.125]).into(),
                 ([0.0,   0.25,  0.0], [0.125, 0.0]).into()].into(),
                [([0.875, 0.125, 0.0], [0.125, 0.875]).into(),
                 ([1.0,   0.125, 0.0], [0.125, 1.0]).into(),
                 ([1.0,   0.25,  0.0], [0.25, 1.0]).into(),
                 ([0.875, 0.25,  0.0], [0.25, 0.875]).into()].into(),
            ])),
            // Positive z
            Some(ComplexObjectSide::new(9, vec![
                [([0.0, 0.0,   1.0], [0.0, 0.0]).into(), 
                 ([1.0, 0.0,   1.0], [0.0, 1.0]).into(),
                 ([1.0, 0.125, 1.0], [0.125, 1.0]).into(),
                 ([0.0, 0.125, 1.0], [0.125, 0.0]).into()].into(),
                [([0.0,   0.125, 1.0], [0.125, 0.0]).into(),
                 ([0.125, 0.125, 1.0], [0.125, 0.125]).into(),
                 ([0.125, 0.25,  1.0], [0.25, 0.125]).into(),
                 ([0.0,   0.25,  1.0], [0.25, 0.0]).into()].into(),
                [([0.875, 0.125, 1.0], [0.125, 0.875]).into(),
                 ([1.0,   0.125, 1.0], [0.125, 1.0]).into(),
                 ([1.0,   0.25,  1.0], [0.25, 1.0]).into(),
                 ([0.875, 0.25,  1.0], [0.25, 0.875]).into()].into(),
            ])),
        ]),
        ComplexObjectParts::TransportBelt([
            // Negative x
            None,
            // Positive x
            None,
            // Negative y
            None,
            // Positive y
            Some(ComplexObjectSide::new(7, vec![
                [([0.125, 0.125, 0.0], [0.125, 0.0]).into(), 
                 ([0.125, 0.125, 1.0], [0.125, 1.0]).into(),
                 ([0.875, 0.125, 1.0], [0.875, 1.0]).into(),
                 ([0.875, 0.125, 0.0], [0.875, 0.0]).into()].into()
            ])),
            // Negative z
            None,
            // Positive z
            None
        ])
    ])
}