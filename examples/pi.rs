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
    let sleep_time = time::Duration::from_secs(1);
    let mut on: bool = true;

    let mut exp = Expander::new(i2c)?;
    exp.select_port(Port::B);
    exp.set_direction(0x00)?;

    loop {
        exp.set_data(if on { 0xff} else { 0x00 })?;
        thread::sleep(sleep_time);

        on ^= true;
    }

    Ok(())
}
