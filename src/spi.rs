use crate::{Error, SevenSegInterface};
use embedded_hal::{blocking::spi::Write, digital::v2::OutputPin};

#[non_exhaustive]
pub enum SpimError<SPIM, GPIO> {
    Spim(SPIM),
    Gpio(GPIO),
}

pub struct SevSegSpim<SPIM, CS> {
    spim: SPIM,
    csn: CS,
}

impl<SPIM, CS> SevSegSpim<SPIM, CS>
where
    SPIM: Write<u8>,
    CS: OutputPin,
{
    /// Create a new SparkFun Serial Seven Segment display using a SPI (Master)
    /// port. The SPI port has a maximum frequency of 250kHz, and must be in Mode 0.
    pub fn new(spim: SPIM, csn: CS) -> Self {
        Self { spim, csn }
    }

    /// Release the components
    pub fn release(self) -> (SPIM, CS) {
        (self.spim, self.csn)
    }
}

impl<SPIM, CS> SevenSegInterface for SevSegSpim<SPIM, CS>
where
    SPIM: Write<u8>,
    CS: OutputPin,
{
    type InterfaceError = SpimError<SPIM::Error, CS::Error>;

    fn send(&mut self, data: &[u8]) -> Result<(), Error<Self::InterfaceError>> {
        self.csn
            .set_low()
            .map_err(|e| Error::Interface(SpimError::Gpio(e)))?;

        let ret = self
            .spim
            .write(&data)
            .map_err(|e| Error::Interface(SpimError::Spim(e)))
            .map(drop);

        self.csn
            .set_high()
            .map_err(|e| Error::Interface(SpimError::Gpio(e)))?;

        ret
    }
}
