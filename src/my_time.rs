use std::time::{Instant, Duration};

pub struct Time {
    global_start: Instant,
    timer_1s: Instant,
    one_second: Duration,
    delta: f32,
    smooth_delta: f32,
    smooth_delta_vec: Vec<f32>,
    last_frame: f32,
}


impl Time {
    pub fn new() -> Self {
        let instant = Instant::now();
        Self {
            global_start: instant,
            timer_1s: instant,
            one_second: Duration::new(1, 0),
            delta: 0.0,
            last_frame: instant.elapsed().as_secs_f32(),
            smooth_delta: 0.0,
            smooth_delta_vec: vec![]
        }
    }


    pub fn is_more_then_1s(&mut self) -> bool {
        let is_more = self.timer_1s.elapsed() >= self.one_second;
        if is_more { self.timer_1s = Instant::now(); }
        is_more
    }


    pub fn current(&self) -> f32 {
        self.global_start.elapsed().as_secs_f32()
    }


    pub fn update(&mut self) {
        let current_time = self.current();
        self.delta = current_time - self.last_frame;
        self.last_frame = current_time;
        self.smooth_delta_vec.push(self.delta);
        if self.smooth_delta_vec.len() > 9 {
            self.smooth_delta = self.smooth_delta_vec.iter().sum::<f32>() / self.smooth_delta_vec.len() as f32;
            self.smooth_delta_vec.clear();
        }
    }

    pub fn delta(&self) -> f32 {
        self.delta
    }

    pub fn smooth_delta(&self) -> f32 {
        self.smooth_delta
    }
}