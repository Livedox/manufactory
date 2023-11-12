use winit::{window::Window, dpi::PhysicalPosition};

pub struct CursorLock {
    is_cursor: bool,
}


impl CursorLock {
    pub fn new() -> Self { Self {is_cursor: true} }

    pub fn update(&mut self, window: &Window) {
        if !self.is_cursor {
            let size = window.inner_size();
            let position = PhysicalPosition::new(size.width as f32/2.0, size.height as f32/2.0);
            window.set_cursor_position(position).unwrap();
        };
    }

    pub fn set_cursor_lock(&mut self, window: &Window, is_cursor: bool) {
        self.is_cursor = is_cursor;
        use winit::window::CursorGrabMode;
        let mode = if is_cursor {CursorGrabMode::None} else {CursorGrabMode::Confined};
        
        window.set_cursor_grab(mode).unwrap();
        window.set_cursor_visible(is_cursor);
    }

    pub fn is_cursor(&self) -> bool { self.is_cursor }
}