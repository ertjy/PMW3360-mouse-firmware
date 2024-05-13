#[derive(Debug)]
pub struct ButtonData {
    pub left_click: bool,
    pub middle_click: bool,
    pub right_click: bool,
}

impl From<ButtonData> for u8 {
    fn from(button_data: ButtonData) -> Self {
        let mut byte = 0u8;

        if button_data.left_click {
            byte |= 1 << 0;
        }

        if button_data.middle_click {
            byte |= 1 << 1;
        }

        if button_data.right_click {
            byte |= 1 << 2;
        }

        byte
    }
}
