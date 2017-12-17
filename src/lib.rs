extern crate spidev;
extern crate failure;

use failure::Error;
use std::io::prelude::*;
use spidev::{Spidev, SpidevOptions, SPI_MODE_0};

/// Possible rotations of the buffer before displaying to the
/// Unicorn HAT HD.
pub enum Rotate {
  /// Default rotation.
  RotNone,
  /// Rotate the output by 90 degrees clockwise.
  RotCW90,
  /// Rotate the output by 90 degrees counter-clockwise.
  RotCCW90,
  /// Rotate the output by 180 degrees.
  Rot180,
}

/// Provide high-level access to the Unicorn HAT HD.
pub struct UnicornHatHd {
  leds: [UnicornHatHdLed; 256],
  spi: Spidev,
  rotation: Rotate,
}

impl UnicornHatHd {
  /// Create a new `UnicornHatHd` with the provided path
  ///
  /// The Unicorn HAT HD should be addressable using the spidev
  /// device with the provided path
  ///
  /// Typically, the path will be something like `"/dev/spidev0.0"`
  /// where the first number if the bus and the second number
  /// is the chip select on that bus for the device being targeted.
  pub fn new(spi_path: &str) -> Result<UnicornHatHd, Error> {
    let mut spidev = try!(Spidev::open(spi_path));
    let options = SpidevOptions::new()
         .bits_per_word(8)
         .max_speed_hz(9_000_000)
         .mode(SPI_MODE_0)
         .build();
    try!(spidev.configure(&options));
    Ok(UnicornHatHd {
      leds: [UnicornHatHdLed::default(); 256],
      spi: spidev,
      rotation: Rotate::RotNone,
    })
  }

  /// Rotate the display buffer by [`Rotate`](enum.Rotate.html) degrees
  /// before sending to the Unicorn HAT HD.
  ///
  /// This allows for different mounting orientations of the Unicorn HAT HD
  /// without having to translate the `(x, y)` of each pixel to account for the
  /// physical rotation of the display.
  pub fn set_rotation(&mut self, rot: Rotate) {
    self.rotation = rot;
  }

  /// Write the display buffer to the Unicorn HAT HD.
  pub fn display(&mut self) -> Result<(), Error> {
    self.spi.write(&[0x72])?;
    let data = self.as_array();
    self.spi.write(&data)?;
    Ok(())
  }

  /// Set an individual pixel's RGB value.
  ///
  /// The origin (`(0, 0)`) is the top-left of the display, with `x` & `y`
  /// increasing to the right, and down, respectively.
  pub fn set_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
    self.leds[(y * 16) + x].set_rgb(r, g, b);
  }

  /// Return a tuple of an individual pixel's RGB value.
  ///
  /// The origin (`(0, 0)`) is the top-left of the display, with `x` & `y`
  /// increasing to the right, and down, respectively.
  ///
  /// *NOTE*: This returns what's in the display buffer, not what the
  /// physical pixel is set to.
  pub fn get_pixel(&self, x: usize, y: usize) -> (u8, u8, u8) {
    self.leds[(y * 16) + x].get_rgb()
  }

  /// Clear the internal buffer of pixel states.
  ///
  /// To clear the display itself, you'll still need to call
  /// [`display`](#method.display) to update the Unicorn HAT HD.
  pub fn clear_pixels(&mut self) {
    self.leds = [UnicornHatHdLed::default(); 256];
  }

  /// Translate the internal buffer into a `Vec<u8>` of RGB values. The LEDs on
  /// the Unicorn HAT HD are addressed in the following order, with each LED
  /// consisting of three `u8`, one each for the R, G, and B values (assuming no
  /// rotation has been set):
  ///
  /// Physical LEDs => Vec<u8> order
  ///     1 2 3
  ///     4 5 6     => 1 2 3 4 5 6 7 8 9
  ///     7 8 9
  fn as_array(&self) -> Vec<u8> {
    let mut arr: Vec<u8> = vec![];

    match self.rotation {
      // 1 2 3    1 2 3
      // 4 5 6 => 4 5 6 => 1 2 3 4 5 6 7 8 9
      // 7 8 9    7 8 9
      Rotate::RotNone => {
        for led in self.leds.iter() {
          let (r, g, b) = led.get_rgb();
          arr.push(r);
          arr.push(g);
          arr.push(b);
        }
      },
      // 1 2 3    7 4 1
      // 4 5 6 => 8 5 2 => 7 4 1 8 5 2 9 6 3
      // 7 8 9    9 6 3
      Rotate::RotCW90 => {
        for x in 0..16 {
          for y in (0..16).rev() {
            let (r, g, b) = self.get_pixel(x, y);
            arr.push(r);
            arr.push(g);
            arr.push(b);
          }
        }
      },
      // 1 2 3    3 6 9
      // 4 5 6 => 2 5 8 => 3 6 9 2 5 8 1 4 7
      // 7 8 9    1 4 7
      Rotate::RotCCW90 => {
        for x in (0..16).rev() {
          for y in 0..16 {
            let (r, g, b) = self.get_pixel(x, y);
            arr.push(r);
            arr.push(g);
            arr.push(b);
          }
        }
      },
      // 1 2 3    9 8 7
      // 4 5 6 => 6 5 4 => 9 8 7 6 5 4 3 2 1
      // 7 8 9    3 2 1
      Rotate::Rot180 => {
        for led in self.leds.iter().rev() {
          let (r, g, b) = led.get_rgb();
          arr.push(r);
          arr.push(g);
          arr.push(b);
        }
      },
    }

    arr
  }
}

impl Default for UnicornHatHd {
  /// Create a `UnicornHatHd` using the default path of "`/dev/spidev0.0`".
  ///
  /// This will panic if the default path is not usable.
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
