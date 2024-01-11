#[derive(Debug, Clone, Copy)]
pub struct Direction(f32, f32, f32);

impl Direction {
    pub fn new(x: f32, y: f32, z: f32) -> Self {Self(x, y, z)}
    pub fn new_x() -> Self {Self(1.0, 0.0, 0.0)}

    pub fn simplify_to_one_greatest(&self, x: bool, y: bool, z: bool) -> [i8; 3] {
        let mut direction = [
            (0, if x {self.0} else {0.0}),
            (1, if y {self.1} else {0.0}),
            (2, if z {self.2} else {0.0})
        ];
        direction.sort_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap());
        let mut result: [i8; 3] = [0, 0, 0];
        result[direction[2].0] = if direction[2].1 > 0.0 {1} else {-1};
        result
    }
}