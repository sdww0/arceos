use axlog::ax_println;
use core::fmt;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

use bitflags::bitflags;

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub(crate) unsafe fn pause() {
    core::arch::x86_64::_mm_pause();
}

#[derive(Debug)]
pub enum Error {
    CommandRetry,
    NoMoreTries,
    ReadTimeout,
    WriteTimeout,
}

bitflags! {
    #[derive(Debug)]
    pub struct StatusFlags: u8 {
        const OUTPUT_FULL = 1;
        const INPUT_FULL = 1 << 1;
        const SYSTEM = 1 << 2;
        const COMMAND = 1 << 3;
        // Chipset specific
        const KEYBOARD_LOCK = 1 << 4;
        // Chipset specific
        const SECOND_OUTPUT_FULL = 1 << 5;
        const TIME_OUT = 1 << 6;
        const PARITY = 1 << 7;
    }
}

bitflags! {
    #[derive(Debug,Clone, Copy)]
    pub struct ConfigFlags: u8 {
        const FIRST_INTERRUPT = 1;
        const SECOND_INTERRUPT = 1 << 1;
        const POST_PASSED = 1 << 2;
        // 1 << 3 should be zero
        const CONFIG_RESERVED_3 = 1 << 3;
        const FIRST_DISABLED = 1 << 4;
        const SECOND_DISABLED = 1 << 5;
        const FIRST_TRANSLATE = 1 << 6;
        // 1 << 7 should be zero
        const CONFIG_RESERVED_7 = 1 << 7;
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
#[allow(dead_code)]
enum Command {
    ReadConfig = 0x20,
    WriteConfig = 0x60,
    DisableSecond = 0xA7,
    EnableSecond = 0xA8,
    TestSecond = 0xA9,
    TestController = 0xAA,
    TestFirst = 0xAB,
    Diagnostic = 0xAC,
    DisableFirst = 0xAD,
    EnableFirst = 0xAE,
    WriteSecond = 0xD4,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
#[allow(dead_code)]
enum KeyboardCommand {
    EnableReporting = 0xF4,
    SetDefaultsDisable = 0xF5,
    SetDefaults = 0xF6,
    Reset = 0xFF,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum KeyboardCommandData {
    ScancodeSet = 0xF0,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
#[allow(dead_code)]
enum MouseCommand {
    SetScaling1To1 = 0xE6,
    SetScaling2To1 = 0xE7,
    StatusRequest = 0xE9,
    GetDeviceId = 0xF2,
    EnableReporting = 0xF4,
    SetDefaultsDisable = 0xF5,
    SetDefaults = 0xF6,
    Reset = 0xFF,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum MouseCommandData {
    SetResolution = 0xE8,
    SetSampleRate = 0xF3,
}

pub struct Ps2 {
    data: Port<u8>,
    status: PortReadOnly<u8>,
    command: PortWriteOnly<u8>,
}

impl Ps2 {
    pub fn new() -> Self {
        Ps2 {
            data: Port::new(0x60),
            status: PortReadOnly::new(0x64),
            command: PortWriteOnly::new(0x64),
        }
    }

    fn status(&mut self) -> StatusFlags {
        StatusFlags::from_bits_truncate(unsafe { self.status.read() })
    }

    fn wait_read(&mut self) -> Result<(), Error> {
        let mut timeout = 1_000_000;
        while timeout > 0 {
            if self.status().contains(StatusFlags::OUTPUT_FULL) {
                return Ok(());
            }
            unsafe {
                pause();
            }
            timeout -= 1;
        }
        Err(Error::ReadTimeout)
    }

    fn wait_write(&mut self) -> Result<(), Error> {
        let mut timeout = 1_000_000;
        while timeout > 0 {
            if !self.status().contains(StatusFlags::INPUT_FULL) {
                return Ok(());
            }
            unsafe {
                pause();
            }
            timeout -= 1;
        }
        Err(Error::WriteTimeout)
    }

    fn flush_read(&mut self, message: &str) {
        let mut timeout = 100;
        while timeout > 0 {
            if self.status().contains(StatusFlags::OUTPUT_FULL) {
                ax_println!("ps2d: flush {}: {:X}", message, unsafe { self.data.read() });
            }
            unsafe {
                pause();
            }
            timeout -= 1;
        }
    }

    fn command(&mut self, command: Command) -> Result<(), Error> {
        self.wait_write()?;
        unsafe { self.command.write(command as u8) };
        Ok(())
    }

    fn read(&mut self) -> Result<u8, Error> {
        self.wait_read()?;
        Ok(unsafe { self.data.read() })
    }

    fn write(&mut self, data: u8) -> Result<(), Error> {
        self.wait_write()?;
        unsafe { self.data.write(data) };
        Ok(())
    }

    fn retry<F: Fn(&mut Self) -> Result<u8, Error>>(
        &mut self,
        name: fmt::Arguments,
        retries: usize,
        f: F,
    ) -> Result<u8, Error> {
        ax_println!("ps2d: {}", name);
        let mut res = Err(Error::NoMoreTries);
        for retry in 0..retries {
            res = f(self);
            match res {
                Ok(ok) => {
                    return Ok(ok);
                }
                Err(ref err) => {
                    ax_println!("ps2d: {}: retry {}/{}: {:?}", name, retry + 1, retries, err);
                }
            }
        }
        res
    }

    fn config(&mut self) -> Result<ConfigFlags, Error> {
        self.retry(format_args!("read config"), 4, |x| {
            x.command(Command::ReadConfig)?;
            x.read()
        })
        .map(ConfigFlags::from_bits_truncate)
    }

    fn set_config(&mut self, config: ConfigFlags) -> Result<(), Error> {
        self.retry(format_args!("write config"), 4, |x| {
            x.command(Command::WriteConfig)?;
            x.write(config.bits())?;
            Ok(0)
        })?;
        Ok(())
    }

    fn keyboard_command_inner(&mut self, command: u8) -> Result<u8, Error> {
        self.write(command)?;
        match self.read()? {
            0xFE => Err(Error::CommandRetry),
            value => Ok(value),
        }
    }

    fn keyboard_command(&mut self, command: KeyboardCommand) -> Result<u8, Error> {
        self.retry(format_args!("keyboard command {:?}", command), 4, |x| {
            x.keyboard_command_inner(command as u8)
        })
    }

    fn keyboard_command_data(
        &mut self,
        command: KeyboardCommandData,
        data: u8,
    ) -> Result<u8, Error> {
        self.retry(
            format_args!("keyboard command {:?} {:#x}", command, data),
            4,
            |x| {
                let res = x.keyboard_command_inner(command as u8)?;
                if res != 0xFA {
                    //TODO: error?
                    return Ok(res);
                }
                x.write(data);
                x.read()
            },
        )
    }

    fn mouse_command_inner(&mut self, command: u8) -> Result<u8, Error> {
        self.command(Command::WriteSecond)?;
        self.write(command)?;
        match self.read()? {
            0xFE => Err(Error::CommandRetry),
            value => Ok(value),
        }
    }

    fn mouse_command(&mut self, command: MouseCommand) -> Result<u8, Error> {
        self.retry(format_args!("mouse command {:?}", command), 4, |x| {
            x.mouse_command_inner(command as u8)
        })
    }

    fn mouse_command_data(&mut self, command: MouseCommandData, data: u8) -> Result<u8, Error> {
        self.retry(
            format_args!("mouse command {:?} {:#x}", command, data),
            4,
            |x| {
                let res = x.mouse_command_inner(command as u8)?;
                if res != 0xFA {
                    //TODO: error?
                    return Ok(res);
                }
                x.command(Command::WriteSecond)?;
                x.write(data as u8)?;
                x.read()
            },
        )
    }

    pub fn next(&mut self) -> Option<(bool, u8)> {
        let status = self.status();
        // let data = unsafe { self.data.read() };

        // ax_println!("data:{:?}",data);
        if status.contains(StatusFlags::OUTPUT_FULL) {
            let data = unsafe { self.data.read() };
            Some((!status.contains(StatusFlags::SECOND_OUTPUT_FULL), data))
        } else {
            None
        }
    }

    pub fn init_keyboard(&mut self) -> Result<(), Error> {
        let mut b;

        {
            // Enable first device
            self.command(Command::EnableFirst)?;

            // Clear remaining data
            self.flush_read("enable first");
        }

        {
            // Reset keyboard
            b = self.keyboard_command(KeyboardCommand::Reset)?;
            if b == 0xFA {
                b = self.read().unwrap_or(0);
                if b != 0xAA {
                    ax_println!("ps2d: keyboard failed self test: {:02X}", b);
                }
            } else {
                ax_println!("ps2d: keyboard failed to reset: {:02X}", b);
            }

            // Clear remaining data
            self.flush_read("keyboard reset");
        }

        self.retry(format_args!("keyboard defaults"), 4, |x| {
            x.flush_read("keyboard before defaults");

            // Set defaults and disable scanning
            let b = x.keyboard_command(KeyboardCommand::SetDefaultsDisable)?;
            if b != 0xFA {
                ax_println!("ps2d: keyboard failed to set defaults: {:02X}", b);
                return Err(Error::CommandRetry);
            }

            // Clear remaining data
            x.flush_read("keyboard after defaults");

            Ok(b)
        })?;

        {
            // Set scancode set to 2
            let scancode_set = 2;
            b = self.keyboard_command_data(KeyboardCommandData::ScancodeSet, scancode_set)?;
            if b != 0xFA {
                ax_println!(
                    "ps2d: keyboard failed to set scancode set {}: {:02X}",
                    scancode_set,
                    b
                );
            }

            // Clear remaining data
            self.flush_read("keyboard scancode");
        }

        Ok(())
    }

    pub fn init_mouse(&mut self) -> Result<bool, Error> {
        let mut b;

        {
            // Enable second device
            self.command(Command::EnableSecond)?;

            // Clear remaining data
            self.flush_read("enable second");
        }

        self.retry(format_args!("mouse reset"), 4, |x| {
            // Clear remaining data
            x.flush_read("mouse before reset");

            // Reset mouse
            let mut b = x.mouse_command(MouseCommand::Reset)?;
            if b == 0xFA {
                b = x.read()?;
                if b != 0xAA {
                    ax_println!("ps2d: mouse failed self test 1: {:02X}", b);
                    return Err(Error::CommandRetry);
                }

                b = x.read()?;
                if b != 0x00 {
                    ax_println!("ps2d: mouse failed self test 2: {:02X}", b);
                    return Err(Error::CommandRetry);
                }
            } else {
                ax_println!("ps2d: mouse failed to reset: {:02X}", b);
                return Err(Error::CommandRetry);
            }

            // Clear remaining data
            x.flush_read("mouse after reset");

            Ok(b)
        })?;

        {
            // Set defaults
            b = self.mouse_command(MouseCommand::SetDefaults)?;
            if b != 0xFA {
                ax_println!("ps2d: mouse failed to set defaults: {:02X}", b);
            }

            // Clear remaining data
            self.flush_read("mouse defaults");
        }

        {
            // Enable extra packet on mouse
            //TODO: show error return values
            if self.mouse_command_data(MouseCommandData::SetSampleRate, 200)? != 0xFA
                || self.mouse_command_data(MouseCommandData::SetSampleRate, 100)? != 0xFA
                || self.mouse_command_data(MouseCommandData::SetSampleRate, 80)? != 0xFA
            {
                ax_println!("ps2d: mouse failed to enable extra packet");
            }

            // Clear remaining data
            self.flush_read("enable extra mouse packet");
        }

        b = self.mouse_command(MouseCommand::GetDeviceId)?;
        let mouse_extra = if b == 0xFA {
            self.read()? == 3
        } else {
            ax_println!("ps2d: mouse failed to get device id: {:02X}", b);
            false
        };

        // Clear remaining data
        self.flush_read("get device id");

        {
            // Set resolution to maximum
            let resolution = 3;
            b = self.mouse_command_data(MouseCommandData::SetResolution, resolution)?;
            if b != 0xFA {
                ax_println!(
                    "ps2d: mouse failed to set resolution to {}: {:02X}",
                    resolution,
                    b
                );
            }

            // Clear remaining data
            self.flush_read("set sample rate");
        }

        {
            // Set scaling to 1:1
            b = self.mouse_command(MouseCommand::SetScaling1To1)?;
            if b != 0xFA {
                ax_println!("ps2d: mouse failed to set scaling: {:02X}", b);
            }

            // Clear remaining data
            self.flush_read("set sample rate");
        }

        {
            // Set sample rate to maximum
            let sample_rate = 200;
            b = self.mouse_command_data(MouseCommandData::SetSampleRate, sample_rate)?;
            if b != 0xFA {
                ax_println!(
                    "ps2d: mouse failed to set sample rate to {}: {:02X}",
                    sample_rate,
                    b
                );
            }

            // Clear remaining data
            self.flush_read("set sample rate");
        }

        {
            b = self.mouse_command(MouseCommand::StatusRequest)?;
            if b != 0xFA {
                ax_println!("ps2d: mouse failed to request status: {:02X}", b);
            } else {
                let a = self.read()?;
                let b = self.read()?;
                let c = self.read()?;

                ax_println!(
                    "ps2d: mouse status {:#x} resolution {:#x} sample rate {:#x}",
                    a,
                    b,
                    c
                );
            }
        }

        Ok(mouse_extra)
    }

    pub fn init(&mut self) -> Result<bool, Error> {
        // Clear remaining data
        self.flush_read("init start");

        {
            // Disable devices
            self.command(Command::DisableFirst)?;
            self.command(Command::DisableSecond)?;

            // Clear remaining data
            self.flush_read("disable");
        }

        // Disable clocks, disable interrupts, and disable translate
        let mut config;
        {
            // Since the default config may have interrupts enabled, and the kernel may eat up
            // our data in that case, we will write a config without reading the current one
            config = ConfigFlags::POST_PASSED
                | ConfigFlags::FIRST_DISABLED
                | ConfigFlags::SECOND_DISABLED;
            ax_println!("ps2d: config set {:?}", config);
            self.set_config(config)?;

            // Clear remaining data
            self.flush_read("disable interrupts");
        }

        {
            // Perform the self test
            self.command(Command::TestController)?;
            assert_eq!(self.read()?, 0x55);

            // Clear remaining data
            self.flush_read("test controller");
        }

        // Initialize keyboard
        self.init_keyboard()?;

        // Initialize mouse
        let (mouse_found, mouse_extra) = match self.init_mouse() {
            Ok(ok) => (true, ok),
            Err(err) => {
                ax_println!("ps2d: failed to initialize mouse: {:?}", err);
                (false, false)
            }
        };

        {
            // Enable keyboard data reporting
            // Use inner function to prevent retries
            self.keyboard_command_inner(KeyboardCommand::EnableReporting as u8)?;
            // Response is ignored since scanning is now on
            //TODO: fix by using interrupts?
        }

        if mouse_found {
            // Enable mouse data reporting
            // Use inner function to prevent retries
            self.mouse_command_inner(MouseCommand::EnableReporting as u8)?;
            // Response is ignored since scanning is now on
            //TODO: fix by using interrupts?
        }

        // Enable clocks and interrupts
        {
            config.remove(ConfigFlags::FIRST_DISABLED);
            config.insert(ConfigFlags::FIRST_TRANSLATE);
            config.insert(ConfigFlags::FIRST_INTERRUPT);
            if mouse_found {
                config.remove(ConfigFlags::SECOND_DISABLED);
                config.insert(ConfigFlags::SECOND_INTERRUPT);
            } else {
                config.insert(ConfigFlags::SECOND_DISABLED);
                config.remove(ConfigFlags::SECOND_INTERRUPT);
            }
            ax_println!("ps2d: config set {:?}", config);
            self.set_config(config)?;
        }

        // Clear remaining data
        self.flush_read("init finish");


        Ok(mouse_extra)
    }
}
