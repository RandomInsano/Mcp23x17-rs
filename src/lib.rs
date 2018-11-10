#![no_std]

extern crate embedded_hal as hal;

// Taken from http://ww1.microchip.com/downloads/en/DeviceDoc/20001952C.pdf

/// IO Direction. 1 = input, Default 0xff
const REG_IODIR: u8 = 0x00;
/// Input polarity inversion. 1 = invert logic
const REG_IPOL: u8 = 0x01;
/// Interript on change. 1 = enabled
const REG_GPINTEN: u8 = 0x02;
/// Comparison for interrupts
const REG_DEFVAL: u8 = 0x03;
/// Interrupt on change configuration.
const REG_INTCON: u8 = 0x04;
/// Chip configuration
const REG_CONFIG: u8 = 0x05;
/// Internal 100KOhm pull-up resistors. 1 = enabled
const REG_GPPUA: u8 = 0x06;
/// Interrupt flag
const REG_INTF: u8 = 0x07;
/// Interrupt captured value
const INTCAP: u8 = 0x08;
/// General Purpose IO value. 1 = high
const REG_GPIO: u8 = 0x09;
/// Output latch. 1 = high
const REG_OLAT: u8 = 0x0A;

/// Device address
const ADDRESS: u8 = 0x20;


use hal::blocking::i2c::{
    Write,
    WriteRead
};

pub enum Port {
    A,
    B,
}

pub struct Mcp23x17<I2C> {
    i2c: I2C,
    active_port: Port,
}

impl<I2C, E> Mcp23x17<I2C>
where
    I2C: WriteRead<Error = E> + Write<Error = E>,
{
    pub fn new(i2c: I2C) -> Result<Self, E> {
        Ok(Self {
            i2c,
            active_port: Port::A,
        })
    }

    fn get_port(&self, register: u8) -> u8 {
        match &self.active_port {
            Port::A => register,
            Port::B => 0x10 | register
        }
    }

    /// This chip optionally splits its registers between two eight bit
    /// ports. This sets the port internally and returns a refrence to
    /// itself
    pub fn select_port(&mut self, port: Port) {
        self.active_port = port;
    }

    pub fn set_direction(&mut self, data: u8) -> Result<(), E> {
        let reg = self.get_port(REG_IODIR);
        self.i2c.write(ADDRESS, &[reg, data])?;

        Ok(())
    }

    pub fn set_data(&mut self, data: u8) -> Result<(), E> {
        let reg = self.get_port(REG_GPIO);
        self.i2c.write(ADDRESS, &[reg, data])?;

        Ok(())
    }

    pub fn data(&mut self) -> Result<u8, E> {
        let reg = self.get_port(REG_GPIO);
        let data: u8 = 0x00;
        self.i2c.write_read(ADDRESS, &[reg], &mut [data])?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate embedded_hal_mock as hal;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
