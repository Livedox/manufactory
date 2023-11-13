use std::time::Instant;

#[derive(Debug)]
pub struct Color(pub f32, pub f32, pub f32);

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Self {Self(r, g, b)}

    pub fn interpolation(&self, color: &Color, progress: f32) -> Color {
        Color(
            self.0 + (color.0 - self.0)*progress,
            self.1 + (color.1 - self.1)*progress,
            self.2 + (color.2 - self.2)*progress
        )
    }
}

impl From<Color> for [f32; 3] {
    fn from(color: Color) -> Self {[color.0, color.1, color.2]}
}

#[derive(Debug)]
pub struct Sun<const N: usize> {
    start_offset: f32,
    start: Instant,
    time_start: [u64; N],
    sun: [Color; N],
    sky: [Color; N],
}


impl<const N: usize> Sun<N> {
    pub fn new(start: u64, time_start: [u64; N], sun: [Color; N], sky: [Color; N]) -> Self {Self {
        start_offset: start as f32,
        start: Instant::now(),
        time_start,
        sun,
        sky
    }}

    pub fn sun_sky(&self) -> (Color, Color) {
        let time = (self.start.elapsed().as_secs_f32() + self.start_offset) % *self.time_start.last().unwrap() as f32;
        for i in (0..(self.sun.len()-1)).rev() {
            if time >= self.time_start[i] as f32 {
                let end_progress = self.time_start[i + 1] - self.time_start[i];
                let progress = (time - self.time_start[i] as f32) / end_progress as f32;
                return (self.sun[i].interpolation(&self.sun[i + 1], progress),
                    self.sky[i].interpolation(&self.sky[i + 1], progress));
            }
        }
        unreachable!("Can't get sun_sky")
    }
}