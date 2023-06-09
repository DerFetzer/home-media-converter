use stm32f0xx_hal::gpio::{
    gpioa::{PA5, PA7},
    OpenDrain, Output, PushPull,
};
use stm32f0xx_hal::prelude::*;

const MDC_DELAY_CYCLES: u32 = 0;
const MMD1F: u8 = 0x1F;

#[repr(u16)]
pub enum T1Registers {
    Regcr = 0xD,
    Addar = 0xE,
    MiiReg16 = 0x16,
    SqiReq1 = 0x871,
    PrbsStatus1 = 0x618,
    PrbsCtrl1 = 0x619,
    PrbsCtrl2 = 0x61A,
    PrbsCtrl3 = 0x61B,
    PrbsStatus2 = 0x61C,
    PrbsStatus3 = 0x61D,
    PrbsStatus4 = 0x61E,
    PrbsStatus6 = 0x620,
    PrbsStatus8 = 0x622,
    PrbsStatus9 = 0x623,
    PrbsCtrl4 = 0x624,
}

/// Software SPI since there is no half-duplex implementation in the HAL.
pub struct Smi {
    mdc: PA5<Output<PushPull>>,
    mdio: PA7<Output<OpenDrain>>,
}

impl Smi {
    pub fn new(mdc: PA5<Output<PushPull>>, mdio: PA7<Output<OpenDrain>>) -> Self {
        Self { mdc, mdio }
    }

    pub fn read(&mut self, device_address: u8, register_address: u8) -> u16 {
        let cmd = 0b0110 << 12
            | (device_address as u16 & 0b11111) << 7
            | (register_address as u16 & 0b11111) << 2
            | 0b11;
        for i in 0..16 {
            self.mdc.set_low().unwrap();
            if (cmd >> (15 - i)) & 0b1 == 0b1 {
                self.mdio.set_high().unwrap();
            } else {
                self.mdio.set_low().unwrap();
            }
            cortex_m::asm::delay(MDC_DELAY_CYCLES);
            self.mdc.set_high().unwrap();
            cortex_m::asm::delay(MDC_DELAY_CYCLES);
        }
        self.mdio.set_high().unwrap();

        let mut recv = 0;

        for i in 0..16 {
            self.mdc.set_low().unwrap();
            cortex_m::asm::delay(MDC_DELAY_CYCLES);
            recv |= (if self.mdio.is_high().unwrap() { 1 } else { 0 }) << (15 - i);
            self.mdc.set_high().unwrap();
            cortex_m::asm::delay(MDC_DELAY_CYCLES);
        }

        recv
    }

    fn load_extended_register(&mut self, device_address: u8, register_address: u16) {
        self.write(device_address, T1Registers::Regcr as u8, MMD1F as u16);
        self.write(device_address, T1Registers::Addar as u8, register_address);
        self.write(
            device_address,
            T1Registers::Regcr as u8,
            MMD1F as u16 | 0x4000,
        );
    }
    pub fn read_extended(&mut self, device_address: u8, register_address: u16) -> u16 {
        self.load_extended_register(device_address, register_address);
        self.read(device_address, T1Registers::Addar as u8)
    }

    pub fn write(&mut self, device_address: u8, register_address: u8, value: u16) {
        let cmd = 0b0101 << 12
            | (device_address as u16 & 0b11111) << 7
            | (register_address as u16 & 0b11111) << 2
            | 0b10;
        let cmd = (cmd as u32) << 16 | value as u32;
        for i in 0..32 {
            self.mdc.set_low().unwrap();
            if (cmd >> (31 - i)) & 0b1 == 0b1 {
                self.mdio.set_high().unwrap();
            } else {
                self.mdio.set_low().unwrap();
            }
            cortex_m::asm::delay(MDC_DELAY_CYCLES);
            self.mdc.set_high().unwrap();
            cortex_m::asm::delay(MDC_DELAY_CYCLES);
        }
        self.mdio.set_high().unwrap();
    }

    pub fn write_extended(&mut self, device_address: u8, register_address: u16, value: u16) {
        self.load_extended_register(device_address, register_address);
        self.write(device_address, T1Registers::Addar as u8, value);
    }
}
