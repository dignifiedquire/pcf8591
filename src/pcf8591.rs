use std::sync::{Arc, Mutex};

use embedded_hal::blocking::i2c;

/// The default address for the PCF8591 (all address pins tied to ground)
pub const PCF8591_DEFAULT_ADDRESS: u8 = 0x48;

/// Control bit for having the DAC active
const PCF8591_ENABLE_DAC: u8 = 0x40;

/// The PCF8591 has four ADC channels, represented here
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum PCFADCNum {
    A0 = 0,
    A1 = 1,
    A2 = 2,
    A3 = 3,
}

#[derive(Debug)]
pub struct PCF8591<C: i2c::WriteRead> {
    inner: Arc<Mutex<Inner<C>>>,
}

impl<C: i2c::WriteRead> Clone for PCF8591<C> {
    fn clone(&self) -> Self {
        PCF8591 {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// A PCF8591 ADC using the underlying i2c channel
#[derive(Debug)]
struct Inner<C: i2c::WriteRead> {
    channel: C,
    config: Config,
    dac_enabled: bool,
    dac_val: u8,
    buffer: [u8; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// The I2C address to connect to.
    pub address: u8,
    /// The voltage level that ADC signals are compared to. An ADC value
    /// of `65,535` will equal `reference_voltage`.
    pub reference_voltage: f32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            address: PCF8591_DEFAULT_ADDRESS,
            reference_voltage: 3.3,
        }
    }
}

impl<C: i2c::WriteRead> PCF8591<C> {
    /// Instantiate the device using the given channel.
    pub fn new(channel: C) -> Self {
        Self::with_config(channel, Config::default())
    }

    /// Instantiate the device using the given channel and the given config.
    pub fn with_config(channel: C, config: Config) -> Self {
        // this is the range supported by the PCF8591
        assert!(
            2.5 <= config.reference_voltage && config.reference_voltage <= 6.0,
            "voltage out of range"
        );

        PCF8591 {
            inner: Arc::new(Mutex::new(Inner {
                channel,
                config,
                dac_enabled: false,
                dac_val: 0,
                buffer: [0u8; 2],
            })),
        }
    }

    /// Enables the DAC when `true`, or sets it to tri-state / high-Z when false.
    pub fn set_dac_enabled(&self, enabled: bool) -> Result<(), C::Error> {
        let inner = &mut self.inner.lock().unwrap();
        inner.dac_enabled = enabled;
        let val = inner.dac_val;
        inner.write(val)
    }

    pub fn dac_enabled(&self) -> bool {
        self.inner.lock().unwrap().dac_enabled
    }

    pub fn reference_voltage(&self) -> f32 {
        self.inner.lock().unwrap().config.reference_voltage
    }

    /// Read a single ADC value from a single port.
    pub fn read(&self, adc: PCFADCNum) -> Result<u8, C::Error> {
        self.inner.lock().unwrap().read(adc)
    }

    /// Writes a uint8_t value to the DAC output.
    ///
    /// The value to write: `0` is GND and `65,535` is VCC.
    pub fn write(&self, value: u8) -> Result<(), C::Error> {
        self.inner.lock().unwrap().write(value)
    }

    /// Consume the driver, returning the underlying channel
    pub fn into_inner(self) -> Result<C, Self> {
        Arc::try_unwrap(self.inner)
            .map(|inner| inner.into_inner().unwrap().channel)
            .map_err(|inner| PCF8591 { inner })
    }
}

impl<C: i2c::WriteRead> Inner<C> {
    fn clear_buffer(&mut self) {
        for el in &mut self.buffer {
            *el = 0;
        }
    }

    fn write(&mut self, value: u8) -> Result<(), C::Error> {
        let command = if self.dac_enabled {
            [PCF8591_ENABLE_DAC, value]
        } else {
            [0u8; 2]
        };

        self.dac_val = value;

        self.clear_buffer();
        self.channel
            .write_read(self.config.address, &command, &mut self.buffer)?;

        Ok(())
    }

    fn read(&mut self, adc: PCFADCNum) -> Result<u8, C::Error> {
        // first trigger the measurement
        self.half_read(adc)?;
        // then communicate again to get the actual result
        self.half_read(adc)
    }

    fn half_read(&mut self, adc: PCFADCNum) -> Result<u8, C::Error> {
        let mut command = if self.dac_enabled {
            [PCF8591_ENABLE_DAC, self.dac_val]
        } else {
            [0u8; 2]
        };

        command[0] |= (adc as u8) & 0x3;

        self.clear_buffer();
        self.channel
            .write_read(self.config.address, &command, &mut self.buffer[..])?;
        Ok(self.buffer[1])
    }
}
