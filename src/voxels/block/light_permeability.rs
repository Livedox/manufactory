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

impl Default for LightPermeability {
    fn default() -> Self { Self::ALL }
}

impl LightPermeability {
    pub fn get_opposite_side(&self) -> LightPermeability {
        let mut even_bits: u8 = self.bits() & 0xAA;
        let mut odd_bits: u8 = self.bits() & 0x55;
        even_bits >>= 1;
        odd_bits <<= 1;
        LightPermeability::from_bits(even_bits | odd_bits).unwrap()
    }

    pub fn check_permeability(self, permeability: Self, side: Self) -> bool {
        (self & permeability.get_opposite_side() & side).bits() > 0
    }

    pub fn sky_passing(&self) -> bool {
        self.contains(Self::Y)
    }
}