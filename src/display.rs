use ag_lcd::LcdDisplay;
use defmt::info;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use esp_hal::gpio::AnyPin;
use esp_hal::i2c::master::AnyI2c;
use heapless::FnvIndexMap;
use heapless::String;
use postcard::from_bytes;
use serde::{Deserialize, Serialize};
use ufloat::Uf32;
#[derive(Debug, Deserialize, Serialize, defmt::Format)]
pub struct CharBitMap<'a, const N: usize> {
    #[serde(borrow)]
    pub map: FnvIndexMap<&'a str, [u8; 8], N>,
}

#[allow(non_camel_case_types)]
pub struct i2cInterface {
    pub scl: AnyPin,
    pub sda: AnyPin,
    pub i2c: AnyI2c,
}

impl i2cInterface {
    pub fn new(scl: AnyPin, sda: AnyPin, i2c: AnyI2c) -> Self {
        Self { scl, sda, i2c }
    }
}

pub struct ExtendedLcdWriter<'a, T: OutputPin + Sized, D: DelayNs + Sized> {
    pub raw_display: &'a mut LcdDisplay<T, D>,
    symbol_map: CharBitMap<'a, 128>, //
    row: u8,
    col: u8,
    move_forward: bool,
    sym_index: u8,
    is_static: bool,
}

impl<'a, T, D> ExtendedLcdWriter<'a, T, D>
where
    T: OutputPin + Sized,
    D: DelayNs + Sized,
{
    pub fn new(raw_display: &'a mut LcdDisplay<T, D>) -> Self {
        Self {
            raw_display,
            symbol_map: CharBitMap::init(),
            row: 0,
            col: 0,
            move_forward: true,
            sym_index: 0,
            is_static: false,
        }
    }

    pub fn print(&mut self, message: &str) {
        // check for special characters in this string  and add then to the display wit
        // less than 8
        self.print_at(message, self.col);
    }

    pub fn print_at(&mut self, message: &str, position: u8) {
        let keys = message.split(":");
        self.move_forward = true;
        self.col = position;
        for key in keys {
            match self.symbol_map.get(key) {
                Some(pixels) => {
                    let pixels = pixels.clone();
                    self.raw_display.set_character(self.sym_index, pixels);
                    self.update_position();
                    self.raw_display.write(self.sym_index);
                    self.sym_index = self.sym_index.saturating_add(1);
                    if self.sym_index > 7 {
                        self.sym_index = 0;
                    }
                }
                _ => {
                    for ch in key.chars() {
                        self.update_position();
                        self.raw_display.write(ch as u8);
                    }
                }
            }
        }
    }

    fn update_position(&mut self) {
        if self.move_forward {
            self.raw_display.set_position(self.col, self.row);
        }
        self.col = self.col.saturating_add(1);
        if self.col > 16 {
            self.raw_display.scroll_left(1);
        }
        self.row = self.row.saturating_add(1);
    }

    pub fn home(&mut self) {
        self.col = 0;
        self.raw_display.clear();
        self.raw_display.home();
    }
    pub fn set_static(&mut self) {
        self.raw_display.clear();
        self.raw_display.set_position(0, 0);
        self.is_static = true;
    }
}
impl<const N: usize> CharBitMap<'_, N> {
    pub fn init() -> Self {
        // get the values for our json file
        let symbol_data: &[u8] = include_bytes!("../display_symbols.txt");
        let map: CharBitMap<N> = from_bytes(symbol_data).expect("failed to serialized");
        info!("screen symbols and emojis loaded!");
        map
    }

    pub fn get(&self, key: &str) -> Option<&[u8; 8]> {
        self.map.get(key)
    }
}

pub fn round(x: f32, decimals: usize) -> Uf32 {
    ufloat::Uf32(x, decimals)
}

pub fn float_to_str(f: Uf32) -> String<32> {
    use ufmt::uwrite;
    let mut buf: String<32> = String::new();
    uwrite!(&mut buf, "{}", f).unwrap(); // Format with 2 decimal places
    buf
}
