//! Continuously read the temperature with the LM75 and display it in
//! an SSD1306 OLED display.
//!
//! This example is for the STM32F103 "Blue Pill" board using I2C1.
//!
//! Wiring connections are as follows:
//!
//! ```
//!    Blue Pill <-> LM75 <-> Display
//! (black)  GND <-> GND  <-> GND
//! (red)    VCC <-> +5V  <-> +5V
//! (yellow) PB9 <-> SDA  <-> SDA
//! (green)  PB8 <-> SCL  <-> SCL
//! ```
//!
//! Run on a Blue Pill with:
//! `cargo run --example bp-display --target thumbv7m-none-eabi`,
//! currently only works on nightly.

#![deny(unsafe_code)]
#![no_std]
#![no_main]

// panic handler
extern crate panic_semihosting;
extern crate embedded_graphics;

use cortex_m_rt::entry;
use stm32f103xx_hal::{
    i2c::{BlockingI2c, DutyCycle, Mode}, prelude::*,
};
use lm75::{Lm75, SlaveAddr};
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::prelude::*;
use ssd1306::prelude::*;
use ssd1306::Builder;

use core::fmt::Write;
#[entry]
fn main() -> ! {
    let dp = stm32f103xx_hal::stm32f103xx::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
    let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);

    let i2c = BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Fast {
            frequency: 400_000,
            duty_cycle: DutyCycle::Ratio2to1,
        },
        clocks,
        &mut rcc.apb1,
        1000,
        10,
        1000,
        1000,
    );

    let manager = shared_bus::BusManager::<cortex_m::interrupt::Mutex<_>, _>::new(i2c);
    let mut disp: GraphicsMode<_> = Builder::new().connect_i2c(manager.acquire()).into();


    disp.init().unwrap();
    disp.flush().unwrap();

    let mut lm75 = Lm75::new(manager.acquire(), SlaveAddr::default());

    loop {
        let mut buffer: heapless::String<heapless::consts::U32> = heapless::String::new();
        let temp = lm75.read_temperature().unwrap();

        write!(buffer, "Temperature {}", temp).unwrap();

        disp.draw(
        Font6x8::render_str(&buffer)
            .with_stroke(Some(1u8.into()))
            .into_iter(),
        );
        disp.flush().unwrap();
    }
}
