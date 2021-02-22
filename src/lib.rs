//! Driver for the [SparkFun Serial 7 Segment Display](https://github.com/sparkfun/Serial7SegmentDisplay/wiki/Serial-7-Segment-Display-Datasheet)
//!
//! This is compatible with `embedded-hal`.
//!
//! Right now, only the SPI or I2C interfaces are supported. In the future,
//! support will be added for UART interfaces

#![no_std]

use bitflags::bitflags;
pub mod i2c;
pub mod spi;

bitflags! {
    /// A bit packed structure representing days of the week
    pub struct PunctuationFlags: u8 {
        const DOT_BETWEEN_1_AND_2        = 0b0000_0001;
        const DOT_BETWEEN_2_AND_3        = 0b0000_0010;
        const DOT_BETWEEN_3_AND_4        = 0b0000_0100;
        const DOT_RIGHT_OF_4             = 0b0000_1000;
        const DOTS_COLON                 = 0b0001_0000;
        const APOSTROPHE_BETWEEN_3_AND_4 = 0b0010_0000;
        const NONE                       = 0b0000_0000;
    }
}

mod command {
    #![allow(dead_code)]

    pub(crate) const CLEAR_DISPLAY: u8 = 0x76;

    pub(crate) const DECIMAL_CTL: u8 = 0x77;
    pub(crate) const CURSOR_CTL: u8 = 0x79;
    pub(crate) const BRIGHTNESS_CTL: u8 = 0x7A;

    pub(crate) const DIGIT_1_CTL: u8 = 0x7B;
    pub(crate) const DIGIT_2_CTL: u8 = 0x7C;
    pub(crate) const DIGIT_3_CTL: u8 = 0x7D;
    pub(crate) const DIGIT_4_CTL: u8 = 0x7E;

    pub(crate) const BAUD_RATE_CFG: u8 = 0x7F;
    pub(crate) const I2C_ADDR_CFG: u8 = 0x80;

    pub(crate) const FACTORY_RESET: u8 = 0x81;
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error<I> {
    Interface(I),
    CursorOutOfRange,
    DigitOutOfRange,
}

pub trait SevenSegInterface {
    /// A single error type used by the interface
    type InterfaceError;

    /// Sending commands to the interface
    fn send(&mut self, data: &[u8]) -> Result<(), Error<Self::InterfaceError>>;

    /// Set the digit cursor to a particular location
    /// `col` may be 0..=3, from left to right.
    fn set_cursor(&mut self, col: u8) -> Result<(), Error<Self::InterfaceError>> {
        if col >= 4 {
            return Err(Error::CursorOutOfRange);
        }

        self.send(&[command::CURSOR_CTL, col])
    }

    /// Set the brightness for the display. The datasheet says that 100 is the
    /// brightest, however my device gets brighter with values above 100 (up to 255).
    /// Your mileage may vary.
    fn set_brightness(&mut self, bright: u8) -> Result<(), Error<Self::InterfaceError>> {
        self.send(&[command::BRIGHTNESS_CTL, bright])
    }

    /// Completely clear the display
    fn clear(&mut self) -> Result<(), Error<Self::InterfaceError>> {
        self.send(&[command::CLEAR_DISPLAY])
    }

    /// Write a digit to the curent cursor position. This also
    /// increments the cursor position
    fn write_digit(&mut self, digit: u8) -> Result<(), Error<Self::InterfaceError>> {
        if digit > 0x0F {
            return Err(Error::DigitOutOfRange);
        }

        self.send(&[digit])
    }

    /// Write the requested punctuation to the display. This does not take
    /// the current state into account, so any unset flags in `punct_flags`
    /// will turn the corresponding LEDs off.
    fn write_punctuation(
        &mut self,
        punct_flags: PunctuationFlags,
    ) -> Result<(), Error<Self::InterfaceError>> {
        self.send(&[command::DECIMAL_CTL, punct_flags.bits()])
    }

    /// Write the requested digits to the display, starting at the current
    /// cursor position. Each digit must be in the range 0x0..=0xF, and up
    /// to 4 digits may be updated at once. The cursor is incremented after
    /// each digit
    fn write_digits(&mut self, digits: &[u8]) -> Result<(), Error<Self::InterfaceError>> {
        // Too many digits?
        if digits.len() > 4 {
            return Err(Error::CursorOutOfRange);
        }

        // Any digit too big?
        for d in digits {
            if *d > 0x0F {
                return Err(Error::DigitOutOfRange);
            }
        }

        self.send(digits)
    }

    /// Write the number to the display. The number will be left-filled
    /// with zeroes if necessary. After this function, the cursor
    /// will be at position 0.
    fn set_num(&mut self, num: u16) -> Result<(), Error<Self::InterfaceError>> {
        if num > 9999 {
            return Err(Error::DigitOutOfRange);
        }

        self.set_cursor(0)?;

        // TODO: We seem to need roughly 15uS between
        // back-to-back commands. How should I handle this?
        // Failure to do so can cause a potential NACK.

        let data: [u8; 4] = [
            (num / 1000) as u8,
            ((num % 1000) / 100) as u8,
            ((num % 100) / 10) as u8,
            (num % 10) as u8,
        ];

        self.send(&data)
    }
}
