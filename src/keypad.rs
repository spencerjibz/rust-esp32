use defmt::info;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};

/// Defines a type that makes it easier to supply the four pins required for rows in the keypad.
/// These pins need to support the `embedded_hal::digital::v2::InputPin` trait
pub type Rows<R> = (R, R, R, R);

/// Defines a type that makes it easier to supply the four pins required for rows in the keypad
/// These pins need to support the `embedded_hal::digital::v2::OutputPin` trait
pub type Columns<C> = (C, C, C, C);

/// Manages the pins and the logic for scanning a keypad
pub struct Keypad<R: InputPin, C: OutputPin> {
    rows: Rows<R>,
    columns: Columns<C>,
}

impl<R: InputPin, C: OutputPin> Keypad<R, C> {
    /// Create a new instance of this structure
    pub fn new(rows: Rows<R>, columns: Columns<C>) -> Self {
        Self { rows, columns }
    }

    /**
    Reads a character from the keypad. This method returns even if no keys are pressed.
    It will return:

    * `'0'` through `'9'`
    * `'*'`
    * `'#'`
    * `' '` if no keys are pressed.
    */
    pub fn read_char(&mut self, delay: &mut dyn DelayNs) -> char {
        let raw = self.read(delay);
        info!(" raw = {}", raw);
        if raw != 0 {
            self.get_char(raw)
        } else {
            ' '
        }
    }

    // Performs a "raw" read of the keypad and returns a bit set for each key down. Note,
    // this doesn't mean this code supports multiple key presses.
    fn read(&mut self, delay: &mut dyn DelayNs) -> u16 {
        let mut res = 0;

        self.columns.0.set_low().unwrap_or_default();
        res |= self.read_column(delay);
        self.columns.0.set_high().unwrap_or_default();

        self.columns.1.set_low().unwrap_or_default();
        res |= self.read_column(delay) << 4;
        self.columns.1.set_high().unwrap_or_default();

        self.columns.2.set_low().unwrap_or_default();
        res |= self.read_column(delay) << 8;
        self.columns.2.set_high().unwrap_or_default();

        self.columns.3.set_low().unwrap_or_default();
        res |= self.read_column(delay) << 12;
        self.columns.3.set_high().unwrap_or_default();
        res
    }

    // Converts the raw value from the read() method into a character that corresponds to the
    // label on each key
    fn get_char(&self, raw_value: u16) -> char {
        let value = self.convert(raw_value);
        info!("{}", value);
        match value {
            -1 => '*',
            -2 => '#',
            -3 => 'A',
            -4 => 'B',
            -5 => 'C',
            -6 => 'D',
            _ => char::from_digit(value as u32, 10).unwrap(),
        }
    }

    fn read_column(&mut self, delay: &mut dyn DelayNs) -> u16 {
        let mut res = 0;

        delay.delay_ms(1);
        if self.rows.0.is_low().unwrap_or_default() {
            res |= 1 << 0;
        }
        if self.rows.1.is_low().unwrap_or_default() {
            res |= 1 << 1;
        }
        if self.rows.2.is_low().unwrap_or_default() {
            res |= 1 << 2;
        }
        if self.rows.3.is_low().unwrap_or_default() {
            res |= 1 << 3;
        }

        res
    }

    // Converts the raw value (2^N) from the read() method into a keypad digit. This will be
    //      0..9    digits
    //      -1      *
    //      -2      #
    pub fn convert(&self, value: u16) -> i16 {
        match value {
            KEYA => -3,
            KEYB => -4,
            KEYC => -5,
            KEYD => -6,
            KEY_0 => 0,
            KEY_1 => 1,
            KEY_2 => 2,
            KEY_3 => 3,
            KEY_4 => 4,
            KEY_5 => 5,
            KEY_6 => 6,
            KEY_7 => 7,
            KEY_8 => 8,
            KEY_9 => 9,
            KEY_HASH => -2,
            KEY_STAR => -1,
            _ => -10,
        }
    }
}

const KEYA: u16 = 1 << 12;
const KEYB: u16 = 1 << 13;
const KEYC: u16 = 1 << 14;
const KEYD: u16 = 1 << 15;
const KEY_0: u16 = 1 << 7;
const KEY_1: u16 = 1;
const KEY_2: u16 = 1 << 4;
const KEY_3: u16 = 1 << 8;
const KEY_4: u16 = 1 << 1;
const KEY_5: u16 = 1 << 5;
const KEY_6: u16 = 1 << 9;
const KEY_7: u16 = 1 << 2;
const KEY_8: u16 = 1 << 6;
const KEY_9: u16 = 1 << 10;
const KEY_HASH: u16 = 1 << 11;
const KEY_STAR:u16 = 1 << 3;