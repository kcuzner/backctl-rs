//! Quick and simple backlight control using udev

extern crate udev;
extern crate clap;
#[macro_use]
extern crate error_chain;

use clap::{App, Arg, SubCommand};

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

struct Update {
    relative: bool,
    percent: bool,
    value: i32,
}

impl Update {
    fn set(valstr: &str) -> Result<Self> {
        Update::new(false, valstr)
    }
    fn inc(valstr: &str) -> Result<Self> {
        Update::new(true, valstr)
    }
    fn dec(valstr: &str) -> Result<Self> {
        let mut res = Update::new(true, valstr)?;
        res.value *= -1;
        Ok(res)
    }
    fn new(relative: bool, valstr: &str) -> Result<Self> {
        Ok(Update { relative: relative, percent: valstr.contains('%'),  value: valstr.trim().trim_end_matches('%').parse()? })
    }

    fn apply(&self, backlight: Backlight) -> Result<Backlight> {
        let max = backlight.get_max_brightness()? as i32;
        let mut value = self.value;

        // Step 1: Percent to brightness-units
        if self.percent {
            value = max * value / 100;
        }

        // Step 2: Relative to absolute
        if self.relative {
            let original = backlight.get_brightness()? as i32;
            value += original;
        }

        // Step 3: Clamp to min/max
        if value > max {
            value = max;
        }
        if value < 0 {
            value = 0;
        }

        backlight.set_brightness(value as u32)
            .and_then(|()| Ok(backlight))
    }
}

fn main() {
    let matches = App::new("Backlight Control")
        .author("Kevin Cuzner <kevin@kevincuzner.com>")
        .about("Sets the backlight brightness through sysfs")
        .arg(Arg::with_name("CMD")
             .required(true)
             .takes_value(true)
             .possible_value("inc")
             .possible_value("dec")
             .possible_value("set"))
        .arg(Arg::with_name("VALUE")
             .required(true))
        .get_matches();

    let cmdstr = matches.value_of("CMD").expect("No command supplied");
    let valstr = matches.value_of("VALUE").expect("No value supplied");

    let update = match cmdstr.as_ref() {
        "inc" => Update::inc(&valstr).expect("Unable to create increment update"),
        "dec" => Update::dec(&valstr).expect("Unable to create decrement update"),
        "set" => Update::set(&valstr).expect("Unable to create set update"),
        _ => panic!("Invalid command supplied"),
    };

    for bl in Backlights::new().unwrap() {
        update.apply(bl).unwrap();
    }
}
