use std::time::{Instant, Duration};


pub struct Timer {
    timer: Instant,
    step: Duration,
}

impl Timer {
    pub fn new(step: Duration) -> Self {
        Self {
            timer: Instant::now(),
            step,
        }
    }


    pub fn check(&mut self) -> bool {
        if self.timer.elapsed() >= self.step {
            self.timer = Instant::now();
            return true;
        }
        false
    }
}

pub struct Time {
    global_start: Instant,
    delta: f32,
    last_frame: f32,
}


impl Time {
    pub fn new() -> Self {
        let instant = Instant::now();
        Self {
            global_start: instant,
            delta: 0.0,
            last_frame: instant.elapsed().as_secs_f32(),
        }
    }


    pub fn current(&self) -> f32 {
        self.global_start.elapsed().as_secs_f32()
    }


    pub fn update(&mut self) {
        let current_time = self.current();
        self.delta = current_time - self.last_frame;
        self.last_frame = current_time;
    }

    pub fn delta(&self) -> f32 {
        self.delta
    }
}