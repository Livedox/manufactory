use winit::event::{Event, WindowEvent, ElementState, DeviceEvent};

use super::{input_broker::InputBroker, KeypressState, InputOffset};

pub type Key = winit::event::VirtualKeyCode;
pub type Mouse = winit::event::MouseButton;


#[derive(Debug)]
pub struct InputService {
    input_broker: InputBroker,
}


impl InputService {
    pub fn new() -> Self { Self { input_broker: InputBroker::new() } }
    pub fn delta(&self) -> &(f32, f32) { &self.input_broker.delta }
    pub fn coords(&self) -> &(f32, f32) { &self.input_broker.coords }

    fn to_mouse_id(mouse: &Mouse) -> usize {
        InputOffset::Mouse as usize + (match mouse {
            winit::event::MouseButton::Left => 0,
            winit::event::MouseButton::Right => 1,
            winit::event::MouseButton::Middle => 2,
            winit::event::MouseButton::Other(a) => 3 + *a as usize,
        })
    }

    pub fn is_key(&self, key: &Key, state: KeypressState) -> bool {
        self.input_broker.is_button(*key as usize + InputOffset::Key as usize, state)
    }

    pub fn is_mouse(&self, mouse: &Mouse, state: KeypressState) -> bool {
        self.input_broker.is_button(Self::to_mouse_id(mouse), state)
    }

    pub fn update_delta_mouse(&mut self) {
        self.input_broker.update_delta_mouse();
    }

    pub fn update(&mut self) {
        self.input_broker.update();
    }

    pub fn wheel(&self) -> i8 {
        self.input_broker.wheel()
    }

    pub fn handle_event(&mut self, event: &Event<'_, ()>) {
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::MouseInput { state, button, .. } => {
                        let is_press = *state == ElementState::Pressed;
                        self.input_broker.press(Self::to_mouse_id(button), is_press);
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
                    },
                    WindowEvent::MouseWheel { delta, .. } => {
                        if let winit::event::MouseScrollDelta::LineDelta(_, y) = delta {
                            self.input_broker.set_wheel(*y as i8);
                        }
                    },
                    _ => {}
            }}
            Event::DeviceEvent { event, .. } => if let DeviceEvent::MouseMotion { delta } = event {
                self.input_broker.set_delta(delta.0 as f32, delta.1 as f32);
            }
            _ => {}
        }
    }
}


impl Default for InputService {
    fn default() -> Self {
        Self::new()
    }
}