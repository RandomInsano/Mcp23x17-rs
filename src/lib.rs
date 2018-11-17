//! Rust Library for the Microchip MCP23X17
//! ========================================
//! In its current incarnation, this only supports I2C but the register
//! map is the same for SPI as well.
//! 
//! Internally, the chip supports a segreggated layout of registers to make
//! two 8 bit GPIO ports or can interleave the registers to emulate one
//! 16 bit GPIO port. This library works on the former layout and so disables
//! setting `BANK` when calling `set_config()`.
//! 
//! ```
//! use linux_hal::I2cdev;
//! use mcp23x17::{
//!     Mcp23x17 as Expander,  
//!     Port
//! };
//! 
//! fn main() -> Result<(), Box<Error>> {
//!     let i2c = I2cdev::new("/dev/i2c-1")?;
//!     let mut exp = Expander::new(i2c)?;
//! 
//!     exp.select_port(Port::B);
//!     exp.set_direction(0x00)?;
//!     exp.set_data(0xff)?;
//! }
//! ```
//! 
//! Implementation details taken from
//! http://ww1.microchip.com/downloads/en/DeviceDoc/20001952C.pdf

#![no_std]
#![deny(missing_docs)]
//#![deny(warnings)]

// TODO: Learn how to use macros! Silly amounts of code duplication here

extern crate embedded_hal as hal;
extern crate bitflags;

use hal::blocking::i2c::{
    Write,
    WriteRead
};
use bitflags::bitflags;

/// IO Direction. 1 = input, Default 0xff
const REG_IODIR: u8 = 0x00;
/// Input polarity inversion. 1 = invert logic
const REG_IPOL: u8 = 0x01;
/// interrupt on change. 1 = enabled
const REG_GPINTEN: u8 = 0x02;
/// Comparison for interrupts
const REG_DEFVAL: u8 = 0x03;
/// Interrupt on change configuration.
const REG_INTCON: u8 = 0x04;
/// Chip configuration
const REG_CONFIG: u8 = 0x05;
/// Internal 100KOhm pull-up resistors. 1 = enabled
const REG_GPPU: u8 = 0x06;
/// Interrupt flag
const REG_INTF: u8 = 0x07;
/// Interrupt captured value
const REG_INTCAP: u8 = 0x08;
/// General Purpose IO value. 1 = high
const REG_GPIO: u8 = 0x09;
/// Output latch. 1 = high
const REG_OLAT: u8 = 0x0A;

/// Device address
pub const ADDRESS: u8 = 0x20;


/// Which port we're actively using. Currently you must select which is
/// active by using select_port on `mcp23x17`. Port A is the default.
pub enum Port {
    /// Port A
    A,
    /// Port B
    B,
}

bitflags! {
    /// Configuration register definition. This register is mirrored
    /// in the register map and pertains to all ports
    pub struct Config: u8 {
        /// If true, interleave port A/B register locations
        const BANK = 1 << 7;
        /// If true, connect interrupt pins
        const MIRROR = 1 << 6;
        /// If true, automatically increment address pointer
        const SEQOP = 1 << 5;
        /// If true, enable slew rate
        const DISSLW = 1 << 4;
        /// If true, use address pins (MCP23S17 only)
        const HAEN = 1 << 3;
        /// If true, output is open-drain 
        const ODR = 1 << 2;
        /// If true, interrupt pins are active high
        const INTPOL = 1 << 1;
        /// Just stops me from looking dumb! This bit is "unimplemented" :D
        const _nothin = 1 << 0;
    }
}

/// 16bit GPIO Expander
pub struct Mcp23x17<I2C> {
    i2c: I2C,
    active_port: Port,
}

impl<I2C, E> Mcp23x17<I2C>
where
    I2C: WriteRead<Error = E> + Write<Error = E>,
{
    /// Create a new instance of the GPIO expander
    pub fn new(i2c: I2C) -> Result<Self, E> {
        Ok(Self {
            i2c,
            active_port: Port::A,
        })
    }

    /// Some quick math for the current register
    fn get_port(&self, register: u8) -> u8 {
        match &self.active_port {
            Port::A => register,
            Port::B => 0x10 | register
        }
    }

    /// Helper function to save my typing when setting
    fn set_thing(&mut self, register: u8, data: u8) -> Result<(), E> {
        let reg = self.get_port(register);

        Ok(self.i2c.write(ADDRESS, &[reg, data])?)
    }

    /// Helper function to save my typing when reading
    fn get_thing(&mut self, register: u8) -> Result<u8, E> {
        let reg = self.get_port(register);
        let mut data = [0u8; 1];

        self.i2c.write_read(ADDRESS, &[reg], &mut data)?;
        Ok(data[0])
    }

    /// This chip optionally splits its registers between two eight bit ports
    /// or virtuall one large 16 bit port. This function sets the port
    /// internally.
    pub fn select_port(&mut self, port: Port) {
        self.active_port = port;
    }

    /// Set the I/O direction for the currently active port. A value
    /// of 1 is for input, 0 for output
    pub fn set_direction(&mut self, data: u8) -> Result<(), E> {
        Ok(self.set_thing(REG_IODIR, data)?)
    }

    /// Get the I/O direction for the active port
    pub fn direction(&mut self) -> Result<u8, E> {
        Ok(self.get_thing(REG_IODIR)?)
    }

    /// Set configuration register. Given the structure of this library and how
    /// the chip can rearrange its registers, any attempt to set the `BANK` bit
    /// will be masked to zero.
    pub fn set_config(&mut self, data: Config) -> Result<(), E> {
        // Safety mechanism to avoid breaking the calls made in the library
        let data = data.bits & !Config::BANK.bits;

        Ok(self.set_thing(REG_CONFIG, data)?)
    }

    /// Read the data state from the active port
    pub fn config(&mut self) -> Result<u8, E> {
        Ok(self.get_thing(REG_CONFIG)?)
    }

    /// Set the pullups. A value of 1 enables the 100KOhm pullup.
    pub fn set_pullups(&mut self, data: u8) -> Result<(), E> {
        Ok(self.set_thing(REG_GPPU, data)?)
    }

    /// Get the pullups.
    pub fn pullups(&mut self) -> Result<u8, E> {
        Ok(self.get_thing(REG_GPPU)?)
    }

    /// Read interrupt state. Each pin that caused an interrupt will have
    /// a bit is set. Not settable.
    /// 
    /// The value will be reset after a read from `data_at_interrupt` or
    /// `data()`.
    pub fn who_interrupted(&mut self) -> Result<u8, E> {
        Ok(self.get_thing(REG_INTF)?)
    }

    /// GPIO value at time of interrupt. It will remain latched to this value
    /// until another interrupt is fired. While it won't reset on read, it does
    /// reset the interrupt state on the corresponding interrupt output pin
    pub fn data_at_interrupt(&mut self) -> Result<u8, E> {
        Ok(self.get_thing(REG_INTCAP)?)
    }

    /// Set a comparison value for the interrupts. The interrupt will
    /// fire if the input value is *different* from what is set here
    pub fn set_int_compare(&mut self, data: u8) -> Result<(), E> {
        Ok(self.set_thing(REG_DEFVAL, data)?)
    }

    /// Read interrupt comparison value. Check `set_int_compare()` for more
    /// details
    pub fn int_compare(&mut self) -> Result<u8, E> {
        Ok(self.get_thing(REG_DEFVAL)?)
    }

    /// Decide how interrupts will fire. If a bit is set, the input data
    /// is compared against what's set by `int_compare()`. If unset, the
    /// interrupt will fire when the pin has changed.
    pub fn set_int_control(&mut self, data: u8) -> Result<(), E> {
        Ok(self.set_thing(REG_INTCON, data)?)
    }

    /// Read how interrupts will fire. More details on `set_int_control()`.
    pub fn int_control(&mut self) -> Result<u8, E> {
        Ok(self.get_thing(REG_INTCON)?)
    }

    /// Enable interrupts. If a bit is set, a change on this pin will trigger an
    /// interrupt. You'll also need to call `set_int_compare()` and
    /// `set_int_control()`
    pub fn set_interrupt(&mut self, data: u8) -> Result<(), E> {
        Ok(self.set_thing(REG_GPINTEN, data)?)
    }

    /// Read the data state from the active port. See `set_interrupt()` for
    /// more details
    pub fn interrupt(&mut self) -> Result<u8, E> {
        Ok(self.get_thing(REG_GPINTEN)?)
    }

    /// Read output latches. This essentially reads the values set from
    /// calling `set_data()`
    pub fn latches(&mut self) -> Result<u8, E> {
        Ok(self.get_thing(REG_OLAT)?)
    }

    /// Set polarity allows inverting the values from input pins. A
    /// value of 1 will flip the polarity.
    pub fn set_polarity(&mut self, data: u8) -> Result<(), E> {
        Ok(self.set_thing(REG_IPOL, data)?)
    }

    /// Read the data state from the active port
    pub fn polarity(&mut self) -> Result<u8, E> {
        Ok(self.get_thing(REG_IPOL)?)
    }

    /// Set the data for the active port
    pub fn set_data(&mut self, data: u8) -> Result<(), E> {
        Ok(self.set_thing(REG_GPIO, data)?)
    }

    /// Read the data state from the active port
    pub fn data(&mut self) -> Result<u8, E> {
        Ok(self.get_thing(REG_GPIO)?)
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
