use crate::button_data::ButtonData;
use crate::motion_data::MotionData;
use usbd_hid_device::HidReport;

pub struct MouseReport {
    // Bytes usage:
    // byte 0: bits 0..2 = buttons
    // byte 1: x
    // byte 2: y
    // byte 3: wheel
    bytes: [u8; 4],
}

impl MouseReport {
    pub(crate) fn new(motion_data: MotionData, button_data: ButtonData) -> Self {
        Self {
            bytes: [
                button_data.into(),
                motion_data.delta_x as u8,
                motion_data.delta_y as u8,
                0,
            ],
        }
    }
}

impl AsRef<[u8]> for MouseReport {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl HidReport for MouseReport {
    const DESCRIPTOR: &'static [u8] = &[
        0x05, 0x01, // USAGE_PAGE Generic Desktop
        0x09, 0x02, // USAGE Mouse
        0xa1, 0x01, // COLLECTION Application
        0x09, 0x01, // USAGE Pointer
        0xa1, 0x00, // COLLECTION Physical
        0x05, 0x09, // USAGE_PAGE Button
        0x19, 0x01, // USAGE_MINIMUM Button 1
        0x29, 0x03, // USAGE_MAXIMUM Button 3
        0x15, 0x00, // LOGICAL_MINIMUM 0
        0x25, 0x01, // LOGICAL_MAXIMUM 1
        0x95, 0x03, // REPORT_COUNT 3
        0x75, 0x01, // REPORT_SIZE 1
        0x81, 0x02, // INPUT Data,Var,Abs
        0x95, 0x01, // REPORT_COUNT 1
        0x75, 0x05, // REPORT_SIZE 5
        0x81, 0x01, // INPUT Cnst,Ary,Abs
        0x05, 0x01, // USAGE_PAGE Generic Desktop
        0x09, 0x30, // USAGE X
        0x09, 0x31, // USAGE Y
        0x09, 0x38, // USAGE Wheel
        0x15, 0x81, // LOGICAL_MINIMUM -127
        0x25, 0x7f, // LOGICAL_MAXIMUM 127
        0x75, 0x08, // REPORT_SIZE 8
        0x95, 0x03, // REPORT_COUNT 3
        0x81, 0x06, // INPUT Data,Var,Rel
        0xc0, // END COLLECTION
        0xc0, // END COLLECTION
    ];
}
