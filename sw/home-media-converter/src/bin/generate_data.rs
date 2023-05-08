#![no_main]
#![no_std]

use home_media_converter as _;
use stm32f0xx_hal::pac::Peripherals; // global logger + panicking-behavior + memory layout
use stm32f0xx_hal::prelude::*;

use home_media_converter::smi::{Smi, T1Registers};

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

    let mut smi = Smi::new(mdc, mdio);

    // smi.write_extended(8, T1Registers::PrbsCtrl1 as u16, 0x0557); // PRBS
    smi.write_extended(8, T1Registers::PrbsCtrl1 as u16, 0x1555); // MAC
    smi.write_extended(8, T1Registers::PrbsCtrl4 as u16, 0x55BF);

    loop {
        defmt::println!("");
        defmt::println!(
            "Ctrl1: {:b}",
            smi.read_extended(8, T1Registers::PrbsCtrl1 as u16)
        );
        defmt::println!(
            "Status1: {:b}",
            smi.read_extended(8, T1Registers::PrbsStatus1 as u16)
        );
        defmt::println!(
            "Status2: {:b}",
            smi.read_extended(8, T1Registers::PrbsStatus2 as u16)
        );
        defmt::println!(
            "Status3: {:b}",
            smi.read_extended(8, T1Registers::PrbsStatus3 as u16)
        );
        defmt::println!(
            "Status4: {:b}",
            smi.read_extended(8, T1Registers::PrbsStatus4 as u16)
        );
        defmt::println!(
            "Status6: {:b}",
            smi.read_extended(8, T1Registers::PrbsStatus6 as u16)
        );
        defmt::println!(
            "Status8: {:b}",
            smi.read_extended(8, T1Registers::PrbsStatus8 as u16)
        );
        defmt::println!(
            "Status9: {:b}",
            smi.read_extended(8, T1Registers::PrbsStatus9 as u16)
        );
        defmt::println!(
            "SqiReg1: {:b}",
            smi.read_extended(8, T1Registers::SqiReq1 as u16)
        );
        cortex_m::asm::delay(8_000_000);
    }
}
