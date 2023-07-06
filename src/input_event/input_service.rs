use winit::event::{Event, WindowEvent, ElementState, DeviceEvent};

use crate::my_time::Time;

use super::{input_broker::{InputBroker}, KeypressState, InputOffset};

pub type Key = winit::event::VirtualKeyCode;
pub type Mouse = winit::event::MouseButton;

fn mouse_button_to_id_with_offset(mouse: &Mouse) -> usize {
    InputOffset::Mouse as usize + (match mouse {
        winit::event::MouseButton::Left => 0,
        winit::event::MouseButton::Right => 1,
        winit::event::MouseButton::Middle => 2,
        winit::event::MouseButton::Other(a) => 3 + *a as usize,
    })
}

#[derive(Debug)]
pub struct InputService {
    input_broker: InputBroker,
    delta_mouse: (Vec<f32>, Vec<f32>),
}


impl InputService {
    pub fn new() -> Self { Self { input_broker: InputBroker::new(), delta_mouse: (vec![], vec![]) } }
    pub fn delta(&self) -> &(f32, f32) { &self.input_broker.delta }
    pub fn coords(&self) -> &(f32, f32) { &self.input_broker.coords }


    pub fn is_keys(&self, keys: &[(Key, KeypressState)]) -> bool {
        let buttons: Vec<(usize, KeypressState)> = keys.iter().map(|key| {
            (key.0 as usize + InputOffset::Key as usize, key.1)
        }).collect();
        self.input_broker.is_buttons(&buttons[..])
    }


    pub fn is_mouse(&self, buttons: &[(Mouse, KeypressState)]) -> bool {
        let buttons: Vec<(usize, KeypressState)> = buttons.iter().map(|button| {
            (mouse_button_to_id_with_offset(&button.0), button.1)
        }).collect();
        self.input_broker.is_buttons(&buttons[..])
    }


    pub fn update_delta_mouse(&mut self) {
        let avg_x = match self.delta_mouse.0.len() {
            0 => 0.0,
            n => self.delta_mouse.0.iter().sum::<f32>() / n as f32
        };
        let avg_y = match self.delta_mouse.1.len() {
            0 => 0.0,
            n => self.delta_mouse.1.iter().sum::<f32>() / n as f32
        };

        self.delta_mouse.0.clear();
        self.delta_mouse.1.clear();

        self.input_broker.set_delta(avg_x, avg_y);
    }


    pub fn update(&mut self) {
        self.input_broker.update();
    }


    pub fn process_events(&mut self, time: &mut Time, event: &Event<'_, ()>) {
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::MouseInput { state, button, .. } => {
                        let is_press = *state == ElementState::Pressed;
                        self.input_broker.press(mouse_button_to_id_with_offset(button), is_press);
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(code) = input.virtual_keycode {
                            let is_press = input.state == ElementState::Pressed;
                            let id = code as usize + InputOffset::Key as usize;
                            self.input_broker.press(id, is_press);
                        }
                    },
                    WindowEvent::CursorMoved { position, .. } => {
                        self.input_broker.set_coords(position.x as f32, position.y as f32);
                    }
                    _ => {}
            }}
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    self.input_broker.set_delta(delta.0 as f32, delta.1 as f32);
                },
                // DeviceEvent::MouseMotion { delta } => {
                //     self.delta_mouse.0.push(delta.0 as f32);
                //     self.delta_mouse.1.push(delta.1 as f32);
                // },
               _ => {}
            }
            _ => {}
        }
    }
}