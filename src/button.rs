use core::sync::atomic::AtomicU8;
#[derive(Default, Clone, Copy)]
#[repr(u8)]
pub enum ButtonState {
    Pressed,
    #[default]
    Released,
}

impl From<u8> for ButtonState {
    fn from(value: u8) -> Self {
        match value {
            0 => ButtonState::Pressed,
            1 => ButtonState::Released,
            _ => panic!("Invalid value for ButtonState"),
        }
    }
}

impl ButtonState {
    pub fn to_printable(&self) -> &'static str {
        match self {
            ButtonState::Pressed => "Button::Pressed",
            ButtonState::Released => "Button::Released",
        }
    }
}
pub static BUTTON_STATE: AtomicU8 = AtomicU8::new(ButtonState::Released as u8);
