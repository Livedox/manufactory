use std::time::Instant;

use crate::input_event::KeypressState;

use super::{State, TIME_BETWEEN_TWO_PRESS, INPUT_LENGTH};


#[derive(Debug)]
pub(super) struct InputBroker {
    instant: Instant,
    keys: [State; INPUT_LENGTH],
    prev_id: Option<usize>,
    pub(super) delta: (f32, f32),
    pub(super) coords: (f32, f32),
}


impl InputBroker {
    pub(super) fn new() -> Self {
        Self {
            instant: Instant::now(),
            keys: [State::default(); INPUT_LENGTH],
            prev_id: None,
            delta: (0.0, 0.0),
            coords: (0.0, 0.0),
        }
    }


    pub(super) fn set_delta(&mut self, delta_x: f32, delta_y: f32) {
        self.delta = (delta_x, delta_y);
    }


    pub(super) fn set_coords(&mut self, x: f32, y: f32) {
        self.coords = (x, y);
    }


    pub(super) fn is_buttons(&self, buttons: &[(usize, KeypressState)]) -> bool {
        buttons.iter().all(|button| {
            button.0 < INPUT_LENGTH && self.keys[button.0].state.is(button.1)
        })
    }


    pub(super) fn update(&mut self) {
        self.update_prev_key();
        self.delta = (0.0, 0.0);
    }


    fn update_prev_key(&mut self) {
        if let Some(id) = self.prev_id {
            if self.keys[id].state.is(KeypressState::AnyJust) {
                self.keys[id].state = self.keys[id].state.to_stay();
            }
            self.prev_id = None;
        }
    }


    pub(super) fn press(&mut self, id: usize, is_press: bool) {
        if id > INPUT_LENGTH { return; }
        self.update_prev_key();
        let state = self.keys[id].state;

        if is_press && !state.is(KeypressState::AnyPress) {
            let last_press_time = self.keys[id].last_press_time;
            let last_press_state = self.keys[id].last_press_state;
            let now = self.instant.elapsed().as_secs_f32();
            let mut new_state = KeypressState::JustPressed;

            if now - last_press_time < TIME_BETWEEN_TWO_PRESS {
                if last_press_state.is(KeypressState::AnySinglePress) {
                    new_state = KeypressState::JustDoublePressed;
                }
                if last_press_state.is(KeypressState::AnyDoublePress) {
                    new_state = KeypressState::JustTriplePressed;
                }
            }
            self.keys[id].state = new_state;
            self.keys[id].last_press_state = new_state;
            self.keys[id].last_press_time = now;
            self.prev_id = Some(id);
        } else if !is_press {
            self.keys[id].state = KeypressState::JustReleased;
            self.prev_id = Some(id);
        }  
    }
}