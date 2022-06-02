use embedded_hal::blocking::i2c;

use crate::{PCFADCNum, PCF8591};

pub struct AnalogIn<C: i2c::WriteRead> {
    pcf: PCF8591<C>,
    channel: PCFADCNum,
}

impl<C: i2c::WriteRead> AnalogIn<C> {
    pub fn new(pcf: PCF8591<C>, channel: PCFADCNum) -> Self {
        AnalogIn { pcf, channel }
    }

    pub fn voltage(&self) -> Result<f32, C::Error> {
        let raw_reading = self.value()?;

        let voltage = (raw_reading as f32 / 65_535.) * self.pcf.reference_voltage();
        Ok(voltage)
    }

    pub fn value(&self) -> Result<u16, C::Error> {
        let raw = self.pcf.read(self.channel)? as u16;
        Ok(raw << 8)
    }

    pub fn reference_voltage(&self) -> f32 {
        self.pcf.reference_voltage()
    }
}
