#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

pub mod button_data;
pub mod button_driver;
pub mod constants;
pub mod motion_data;
pub mod mouse_report;
pub mod pmw_driver;
pub mod usb_driver;

extern crate alloc;

use alloc::rc::Rc;
use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout;
use core::cell::RefCell;
use panic_rtt_target as _;

use crate::button_driver::ButtonDriver;
use crate::pmw_driver::PmwDriver;
use crate::usb_driver::UsbDriver;
use cortex_m_rt::entry;
use fugit::{HertzU32, MicrosDurationU32};
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::pac::{CorePeripherals, Peripherals};
use stm32f1xx_hal::{prelude::*, usb};

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
    let rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain();
    let mut gpioa = dp.GPIOA.split();
    let mut gpioc = dp.GPIOC.split();

    let clocks = rcc
        .cfgr
        .use_hse(HertzU32::MHz(8))
        .sysclk(HertzU32::MHz(72))
        .hclk(HertzU32::MHz(72))
        .pclk1(HertzU32::MHz(36))
        .pclk2(HertzU32::MHz(72))
        .freeze(&mut flash.acr);

    let delay = Rc::new(RefCell::new(cp.SYST.delay(&clocks)));

    let button_driver = ButtonDriver::new(
        gpioc.pc3.into_floating_input(&mut gpioc.crl),
        gpioc.pc4.into_floating_input(&mut gpioc.crl),
        gpioc.pc5.into_floating_input(&mut gpioc.crl),
    );

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
    // usb_driver.poll();

    pmw_driver.enter_loop(|motion_data| {
        let button_data = button_driver.get_current_data();
        // rprintln!("{:?}", motion_data);
        rprintln!("{:?}", button_data);

        usb_driver.handle_data(motion_data, button_data);
        usb_driver.poll();
    });
}

#[global_allocator]
pub static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    panic!("Ran out of memory");
}
