extern crate spidev;
extern crate failure;

use failure::Error;
use std::io::prelude::*;
use spidev::{Spidev, SpidevOptions, SPI_MODE_0};

///
///
/// Doc for UnicornHatHd
pub struct UnicornHatHd {
  leds: [[UnicornHatHdLed; 16]; 16],
  spi: Spidev,
}

impl UnicornHatHd {
  fn new(spi_path: &str) -> Result<UnicornHatHd, Error> {
    let mut spidev = try!(Spidev::open(spi_path));
    let options = SpidevOptions::new()
         .bits_per_word(8)
         .max_speed_hz(9_000_000)
         .mode(SPI_MODE_0)
         .build();
    try!(spidev.configure(&options));
    Ok(UnicornHatHd {
      leds: [[UnicornHatHdLed::default(); 16]; 16],
      spi: spidev,
    })
  }

  pub fn display(&mut self) -> Result<(), Error> {
    self.spi.write(&[0x72])?;
    let data = self.as_array();
    self.spi.write(&data)?;
    Ok(())
  }

  pub fn set_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
    self.leds[x][y].set_rgb(r, g, b);
  }

  pub fn get_pixel(&self, x: usize, y: usize) -> (u8, u8, u8) {
    self.leds[x][y].get_rgb()
  }

  fn as_array(&self) -> Vec<u8> {
    let mut arr: Vec<u8> = vec![];

    for row in self.leds.iter() {
      for led in row.iter() {
        let (r, g, b) = led.get_rgb();
        arr.push(r);
        arr.push(g);
        arr.push(b);
      }
    }

    arr
  }
}

impl Default for UnicornHatHd {
  fn default() -> UnicornHatHd {
    UnicornHatHd::new("/dev/spidev0.0").unwrap()
  }
}

#[derive(Clone,Copy)]
struct UnicornHatHdLed {
  r: u8,
  b: u8,
  g: u8,
}

impl UnicornHatHdLed {
  pub fn set_rgb(&mut self, r: u8, g: u8, b: u8) {
    self.r = r;
    self.g = g;
    self.b = b;
  }

  pub fn get_rgb(&self) -> (u8, u8, u8) {
    (self.r, self.g, self.b)
  }
}

impl Default for UnicornHatHdLed {
  fn default() -> UnicornHatHdLed {
    UnicornHatHdLed {
      r: 0,
      g: 0,
      b: 0,
    }
  }
}
