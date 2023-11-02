pub mod key;
pub mod key2ascii;

use ascii::AsciiChar;
use axlog::ax_println;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

use bitflags::bitflags;

use crate::{
    platform::console,
    ps2_key::{
        key::{InputEvent, Key},
        key2ascii::KeyToAscii,
    },
};

pub fn ps2_test() {
    let mut ps2 = Ps2Keyboard::new();
    ps2.init().unwrap();

    let mut count = 1000;
    let mut key_to_ascii = KeyToAscii::new();
    console::reset_color();
    ax_println!("Receiving 1000 ps2 messages:");
    while count > 0 {
        loop {
            if let Some(val) = ps2.recv() {
                let event = {
                    if val <= 0x80 {
                        InputEvent::KeyBoard(
                            Key::try_from(val as u16).unwrap(),
                            key::KeyStatus::Pressed,
                        )
                    } else if val > 0x80 && val <= 0xD8 {
                        InputEvent::KeyBoard(
                            Key::try_from((val - 0x80) as u16).unwrap(),
                            key::KeyStatus::Released,
                        )
                    } else if val == 0xE0 {
                        // Extended code
                        todo!();
                    } else {
                        panic!("Unsupport value:{:x?}", val);
                    }
                };
                let asciis = key_to_ascii.input(event);
                if let Some(asciis) = asciis {
                    for ascii in asciis {
                        if ascii != AsciiChar::Null {
                            ax_println!("Receive input:{:?} from PS/2", ascii);
                        }
                    }
                }
                break;
            }
        }
        count -= 1;
    }
    ax_println!("Receive complete");
}

#[derive(Debug)]
pub enum Ps2InitError {
    CommandFailed,
}

bitflags! {
    #[derive(Debug)]
    pub struct StatusFlags: u8 {
        const OUTPUT_FULL           = 1 << 0;
        const INPUT_FULL            = 1 << 1;
        const SYSTEM                = 1 << 2;
        const COMMAND               = 1 << 3;
        // Chipset specific
        const KEYBOARD_LOCK         = 1 << 4;
        // Chipset specific
        const SECOND_OUTPUT_FULL    = 1 << 5;
        const TIME_OUT              = 1 << 6;
        const PARITY                = 1 << 7;
    }
}

bitflags! {
    #[derive(Debug,Clone, Copy)]
    pub struct Ps2Config: u8 {
        const FIRST_INTERRUPT   = 1 << 0;
        const SECOND_INTERRUPT  = 1 << 1;
        const POST_PASSED       = 1 << 2;
        const RESERVED_3        = 1 << 3;
        const FIRST_DISABLED    = 1 << 4;
        const SECOND_DISABLED   = 1 << 5;
        const FIRST_TRANSLATE   = 1 << 6;
        const RESERVED_7        = 1 << 7;
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
#[allow(dead_code)]
enum Ps2Command {
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
enum Ps2KeyboardCommand {
    EnableReporting = 0xF4,
    SetDefaultsDisable = 0xF5,
    SetDefaults = 0xF6,
    Reset = 0xFF,
    ScancodeSet = 0xF0,
}

pub struct Ps2Keyboard {
    data: Port<u8>,
    status: PortReadOnly<u8>,
    command: PortWriteOnly<u8>,
}

impl Ps2Keyboard {
    pub fn new() -> Self {
        Ps2Keyboard {
            data: Port::new(0x60),
            status: PortReadOnly::new(0x64),
            command: PortWriteOnly::new(0x64),
        }
    }

    fn set_ps2_config(&mut self, config: Ps2Config) {
        self.ps2_command(Ps2Command::WriteConfig);
        self.write(config.bits());
    }

    fn ps2_command(&mut self, command: Ps2Command) {
        unsafe { self.command.write(command as u8) };
    }

    fn read(&mut self) -> u8 {
        unsafe { self.data.read() }
    }

    fn write(&mut self, data: u8) {
        unsafe { self.data.write(data) };
    }

    fn keyboard_command(&mut self, command: Ps2KeyboardCommand) -> Result<(),Ps2InitError>{
        self.write(command as u8);
        match self.read(){
            0xFA => Ok(()),
            _ => Err(Ps2InitError::CommandFailed)
        }
    }

    pub fn recv(&mut self) -> Option<u8> {
        let status = StatusFlags::from_bits_truncate(unsafe { self.status.read() });
        if status.contains(StatusFlags::OUTPUT_FULL) {
            let data = unsafe { self.data.read() };
            Some(data)
        } else {
            None
        }
    }

    pub fn init(&mut self) -> Result<(), Ps2InitError> {
        // TODO: Support PS/2 mouse
        // Disable devices
        self.ps2_command(Ps2Command::DisableFirst);
        self.ps2_command(Ps2Command::DisableSecond);
        // Disable clocks, disable interrupts, and disable translate
        let mut config =
            Ps2Config::POST_PASSED | Ps2Config::FIRST_DISABLED | Ps2Config::SECOND_DISABLED;
        self.set_ps2_config(config);

        // Perform the self test
        self.ps2_command(Ps2Command::TestController);
        assert_eq!(self.read(), 0x55);

        // Initialize keyboard
        self.ps2_command(Ps2Command::EnableFirst);
        // Reset keyboard
        self.keyboard_command(Ps2KeyboardCommand::Reset)?;
        let res = self.read();
        if res != 0xAA {
            ax_println!("PS/2 keyboard failed self test: {:02X}", res);
            return Err(Ps2InitError::CommandFailed);
        }
        self.keyboard_command(Ps2KeyboardCommand::SetDefaultsDisable)?;

        // Set scancode set to 2
        {
            let scancode_set = 2;
            self.keyboard_command(Ps2KeyboardCommand::ScancodeSet)?;
            self.write(scancode_set);
            let res = self.read();
            if res != 0xFA {
                ax_println!(
                    "PS/2 keyboard failed to set scancode set {}: {:02X}",
                    scancode_set,
                    res
                );
                return Err(Ps2InitError::CommandFailed);
            }
        }
        // Enable keyboard
        let res = self.keyboard_command(Ps2KeyboardCommand::EnableReporting);
        config.remove(Ps2Config::FIRST_DISABLED);
        config.insert(Ps2Config::FIRST_TRANSLATE);
        config.insert(Ps2Config::FIRST_INTERRUPT);
        ax_println!("PS/2 config set {:?}", config);
        self.set_ps2_config(config);
        Ok(())
    }
}
