use crate::button_data::ButtonData;
use crate::motion_data::MotionData;
use crate::mouse_report::MouseReport;
use stm32_usbd::UsbBus;
use stm32f1xx_hal::usb;
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_hid_device::Hid;

const USB_CLASS_HID: u8 = 0x03;
const POLL_TIME_MS: u8 = 5;

static mut USB_BUS_ALLOCATOR: Option<UsbBusAllocator<UsbBus<usb::Peripheral>>> = None;

pub struct UsbDriver<'a> {
    usb_device: UsbDevice<'a, UsbBus<usb::Peripheral>>,
    hid: Hid<'a, MouseReport, UsbBus<usb::Peripheral>>,
}

impl<'a> UsbDriver<'a> {
    pub fn new(usb_peripheral: usb::Peripheral) -> Self {
        unsafe {
            let _ = USB_BUS_ALLOCATOR.insert(UsbBus::new(usb_peripheral));
        }

        let hid = unsafe { Hid::new(USB_BUS_ALLOCATOR.as_ref().unwrap(), POLL_TIME_MS) };
        let usb_device = unsafe {
            UsbDeviceBuilder::new(
                USB_BUS_ALLOCATOR.as_ref().unwrap(),
                UsbVidPid(0x16c0, 0x27dd),
            )
            .manufacturer("Fake company")
            .product("IDK MOUSE")
            .serial_number("rev 3")
            .device_class(USB_CLASS_HID)
            .build()
        };

        Self { hid, usb_device }
    }

    pub fn poll(&mut self) {
        self.usb_device.poll(&mut [&mut self.hid]);
    }

    pub fn handle_data(&mut self, motion_data: MotionData, button_data: ButtonData) {
        let _ = self
            .hid
            .send_report(&MouseReport::new(motion_data, button_data));
    }
}
