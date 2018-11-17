//! Pi - Drive some LEDs from a Raspberry Pi
//! =============================================
//! 
//! Since the Raspberry Pi doens't have 25mA tolerant GPIO pins, using
//! a GPIO expander when a lot of lights and an interrupt are needed
//! makes sense. This example shows a counting binary value on GPIO
//! bank B (pins 1 through 8) with the LEDs tied to ground.
//! 
//! It also triggers an interrupt on GPIO bank A on pin 28.

extern crate linux_embedded_hal as linux_hal;
extern crate embedded_hal;
extern crate mcp23x17;

use linux_hal::I2cdev;
use mcp23x17::{
    Mcp23x17 as Expander,
    Port
};

use std::{
    thread,
    time,
    error::Error
};

fn main() -> Result<(), Box<Error>> {
    let i2c = I2cdev::new("/dev/i2c-1").unwrap();
    let sleep_time = time::Duration::from_millis(1000);
    let mut count = 0u8;
    let mut exp = Expander::new(i2c)?;

    // We'll have an interrupt when bit 8 of GPIO port A changes.
    exp.select_port(Port::A);
    exp.set_interrupt(0x80)?;
    exp.set_int_control(0x00)?;
    exp.set_direction(0xff)?;
    exp.set_pullups(0xff)?;

    // Prep Port B to show some pretty lights
    exp.select_port(Port::B);
    exp.set_direction(0x00)?;

    loop {
        exp.select_port(Port::B);
        exp.set_data(count)?;
        thread::sleep(sleep_time);

        exp.select_port(Port::A);
        println!("Interrupt pins: {:x?}", exp.who_interrupted()?);
        println!("Data Interrupt: {:x?}", exp.data_at_interrupt()?);

        count = (count + 1) % 255
    }
}
