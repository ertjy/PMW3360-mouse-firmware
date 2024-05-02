use stm32_usbd::UsbBus;
use stm32f1xx_hal::gpio::Pin;
use stm32f1xx_hal::{pac, usb};
use usb_device::bus::UsbBusAllocator;
use usb_device::device;
use usb_device::prelude::*;
use usbd_hid_device::Hid;
use crate::motion_data::MotionData;
use crate::mouse_report::MouseReport;

const USB_CLASS_HID: u8 = 0x03;
const USB_SUBCLASS_HID: u8 = 0x00;
const USB_PROTOCOL_HID: u8 = 0x02;
const HID_VERSION: [u8; 2] = [0x10, 0x01];
const POLL_TIME_MS: u8 = 5;

pub struct UsbDriver<'a> {
    usb_device: UsbDevice<'a, UsbBus<usb::Peripheral>>,
    usb_bus: UsbBusAllocator<UsbBus<usb::Peripheral>>,
    hid: Hid<'a, MouseReport, UsbBus<usb::Peripheral>>,
}

impl<'a> UsbDriver<'a> {
    pub fn new(usb_peripheral: usb::Peripheral) -> Self {
        let usb_bus = UsbBus::new(usb_peripheral);
        let hid = Hid::new(&usb_bus, POLL_TIME_MS);
        let usb_device = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Fake company")
            .product("IDK MOUSE")
            .serial_number("rev 3")
            .device_class(USB_CLASS_HID)
            .build();

        Self {
            usb_bus,
            hid,
            usb_device,
        }
    }

    pub fn poll(&mut self) {
        self.usb_device.poll(&mut [&mut self.hid]);
    }

    pub fn handle_motion_data(&mut self, motion_data: MotionData) {
        let _ = self.hid.send_report(&MouseReport::new(motion_data));
    }
}