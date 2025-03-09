pub mod display;
pub use display::*;
mod button;
pub use button::*;
mod keypad;
use esp_hal::gpio::{AnyPin, Flex, Level, Pull};
pub use keypad::Keypad;

pub fn create_open_drain_pin<'a>(pin: AnyPin) -> Flex<'a> {
    let mut od_pin = Flex::new(pin);
    od_pin.set_level(Level::Low);
    od_pin.set_as_open_drain(Pull::Up);
    od_pin
}
