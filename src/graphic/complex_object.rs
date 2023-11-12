pub struct ComplexObjectVertex {
    pub xyz: [f32; 3],
    pub uv: [f32; 2],
}

impl ComplexObjectVertex {
    pub fn new(xyz: [f32; 3], uv: [f32; 2]) -> Self { Self {xyz, uv} }

    pub fn x(&self, rotation_index: usize) -> f32 {
        if rotation_index > 1 {self.xyz[2]} else {self.xyz[0]}
        // Same
        // self.xyz[rotation_index & 0b10]
    }

    pub fn y(&self) -> f32 {self.xyz[1]}

    pub fn z(&self, rotation_index: usize) -> f32 {
        if rotation_index > 1 {self.xyz[0]} else {self.xyz[2]}
        // Same
        // self.xyz[rotation_index ^ 0b10 & 0b10]
    }

    pub fn u(&self) -> f32 {self.uv[0]}
    pub fn v(&self) -> f32 {self.uv[1]}
}

pub struct ComplexObjectGroup(pub [ComplexObjectVertex; 4]);

impl ComplexObjectGroup {
    const VERTEX_ORDER: [[usize; 4]; 4] = [
        [0, 1, 2, 3],
        [2, 3, 0, 1],
        [3, 2, 1, 0],
        [1, 0, 3, 2]
    ];

    pub fn get(&self, rotation_index: usize, index: usize) -> &ComplexObjectVertex {
        &self.0[Self::VERTEX_ORDER[rotation_index][index]]
    }

    pub fn x(&self, rotation_index: usize, index: usize) -> f32 {
        self.0[Self::VERTEX_ORDER[rotation_index][index]].x(rotation_index)
    }

    pub fn y(&self, rotation_index: usize, index: usize) -> f32 {
        self.0[Self::VERTEX_ORDER[rotation_index][index]].y()
    }

    pub fn z(&self, rotation_index: usize, index: usize) -> f32 {
        self.0[Self::VERTEX_ORDER[rotation_index][index]].z(rotation_index)
    }

    pub fn u(&self, index: usize) -> f32 {self.0[index].u()}
    pub fn v(&self, index: usize) -> f32 {self.0[index].v()}
}

pub struct ComplexObjectSide {
    pub texture_layer: u32,
    pub vertex_groups: Vec<ComplexObjectGroup>,
}

impl ComplexObjectSide {
    pub fn new_zero() -> Self {
        Self { texture_layer: 0, vertex_groups: vec![] }
    }
}

pub struct ComplexObjectPart {
    pub positive_x: ComplexObjectSide,
    pub negative_x: ComplexObjectSide,
    
    pub positive_y: ComplexObjectSide,
    pub negative_y: ComplexObjectSide,

    pub positive_z: ComplexObjectSide,
    pub negative_z: ComplexObjectSide,
}

pub enum ComplexObjectParts {
    Block(ComplexObjectPart),
    TransportBelt(ComplexObjectPart),
}

pub struct ComplexObject {
    pub parts: Vec<ComplexObjectParts>
}

pub fn new_transport_belt() -> ComplexObject {
    let positive_x = ComplexObjectSide {
        texture_layer: 9,
        vertex_groups: vec![
            ComplexObjectGroup([
                ComplexObjectVertex::new([1.0, 0.0,   0.0], [0.0, 0.0]),
                ComplexObjectVertex::new([1.0, 0.25,  0.0], [0.0, 0.25]),
                ComplexObjectVertex::new([1.0, 0.25,  1.0], [1.0, 0.25]),
                ComplexObjectVertex::new([1.0, 0.0,   1.0], [1.0, 0.0]),
            ]),
            ComplexObjectGroup([
                ComplexObjectVertex::new([0.125, 0.125, 0.0], [0.0, 0.125]),
                ComplexObjectVertex::new([0.125, 0.25,  0.0], [0.0, 0.25]),
                ComplexObjectVertex::new([0.125, 0.25,  1.0], [1.0, 0.25]),
                ComplexObjectVertex::new([0.125, 0.125, 1.0], [1.0, 0.125]),
            ])
        ]
    };
    let negative_x = ComplexObjectSide {
        texture_layer: 9,
        vertex_groups: vec![
            ComplexObjectGroup([
                ComplexObjectVertex::new([0.0, 0.0,   0.0], [0.0, 0.0]),
                ComplexObjectVertex::new([0.0, 0.25,  0.0], [0.0, 0.25]),
                ComplexObjectVertex::new([0.0, 0.25,  1.0], [1.0, 0.25]),
                ComplexObjectVertex::new([0.0, 0.0,   1.0], [1.0, 0.0]),
            ]),
            ComplexObjectGroup([
                ComplexObjectVertex::new([0.875, 0.125, 0.0], [0.0, 0.125]),
                ComplexObjectVertex::new([0.875, 0.25,  0.0], [0.0, 0.25]),
                ComplexObjectVertex::new([0.875, 0.25,  1.0], [1.0, 0.25]),
                ComplexObjectVertex::new([0.875, 0.125, 1.0], [1.0, 0.125]),
            ])
        ]
    };
    let positive_y = ComplexObjectSide {
        texture_layer: 9,
        vertex_groups: vec![
            ComplexObjectGroup([
                ComplexObjectVertex::new([0.0,   0.25,   0.0], [0.0, 0.0]),
                ComplexObjectVertex::new([0.0,   0.25,   1.0], [0.0, 1.0]),
                ComplexObjectVertex::new([0.125, 0.25,   1.0], [0.125, 1.0]),
                ComplexObjectVertex::new([0.125, 0.25,   0.0], [0.125, 0.0]),
            ]),
            ComplexObjectGroup([
                ComplexObjectVertex::new([0.875, 0.25,  0.0], [0.875, 0.0]),
                ComplexObjectVertex::new([0.875, 0.25,  1.0], [0.875, 1.0]),
                ComplexObjectVertex::new([1.0,   0.25,  1.0], [1.0, 1.0]),
                ComplexObjectVertex::new([1.0,   0.25,  0.0], [1.0, 0.0]),
            ])
        ]
    };
    let negative_y = ComplexObjectSide {
        texture_layer: 9,
        vertex_groups: vec![
            ComplexObjectGroup([
                ComplexObjectVertex::new([0.0, 0.0, 0.0], [0.0, 0.0]),
                ComplexObjectVertex::new([0.0, 0.0, 1.0], [0.0, 1.0]),
                ComplexObjectVertex::new([1.0, 0.0, 1.0], [1.0, 1.0]),
                ComplexObjectVertex::new([1.0, 0.0, 0.0], [1.0, 0.0]),
            ]),
        ]
    };
    let positive_z = ComplexObjectSide {
        texture_layer: 9,
        vertex_groups: vec![
            ComplexObjectGroup([
                ComplexObjectVertex::new([0.0, 0.0,   1.0], [0.0, 0.0]),
                ComplexObjectVertex::new([1.0, 0.0,   1.0], [0.0, 1.0]),
                ComplexObjectVertex::new([1.0, 0.125, 1.0], [0.125, 1.0]),
                ComplexObjectVertex::new([0.0, 0.125, 1.0], [0.125, 0.0]),
            ]),
            ComplexObjectGroup([
                ComplexObjectVertex::new([0.0,   0.125, 1.0], [0.125, 0.0]),
                ComplexObjectVertex::new([0.125, 0.125, 1.0], [0.125, 0.125]),
                ComplexObjectVertex::new([0.125, 0.25,  1.0], [0.25, 0.125]),
                ComplexObjectVertex::new([0.0,   0.25,  1.0], [0.25, 0.0]),
            ]),
            ComplexObjectGroup([
                ComplexObjectVertex::new([0.875, 0.125, 1.0], [0.125, 0.875]),
                ComplexObjectVertex::new([1.0,   0.125, 1.0], [0.125, 1.0]),
                ComplexObjectVertex::new([1.0,   0.25,  1.0], [0.25, 1.0]),
                ComplexObjectVertex::new([0.875, 0.25,  1.0], [0.25, 0.875]),
            ])
        ]
    };
    let negative_z = ComplexObjectSide {
        texture_layer: 9,
        vertex_groups: vec![
            ComplexObjectGroup([
                ComplexObjectVertex::new([0.0, 0.0,   0.0], [0.0, 0.0]),
                ComplexObjectVertex::new([1.0, 0.0,   0.0], [0.0, 1.0]),
                ComplexObjectVertex::new([1.0, 0.125, 0.0], [0.125, 1.0]),
                ComplexObjectVertex::new([0.0, 0.125, 0.0], [0.125, 0.0]),
            ]),
            ComplexObjectGroup([
                ComplexObjectVertex::new([0.0,   0.125, 0.0], [0.125, 0.0]),
                ComplexObjectVertex::new([0.125, 0.125, 0.0], [0.125, 0.125]),
                ComplexObjectVertex::new([0.125, 0.25,  0.0], [0.25, 0.125]),
                ComplexObjectVertex::new([0.0,   0.25,  0.0], [0.125, 0.0]),
            ]),
            ComplexObjectGroup([
                ComplexObjectVertex::new([0.875, 0.125, 0.0], [0.125, 0.875]),
                ComplexObjectVertex::new([1.0,   0.125, 0.0], [0.125, 1.0]),
                ComplexObjectVertex::new([1.0,   0.25,  0.0], [0.25, 1.0]),
                ComplexObjectVertex::new([0.875, 0.25,  0.0], [0.25, 0.875]),
            ])
        ]
    };

    let default = ComplexObjectPart {
        positive_x,
        negative_x,
        positive_y,
        negative_y,
        positive_z,
        negative_z,
    };

    let belt_positive_y = ComplexObjectSide {
        texture_layer: 7,
        vertex_groups: vec![
            ComplexObjectGroup([
                ComplexObjectVertex::new([0.125, 0.125, 0.0], [0.125, 0.0]),
                ComplexObjectVertex::new([0.125, 0.125, 1.0], [0.125, 1.0]),
                ComplexObjectVertex::new([0.875, 0.125, 1.0], [0.875, 1.0]),
                ComplexObjectVertex::new([0.875, 0.125, 0.0], [0.875, 0.0]),
            ]),
        ]
    };

    let belt = ComplexObjectPart {
        positive_x: ComplexObjectSide::new_zero(),
        negative_x: ComplexObjectSide::new_zero(),
        positive_y: belt_positive_y,
        negative_y: ComplexObjectSide::new_zero(),
        positive_z: ComplexObjectSide::new_zero(),
        negative_z: ComplexObjectSide::new_zero(),
    };

    ComplexObject {
        parts: vec![
            ComplexObjectParts::Block(default),
            ComplexObjectParts::TransportBelt(belt)
        ]
    }
}