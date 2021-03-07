extern crate clap;

use dht_hal_drv::{dht_read, DhtType};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use rppal::gpio::{Gpio, IoPin, Mode};
use spin_sleep;
use std::{thread, time};
use void;

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

fn main() {
    let dht11_pin = 24_u8;
    let valve_pin = 2_u8;

    println!("DHT11 initialized at pin {}", dht11_pin);
    println!("Solenoid valve initialized at pin {}", valve_pin);

    let dht11_gpio = Gpio::new().expect("Can not init Gpio structure");
    let valve_gpio = Gpio::new().expect("Can not init Gpio structure");

    let dht11_iopin = dht11_gpio
        .get(dht11_pin)
        .expect("Was not able to get Pin for DHT11")
        .into_io(Mode::Input);
    let valve_iopin = valve_gpio
        .get(valve_pin)
        .expect("Was not able to get Pin for solenoid valve")
        .into_io(Mode::Output);

    let mut dht11_opin = OpenPin::new(dht11_iopin);
    let mut valve_opin = OpenPin::new(valve_iopin);

    loop {
        let readings = dht_read(DhtType::DHT11, &mut dht11_opin, &mut |d| {
            spin_sleep::sleep(time::Duration::from_micros(d as u64))
        });

        match readings {
            Ok(res) => {
                println!("DHT readins {}C {}%", res.temperature(), res.humidity());
            }
            Err(_) => (),
        };

        // Turn valve on/off
        valve_opin.set_high().unwrap();
        thread::sleep(time::Duration::from_secs(10));
        valve_opin.set_low().unwrap();

        thread::sleep(time::Duration::from_secs(2));
    }
}
