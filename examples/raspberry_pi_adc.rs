use std::thread::sleep;
use std::time::Duration;

use linux_embedded_hal::I2cdev;
use pcf8591_hal::*;

// # AnalogOut & AnalogIn Example
//
// This example shows how to use the included AnalogIn and AnalogOut
// classes to set the internal DAC to output a voltage and then measure
// it with the first ADC channel.
//
// # Wiring:
// Connect the DAC output to the first ADC channel, in addition to the
// normal power and I2C connections.

pub fn main() {
    let i2c = I2cdev::new("/dev/i2c-1").expect("can open i2c device");
    let pcf = PCF8591::new(i2c);

    let pcf_in_0 = AnalogIn::new(pcf.clone(), PCFADCNum::A0);
    let mut pcf_out =
        AnalogOut::new(pcf.clone(), OutPin::Default).expect("failed to create writer");

    loop {
        println!("Setting out to 65,535");
        pcf_out.set_value(65_535).expect("failed to write");
        let raw_value = pcf_in_0.value().expect("failed to read");
        let scaled_value = scale(raw_value, pcf_in_0.reference_voltage());

        println!("Pin 0: {:.02}V", scaled_value);
        sleep(Duration::from_secs(1));

        println!("Setting out to 32767");
        pcf_out.set_value(32_767).expect("failed to write");
        let raw_value = pcf_in_0.value().expect("failed to read");
        let scaled_value = scale(raw_value, pcf_in_0.reference_voltage());

        println!("Pin 0: {:.02}V", scaled_value);
        sleep(Duration::from_secs(1));

        println!("Setting out to 0");
        pcf_out.set_value(0).expect("failed to write");
        let raw_value = pcf_in_0.value().expect("failed to read");
        let scaled_value = scale(raw_value, pcf_in_0.reference_voltage());

        println!("Pin 0: {:.02}V", scaled_value);
        sleep(Duration::from_secs(1));
    }
}

fn scale(raw: u16, voltage: f32) -> f32 {
    (raw as f32 / 65_535.) * voltage
}
