use alloc::vec::Vec;
use rtt_target::rprintln;

const SWAP_DIRECTION: bool = true;

#[derive(Debug)]
pub struct MotionData {
    pub(crate) delta_x: i8,
    pub(crate) delta_y: i8,
}

impl From<Vec<u8>> for MotionData {
    fn from(value: Vec<u8>) -> Self {
        if value.len() != 12 {
            panic!("Tried to decode invalid motion data.");
        }

        if value[0] & (1 << 7) != 0 {
            let delta_x = value[2] as i16 | ((value[3] as i16) << 8);
            let delta_y = value[4] as i16 | ((value[5] as i16) << 8);
            if SWAP_DIRECTION {
                Self {delta_x: -delta_x as i8, delta_y: -delta_y as i8}
            } else {
                Self {delta_x: delta_x as i8, delta_y: delta_y as i8}
            }
        } else {
            Self {delta_x: 0, delta_y: 0}
        }
    }
}