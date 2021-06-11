extern crate clap;

use dht_hal_drv::{dht_read, DhtType};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use rppal::gpio::{Gpio, IoPin, Mode};
use spin_sleep;
use std::{thread, time};
use void;
use std::env;
use std::fs::File;
use std::io::Write;

// We're using a modified version of the example from
// https://github.com/rustrum/dht-hal-drv/blob/master/examples/rpi-rppal/src/main.rs

struct OpenPin {
    iopin: IoPin,
    mode: Mode,
}

impl OpenPin {
    fn new(mut pin: IoPin) -> OpenPin {
        pin.set_mode(Mode::Input);
        OpenPin {
            iopin: pin,
            mode: Mode::Input,
        }
    }

    fn switch_input(&mut self) {
        if self.mode != Mode::Input {
            self.mode = Mode::Input;
            self.iopin.set_mode(Mode::Input);
        }
    }

    fn switch_output(&mut self) {
        if self.mode != Mode::Output {
            self.mode = Mode::Output;
            self.iopin.set_mode(Mode::Output);
        }
    }
}

impl InputPin for OpenPin {
    type Error = void::Void;

    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.iopin.is_high())
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.iopin.is_low())
    }
}

impl OutputPin for OpenPin {
    type Error = void::Void;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.switch_output();
        self.iopin.set_low();
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.iopin.set_high();
        self.switch_input();
        Ok(())
    }
}

/// Log results from a reading and whether
/// the plant was watered or not.
fn log(watered: bool, temp: f32, humi: f32) {
	let log_msg: String;
    match watered {
        true => log_msg = format!("Last reading: \ntemperature: {}\nhumidity: {}\n Watered plant: yes", temp, humi),
        false => log_msg = format!("Last reading: \ntemperature: {}\nhumidity: {}\n Watered plant: no", temp, humi),
    }

    // Create a temporary file.
    let temp_directory = env::temp_dir();
    let temp_file = temp_directory.join("/tmp/awa_data.txt");

    // Open a file in write-only (ignoring errors).
    // This creates the file if it does not exist (and empty the file if it exists).
    let mut file = File::create(temp_file).unwrap();

    // Write a &str in the file (ignoring the result).
    writeln!(&mut file, "{}", log_msg).unwrap();

}

fn main() {
    let dht11_pin = 24_u8;
    let valve_pin = 2_u8;

    println!("DHT11 sensor was initialized at pin {}", dht11_pin);
    println!("Solenoid valve was initialized at pin {}", valve_pin);

    let dht11_gpio = Gpio::new().expect("Can not init Gpio structure for the DHT11 sesnsor");
    let valve_gpio = Gpio::new().expect("Can not init Gpio structure for the solenoid valve");

    let dht11_iopin = dht11_gpio
        .get(dht11_pin)
        .expect("Was not able to get Pin for the DHT11 sensor")
        .into_io(Mode::Input);
    let valve_iopin = valve_gpio
        .get(valve_pin)
        .expect("Was not able to get Pin for the solenoid valve")
        .into_io(Mode::Output);

    let mut dht11_opin = OpenPin::new(dht11_iopin);
    let mut valve_opin = OpenPin::new(valve_iopin);

    loop {
        let readings = dht_read(DhtType::DHT11, &mut dht11_opin, &mut |d| {
            spin_sleep::sleep(time::Duration::from_micros(d as u64))
        });

        match readings {
            Ok(res) => {
                let mut watered: bool = false;
                println!("DHT readings {}C {}%", res.temperature(), res.humidity());
                if res.temperature() > 15.0 && res.humidity() < 85.0 {
                    watered = true;
                    // Turn valve on/off
                    println!("Watering plant...");
                    valve_opin.set_low().unwrap();
                    thread::sleep(time::Duration::from_secs(1));
                    valve_opin.set_high().unwrap();
                }
				log(watered, res.temperature(), res.humidity());
            }
            Err(_) => (),
        };

        thread::sleep(time::Duration::from_secs(1800));
    }
}
