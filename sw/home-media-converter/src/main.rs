#![no_main]
#![no_std]

use home_media_converter as _;
use stm32f0xx_hal::pac::Peripherals; // global logger + panicking-behavior + memory layout
use stm32f0xx_hal::prelude::*;

use crate::smi::{Smi, T1Registers};

mod smi;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");
    let mut dp = Peripherals::take().unwrap();
    let mut rcc = dp.RCC.configure().sysclk(8.mhz()).freeze(&mut dp.FLASH);

    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiof = dp.GPIOF.split(&mut rcc);

    let (jumper3, mut led_t1_conn, mdc, mdio) = cortex_m::interrupt::free(move |cs| {
        let _int_t1 = gpioa.pa0;
        let _int_tx = gpioa.pa1;

        let jumper3 = gpioa.pa2.into_pull_down_input(cs);
        let _jumper4 = gpioa.pa3.into_pull_down_input(cs);

        let _led3 = gpiof.pf0.into_push_pull_output(cs);
        let _led4 = gpiof.pf1.into_push_pull_output(cs);
        let led_t1_conn = gpioa.pa4.into_push_pull_output(cs);

        let mdc = gpioa.pa5.into_push_pull_output(cs);
        let mut mdio = gpioa.pa7.into_open_drain_output(cs);
        mdio.set_high().unwrap();

        (jumper3, led_t1_conn, mdc, mdio)
    });

    let mut sdi = Smi::new(mdc, mdio);
    loop {
        if jumper3.is_high().unwrap() {
            led_t1_conn.set_high().unwrap();
            let worst_sqi = (sdi.read_extended(8, T1Registers::SqiReq1 as u16) >> 5) & 0b111;
            defmt::println!("Worst SQI: {}", worst_sqi);
            for _ in 0..(7 - worst_sqi) {
                led_t1_conn.set_low().unwrap();
                cortex_m::asm::delay(300_000);
                led_t1_conn.set_high().unwrap();
                cortex_m::asm::delay(300_000);
            }
        } else {
            led_t1_conn.set_low().unwrap();
        }
        cortex_m::asm::delay(15_000_000);
    }
}
