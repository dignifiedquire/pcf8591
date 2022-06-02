use embedded_hal::blocking::i2c;

use crate::PCF8591;

pub struct AnalogOut<C: i2c::WriteRead> {
    pcf: PCF8591<C>,
    value: u16,
    _pin: Pin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pin {
    Default = 0,
}

impl<C: i2c::WriteRead> AnalogOut<C> {
    pub fn new(pcf: PCF8591<C>, pin: Pin) -> Result<Self, C::Error> {
        pcf.set_dac_enabled(true)?;
        Ok(AnalogOut {
            pcf,
            _pin: pin,
            value: 0,
        })
    }

    pub fn value(&self) -> u16 {
        self.value
    }

    pub fn set_value(&mut self, new_value: u16) -> Result<(), C::Error> {
        assert!(self.pcf.dac_enabled(), "Underlying DAC is disabled");

        // scale down to 8 bit
        let to_write = (new_value >> 8) as u8;
        self.pcf.write(to_write)?;
        self.value = new_value;

        Ok(())
    }
}

impl<C: i2c::WriteRead> Drop for AnalogOut<C> {
    fn drop(&mut self) {
        let _e = self.pcf.set_dac_enabled(false);
        // TODO: error handling
    }
}
