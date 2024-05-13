use crate::constants::{
    INIT_DELAY, PMW_3360_FIRMWARE, READ_ADDRESS_DATA_DELAY, REG_CONFIG_2, REG_DELTA_X_H,
    REG_DELTA_X_L, REG_DELTA_Y_H, REG_DELTA_Y_L, REG_MOTION, REG_MOTION_BURST, REG_POWER_UP_RESET,
    REG_SROM_ENABLE, REG_SROM_LOAD_BURST, SROM_DOWNLOAD_DELAY, SROM_ENABLE_DELAY,
};
use crate::motion_data::MotionData;
use alloc::borrow::ToOwned;
use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use cortex_m::prelude::_embedded_hal_blocking_spi_Transfer;
use fugit::HertzU32;
use stm32f1xx_hal::afio::MAPR;
use stm32f1xx_hal::device::SPI1;
use stm32f1xx_hal::gpio::{Alternate, Output, Pin};
use stm32f1xx_hal::rcc::Clocks;
use stm32f1xx_hal::spi::{Mode, Phase, Polarity, Spi, Spi1NoRemap};
use stm32f1xx_hal::timer::SysDelay;

pub type PmwCe = Pin<'A', 4, Output>;
pub type PmwSck = Pin<'A', 5, Alternate>;
pub type PmwMiso = Pin<'A', 6>;
pub type PmwMosi = Pin<'A', 7, Alternate>;
pub type PmwSpi = Spi<SPI1, Spi1NoRemap, (PmwSck, PmwMiso, PmwMosi), u8>;

pub struct PmwDriver {
    spi: PmwSpi,
    chip_enable_pin: PmwCe,
    delay: Rc<RefCell<SysDelay>>,
}

impl PmwDriver {
    pub fn new(
        pmw_ce: PmwCe,
        pmw_sck: PmwSck,
        pmw_miso: PmwMiso,
        pmw_mosi: PmwMosi,
        spi1: SPI1,
        mapr: &mut MAPR,
        delay: Rc<RefCell<SysDelay>>,
        clocks: Clocks,
    ) -> Self {
        let pins = (pmw_sck, pmw_miso, pmw_mosi);

        let spi_mode = Mode {
            polarity: Polarity::IdleHigh,
            phase: Phase::CaptureOnSecondTransition,
        };

        let spi = Spi::spi1(spi1, pins, mapr, spi_mode, HertzU32::kHz(10), clocks);

        Self {
            spi,
            chip_enable_pin: pmw_ce,
            delay,
        }
    }

    pub fn init(&mut self) {
        self.chip_enable_pin.set_low();
        self.delay.borrow_mut().delay(INIT_DELAY);
        self.chip_enable_pin.set_high();

        self.pmw_write(REG_POWER_UP_RESET, vec![0x5a]);

        self.delay.borrow_mut().delay(INIT_DELAY);
        self.pmw_read(REG_MOTION, 1);
        self.pmw_read(REG_DELTA_X_L, 1);
        self.pmw_read(REG_DELTA_X_H, 1);
        self.pmw_read(REG_DELTA_Y_L, 1);
        self.pmw_read(REG_DELTA_Y_H, 1);

        self.disable_rest_mode();

        self.pmw_write(REG_SROM_ENABLE, vec![0x1d]);
        self.delay.borrow_mut().delay(SROM_ENABLE_DELAY);
        self.pmw_write(REG_SROM_ENABLE, vec![0x18]);
        self.pmw_write(REG_SROM_LOAD_BURST, PMW_3360_FIRMWARE.to_vec());
        self.delay.borrow_mut().delay(SROM_DOWNLOAD_DELAY);
        self.pmw_write(REG_CONFIG_2, vec![0x00]);
    }

    pub fn enter_loop(&mut self, mut motion_handler: impl FnMut(MotionData)) -> ! {
        self.pmw_write(REG_MOTION_BURST, vec![0xff]);
        loop {
            let motion_data: MotionData = self.pmw_read(REG_MOTION_BURST, 12).into();
            motion_handler(motion_data);
        }
    }

    // fn enable_rest_mode(&mut self) {
    //     let mut config2 = self.pmw_read(0x10, 1);
    //     config2[0] |= 1 << 5;
    //     self.pmw_write(0x10, config2);
    // }

    fn disable_rest_mode(&mut self) {
        let mut config2 = self.pmw_read(0x10, 1);
        config2[0] &= !(1 << 5);
        self.pmw_write(0x10, config2);
    }

    fn pmw_write(&mut self, address: u8, data: Vec<u8>) {
        self.pmw_transfer(true, address, data);
    }

    fn pmw_read(&mut self, address: u8, count: usize) -> Vec<u8> {
        self.pmw_transfer(false, address, vec![0xff; count])
    }

    fn pmw_transfer(&mut self, is_write: bool, address: u8, mut data: Vec<u8>) -> Vec<u8> {
        let first_byte = if is_write {
            (1 << 7) | address
        } else {
            !(1 << 7) & address
        };

        self.chip_enable_pin.set_low();

        self.spi
            .transfer(&mut [first_byte])
            .expect("Failed to transfer bytes over SPI1.");

        self.delay.borrow_mut().delay(READ_ADDRESS_DATA_DELAY);

        let result = self
            .spi
            .transfer(&mut data)
            .expect("Failed to transfer bytes over SPI1.")
            .to_owned();

        self.chip_enable_pin.set_high();

        result
    }
}
