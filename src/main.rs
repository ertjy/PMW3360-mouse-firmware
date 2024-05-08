#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

pub mod constants;
pub mod pmw_driver;
pub mod usb_driver;
pub mod motion_data;
pub mod mouse_report;
pub mod button_driver;

extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout;
use core::cell::RefCell;
use cortex_m::asm::delay;
use cortex_m::peripheral::SYST;
use panic_rtt_target as _;

use crate::constants::{
    INIT_DELAY, PMW_3360_FIRMWARE, REG_POWER_UP_RESET, READ_ADDRESS_DATA_DELAY, SROM_DOWNLOAD_DELAY,
    SROM_ENABLE_DELAY,
};
use crate::pmw_driver::PmwDriver;
use cortex_m_rt::entry;
use fugit::{Duration, Hertz, HertzU32, MicrosDurationU32};
use rtt_target::{rprint, rprintln, rtt_init_print};
use stm32f1xx_hal::device::SPI1;
use stm32f1xx_hal::gpio::{Alternate, Output, Pin};
use stm32f1xx_hal::pac::{CorePeripherals, Peripherals};
use stm32f1xx_hal::rcc::Clocks;
use stm32f1xx_hal::spi::Spi1NoRemap;
use stm32f1xx_hal::time::Hz;
use stm32f1xx_hal::timer::{SysDelay, Timer};
use stm32f1xx_hal::{prelude::*, spi::{Mode, Phase, Polarity, Spi}, usb};
use crate::motion_data::MotionData;
use crate::usb_driver::UsbDriver;

type PmwSpi =
    Spi<SPI1, Spi1NoRemap, (Pin<'A', 5, Alternate>, Pin<'A', 6>, Pin<'A', 7, Alternate>), u8>;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    unsafe {
        let start = cortex_m_rt::heap_start() as usize;
        ALLOCATOR.init(start, 16384);
    }

    let cp = CorePeripherals::take().unwrap();
    let dp = Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain();
    let mut gpioa = dp.GPIOA.split();

    let clocks = rcc.cfgr.use_hse(HertzU32::MHz(8))
        .sysclk(HertzU32::MHz(72))
        .hclk(HertzU32::MHz(72))
        .pclk1(HertzU32::MHz(36))
        .pclk2(HertzU32::MHz(72))
        .freeze(&mut flash.acr);

    let delay = Rc::new(RefCell::new(cp.SYST.delay(&clocks)));

    let mut pmw_driver = PmwDriver::new(
        gpioa.pa4.into_push_pull_output(&mut gpioa.crl),
        gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl),
        gpioa.pa6.into_floating_input(&mut gpioa.crl),
        gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl),
        dp.SPI1,
        &mut afio.mapr,
        delay.clone(),
        clocks,
    );
    pmw_driver.init();

    let usb_dm = gpioa.pa11.into_push_pull_output(&mut gpioa.crh);
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low();
    delay.borrow_mut().delay(MicrosDurationU32::millis(10));

    let usb_peripheral = usb::Peripheral {
        usb: dp.USB,
        pin_dm: usb_dm.into_floating_input(&mut gpioa.crh),
        pin_dp: usb_dp.into_floating_input(&mut gpioa.crh),
    };

    let mut usb_driver = UsbDriver::new(usb_peripheral);
    usb_driver.poll();

    // loop {
    //     usb_driver.poll();
    //     usb_driver.handle_motion_data(MotionData{
    //         delta_x: 1,
    //         delta_y: 0,
    //     })
    // }
    pmw_driver.enter_loop(|motion_data| {
        // position_x += motion_data.delta_x as f32 / 65536f32;
        // position_y += motion_data.delta_y as f32 / 65536f32;
        rprintln!("{:?}", motion_data);
        usb_driver.poll();
        usb_driver.handle_motion_data(motion_data);
    });
}

#[global_allocator]
pub static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    panic!("Ran out of memory");
}
