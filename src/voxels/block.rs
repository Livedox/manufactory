use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy)]
    pub struct LightPermeability: u8 {
        const NONE = 0b000000;
        const RIGHT = 0b000001;
        const LEFT = 0b000010;
        const UP = 0b000100;
        const DOWN = 0b001000;
        const FRONT = 0b010000;
        const BACK = 0b100000;
        const X = 0b000011;
        const Y = 0b001100;
        const Z = 0b110000;
        const X_UP_Z = 0b110111;
        const ALL = 0b111111;
    }
}

impl LightPermeability {
    pub fn get_opposite_side(&self) -> LightPermeability {
        let mut even_bits: u8 = self.bits() & 0xAA;
        let mut odd_bits: u8 = self.bits() & 0x55;
        even_bits >>= 1;
        odd_bits <<= 1;
        LightPermeability::from_bits(even_bits | odd_bits).unwrap()
    }

    pub fn check_permeability(&self, permeability: &Self, side: &Self) -> bool {
        (*self & permeability.get_opposite_side() & *side).bits() > 0
    }
}

pub struct Block {
    pub id: u32,
    pub faces: [u32; 6], // -x x -y y -z z
    pub emission: [u8; 3],
    pub light_permeability: LightPermeability,
}


impl Block {
    pub const fn new_air(id: u32, face: u32, emission: u8) -> Block {
        Block {
            id,
            faces: [face; 6],
            emission: [emission; 3],
            light_permeability: LightPermeability::ALL
        }
    }
    pub const fn new(id: u32, face: u32, emission: u8) -> Block {
        Block {
            id,
            faces: [face; 6],
            emission: [emission; 3],
            light_permeability: LightPermeability::NONE,
        }
    }
    pub const fn new_with_faces(id: u32, faces: [u32; 6], emission: u8) -> Block {
        Block {
            id,
            faces,
            emission: [emission; 3],
            light_permeability: LightPermeability::Y,
        }
    }
    pub const fn new_a(id: u32, face: u32, emission: u8) -> Block {
        Block {
            id,
            faces: [face; 6],
            emission: [emission; 3],
            light_permeability: LightPermeability::X_UP_Z,
        }
    }
}


pub const BLOCKS: [Block; 8] = [
    Block::new_air(0, 0, 0),
    Block::new(1, 1, 0),
    Block::new(2, 2, 0),
    Block::new(3, 3, 15),
    Block::new(4, 4, 0),
    Block::new_with_faces(5, [6, 6, 6, 5, 6, 6], 0),
    Block::new_a(6, 4, 0),
    Block::new(7, 7, 0),
];

