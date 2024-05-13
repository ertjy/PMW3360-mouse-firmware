use crate::button_data::ButtonData;
use stm32f1xx_hal::gpio::{Floating, Input, Pin};

pub struct ButtonDriver {
    left_click: Pin<'C', 3, Input<Floating>>,
    right_click: Pin<'C', 4, Input<Floating>>,
    middle_click: Pin<'C', 5, Input<Floating>>,
}

impl ButtonDriver {
    pub fn new(
        left_click: Pin<'C', 3, Input<Floating>>,
        right_click: Pin<'C', 4, Input<Floating>>,
        middle_click: Pin<'C', 5, Input<Floating>>,
    ) -> Self {
        Self {
            left_click,
            right_click,
            middle_click,
        }
    }

    pub fn get_current_data(&self) -> ButtonData {
        ButtonData {
            left_click: self.left_click.is_low(),
            right_click: self.right_click.is_low(),
            middle_click: self.middle_click.is_low(),
        }
    }
}
