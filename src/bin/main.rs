#![no_std]
#![no_main]
#![feature(lazy_get)]
use core::sync::atomic::Ordering;

use ag_lcd::{Cursor, LcdDisplay};
use defmt::info;
use embassy_executor::Spawner;
use embassy_time::Timer;
use esp32::{
    create_open_drain_pin, i2cInterface, ButtonState, ExtendedLcdWriter, Keypad, BUTTON_STATE,
};
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{AnyPin, Flex, Input, InputConfig, Level, Output, OutputConfig, Pin, Pull};
use esp_hal::i2c;
use esp_hal::i2c::master::Config;
use esp_hal::time::Rate;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;
use {esp_backtrace as _, esp_println as _};

use heapless::String;
use mpu6050_dmp::address::Address;
use mpu6050_dmp::sensor::Mpu6050;
use port_expander::Pcf8574;
extern crate alloc;
#[embassy_executor::task]
async fn blink(pin: AnyPin) {
    let mut led = Output::new(pin, Level::Low, OutputConfig::default().with_pull(Pull::Up));
    loop {
        let state = ButtonState::from(BUTTON_STATE.load(Ordering::Relaxed));

        match state {
            ButtonState::Pressed => {
                if led.is_set_low() {
                    info!("{}", state.to_printable());

                    led.set_high();
                }
            }
            ButtonState::Released => led.set_low(),
        };
        Timer::after_millis(100).await;
    }
}

#[embassy_executor::task]
async fn handle_reads(
    lcd_pins: i2cInterface,
    sensor_pins: i2cInterface,
    switch_pin: AnyPin,
    mut keypad: Keypad<Flex<'static>, Flex<'static>>,
) {
    let i2cInterface { scl, sda, i2c } = lcd_pins;
    let i2c_bus = i2c::master::I2c::new(i2c, Config::default())
        .unwrap()
        .with_scl(scl)
        .with_sda(sda);
    let config = esp_hal::gpio::InputConfig::default().with_pull(Pull::None);
    let switch = Input::new(switch_pin, config);
    let mut i2c_expander = Pcf8574::new(i2c_bus, true, true, true);
    let delay = Delay::new();
    let mut lcd = LcdDisplay::new_pcf8574(&mut i2c_expander, delay)
        .with_cursor(Cursor::On)
        .with_blink(ag_lcd::Blink::On)
        .build();
    // mpu6050 sensor
    let i2cInterface { scl, sda, i2c } = sensor_pins;
    let i2c_bus = i2c::master::I2c::new(i2c, Config::default().with_frequency(Rate::from_khz(400)))
        .unwrap()
        .with_scl(scl)
        .with_sda(sda);

    let mut mpu = Mpu6050::new(i2c_bus, Address::default()).unwrap();
    let mut delay = Delay::new();
    mpu.initialize_dmp(&mut delay)
        .expect("failed to initialized sensor");
    let mut extended_writer = ExtendedLcdWriter::new(&mut lcd);
    extended_writer.print(":heart: Welcome :heart:");
    extended_writer.print(":cdot: :sdot: :circle:");
    // extended_writer.print(":ascchart: :arrowright:");
    loop {
        let state = switch.is_low();
        match state {
            true => {
                extended_writer.raw_display.display_on();
            }
            _ => {
                extended_writer.raw_display.display_off();
            }
        };
        //match mpu.temperature() {
        //    Ok(reading) => {
        //        let mut temp_string: String<128> = heapless::String::new();
        //        _ = temp_string.push_str("Temp-> ");
        //        let temp_str = float_to_str(round(reading.celsius(), 2));
        //        _ = temp_string.push_str(&temp_str);
        //        _ = temp_string.push_str(" :cdot: C");
        //        extended_writer.home();
        //        //extended_writer.set_static();
        //        extended_writer.print_at(&temp_string, 0);
        //    }
        //    Err(err) => println!("{:?}", err),
        //}
        let character = keypad.read_char(&mut delay);
        info!("{}", character);
        let mut temp_string: String<1> = heapless::String::new();
        temp_string.push(character).ok();
        extended_writer.raw_display.print(&temp_string);

        Timer::after_millis(200).await;
    }
}
#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    esp_alloc::heap_allocator!(size: 20 * 1024);
    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);
    info!("Embassy initialized!");

    let timer1 = TimerGroup::new(peripherals.TIMG0);
    let _init = esp_wifi::init(
        timer1.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();
    let _config = esp_hal::gpio::InputConfig::default().with_pull(Pull::Up);
    // let mut button = Input::new(peripherals.GPIO10, config);
    // LCD;
    let sda = peripherals.GPIO17.degrade();
    let scl = peripherals.GPIO18.degrade();
    let i2c = peripherals.I2C0.into();
    let lcd_pins = i2cInterface::new(scl, sda, i2c);
    // Mpu6050 sensor pins;
    let sda = peripherals.GPIO15.degrade();
    let scl = peripherals.GPIO16.degrade();
    let i2c = peripherals.I2C1.into();
    let sensor_pins = i2cInterface::new(scl, sda, i2c);
    // keypad;
    let rows = (
        create_open_drain_pin(peripherals.GPIO13.degrade()),
        create_open_drain_pin(peripherals.GPIO12.degrade()),
        create_open_drain_pin(peripherals.GPIO11.degrade()),
        create_open_drain_pin(peripherals.GPIO10.degrade()),
    );
    let _config = InputConfig::default().with_pull(Pull::None);
    let cols = (
        Flex::new(peripherals.GPIO9.degrade()),
        Flex::new(peripherals.GPIO46.degrade()),
        Flex::new(peripherals.GPIO3.degrade()),
        Flex::new(peripherals.GPIO8.degrade()),
    );
    let keypad = Keypad::new(cols, rows);
    spawner
        .spawn(handle_reads(
            lcd_pins,
            sensor_pins,
            peripherals.GPIO4.degrade(),
            keypad,
        ))
        .unwrap();
    // spawner.spawn(blink(peripherals.GPIO6.degrade())).unwrap();
    // loop {
    //     button.wait_for_low().await;
    //     BUTTON_STATE.store(ButtonState::Pressed as u8, Ordering::Relaxed);
    //     button.wait_for_high().await;

    //     BUTTON_STATE.store(ButtonState::Released as u8, Ordering::Relaxed);
    //     Timer::after_millis(300).await
    // }
    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.0/examples/src/bin
}
