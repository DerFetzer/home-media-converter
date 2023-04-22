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
    SqiReq1 = 0x871,
}

/// Software SPI since there is no half-duplex implementation in the HAL.
pub(crate) struct Smi {
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

    pub fn read_extended(&mut self, device_address: u8, register_address: u16) -> u16 {
        self.write(device_address, T1Registers::Regcr as u8, MMD1F as u16);
        self.write(device_address, T1Registers::Addar as u8, register_address);
        self.write(
            device_address,
            T1Registers::Regcr as u8,
            MMD1F as u16 | 0x4000,
        );
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
}
