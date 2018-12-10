//! Quick and simple backlight control using udev

extern crate udev;
extern crate clap;
#[macro_use]
extern crate error_chain;

use clap::App;

use std::{fs, io, num};
use std::io::{Write, Read};
use std::path::{Path, PathBuf};

error_chain! {
    foreign_links {
        Udev(::udev::Error);
        Io(::io::Error);
        ParseInt(::num::ParseIntError);
    }
}

struct Backlight {
    root: PathBuf,
}

impl Backlight {
    fn new(path: &Path) -> Self {
        Backlight { root: PathBuf::from(path) }
    }

    fn read_value(&self, property: &Path) -> Result<u32> {
        let mut f = fs::File::open(self.root.as_path().join(property))?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)?;
        Ok(buf.trim().parse()?)
    }

    fn get_max_brightness(&self) -> Result<u32> {
        self.read_value(Path::new("max_brightness"))
    }

    fn get_brightness(&self) -> Result<u32> {
        self.read_value(Path::new("brightness"))
    }

    fn set_brightness(&self, brightness: u32) -> Result<()> {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .open(self.root.as_path().join("brightness"))?;
        f.write_all(&brightness.to_string().into_bytes())?;
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
    type Item = Backlight;

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

    for bl in Backlights::new().unwrap() {
        println!("{}", bl.get_max_brightness().unwrap());
    }
}
