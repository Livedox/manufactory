pub mod input_broker;
pub mod input_service;

use bitflags::bitflags;


const TIME_BETWEEN_TWO_PRESS: f32 = 0.3;

const KEY_LENGTH: usize = 255;
const MOUSE_LENGTH: usize = 32;
const INPUT_LENGTH: usize = KEY_LENGTH+MOUSE_LENGTH;

#[repr(usize)]
pub(super) enum InputOffset {
    Key = 0,
    Mouse = KEY_LENGTH
}


bitflags! {
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct KeypressState: u8 {
        const JustPressed = 0b00000001;
        const Pressed = 0b00000010;
        const AnySinglePress = 0b00000011;

        const JustDoublePressed = 0b00000100;
        const DoublePressed = 0b00001000;
        const AnyDoublePress = 0b00001100;

        const JustTriplePressed = 0b00010000;
        const TriplePressed = 0b00100000;
        const AnyTriplePress = 0b00110000;

        const AnyJustPress = 0b00010101;
        const AnyStayPress = 0b00101010;
        const AnyPress = 0b00111111;

        const JustReleased = 0b01000000;
        const Released = 0b10000000;
        const AnyReleased = 0b11000000;

        const AnyJust = 0b01010101;
    }
}
impl Default for KeypressState { fn default() -> Self { Self::Released } }
impl KeypressState {
    pub fn is(&self, state: Self) -> bool {
        (*self & state).bits() > 0
    }

    pub(super) fn to_stay(self) -> Self {
        KeypressState::from_bits(self.0.bits() << 1).unwrap()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct State {
    state: KeypressState,
    last_press_state: KeypressState,
    last_press_time: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            state: KeypressState::default(),
            last_press_state: KeypressState::default(),
            last_press_time: -TIME_BETWEEN_TWO_PRESS,
        }
    }
}