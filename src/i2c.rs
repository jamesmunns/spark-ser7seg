use crate::{Error, SevenSegInterface};
use embedded_hal::blocking::i2c::Write;

#[non_exhaustive]
pub enum I2cError<I2C> {
    I2c(I2C),
}

pub struct SevSegI2c<I2C> {
    i2c: I2C,
    addr: u8,
}

impl<I2C> SevSegI2c<I2C>
where
    I2C: Write,
{
    /// Create a new SparkFun Serial Seven Segment display using an I2C
    /// port. The I2C port supports 100kHz and 400kHz modes.
    ///
    /// If no address is supplied, the default 7-bit address of `0x71`
    /// will be used.
    pub fn new(i2c: I2C, addr: Option<u8>) -> Self {
        Self {
            i2c,
            addr: addr.unwrap_or(0x71),
        }
    }

    /// Update the address of the display used by the library.
    ///
    /// This does NOT reconfigure the display to use this new address.
    ///
    /// For now, this is probably not useful until we implement the
    /// "change address" command in the main interface.
    pub fn set_address(&mut self, addr: u8) {
        self.addr = addr;
    }

    /// Release the components
    pub fn release(self) -> I2C {
        self.i2c
    }
}

impl<I2C> SevenSegInterface for SevSegI2c<I2C>
where
    I2C: Write,
{
    type InterfaceError = I2cError<I2C::Error>;

    fn send(&mut self, data: &[u8]) -> Result<(), Error<Self::InterfaceError>> {
        self.i2c
            .write(self.addr, &data)
            .map_err(|e| Error::Interface(I2cError::I2c(e)))
            .map(drop)
    }
}
