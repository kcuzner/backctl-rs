//! Quick and simple backlight control using udev

extern crate udev;
extern crate clap;
#[macro_use]
extern crate error_chain;

use clap::App;

use std::{fs, io, num};
use std::io::{Write, Read};
use std::path::Path;

error_chain! {
    foreign_links {
        Udev(::udev::Error);
        Io(::io::Error);
        ParseInt(::num::ParseIntError);
    }
}

struct Backlight {
    max_brightness: u32,
    brightness: fs::File,
}

impl Backlight {
    fn new(path: &Path) -> Result<Self> {
        let mut mb_file = fs::File::open(path.join(Path::new("max_brightness")))?;
        let mut mb_buf = String::new();
        mb_file.read_to_string(&mut mb_buf)?;
        let mb: u32 = mb_buf.trim().parse()?;
        let b_file = fs::OpenOptions::new()
            .write(true)
            .open(path.join(Path::new("brightness")))?;
        Ok(Backlight { max_brightness: mb, brightness: b_file })
    }

    fn get_max_brightness(&self) -> u32 {
        self.max_brightness
    }

    fn set_brightness(mut self, brightness: u32) -> Result<()> {
        self.brightness.write_all(&brightness.to_string().into_bytes())?;
        Ok(())
    }
}

struct Backlights {
    iter: udev::Devices,
}

impl Backlights {
    fn new() -> Result<Self> {
        let context = udev::Context::new()?;
        let mut enumerator = udev::Enumerator::new(&context)?;
        enumerator.match_is_initialized()?;
        enumerator.match_subsystem("backlight")?;
        let devs = enumerator.scan_devices()?;
        Ok(Backlights { iter: devs })
    }
}

impl Iterator for Backlights {
    type Item = Result<Backlight>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(dev) => Some(Backlight::new(dev.syspath())),
            _ => None,
        }
    }
}

fn main() {
    let matches = App::new("Backlight Control")
        .author("Kevin Cuzner <kevin@kevincuzner.com>")
        .about("Sets the backlight brightness through sysfs")
        .get_matches();

    for res in Backlights::new().unwrap() {
        let bl = res.unwrap();
        println!("{}", bl.get_max_brightness());
    }
}
