use embedded_hal::digital::v2::InputPin;
use heapless::{FnvIndexMap, IndexMap};
use rp_pico::{hal::gpio::dynpin::*, Pins};
use usbd_hid::descriptor::generator_prelude::*;
// This is our custom keyboard report descriptor. It has a bit-packed u8 that
// represents the modifier keys (per HID usage tables), an empty reserve byte,
// and then two arrays of keycodes. The keycodes live in the 42-byte keycodes
// array.
//
// This was done rather than having two 21-byte keycode arrays (one
// for each half of the split keyboard), because the HID spec uses the order
// of the array to parse the order of the keypresses, and so by sending
// two separate arrays of keycodes, the codes in the second array would always
// behave as if they were pressed AFTER the keycodes in the first array.
#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = KEYBOARD) = {
        (usage_page = KEYBOARD, usage_min = 0xE0, usage_max = 0xE7) = {
            #[packed_bits 8] #[item_settings data,variable,absolute] modifier=input;
        };
        (usage_min = 0x00, usage_max = 0xFF) = {
            #[item_settings constant,variable,absolute] reserved=input;
        };
        (usage_page = KEYBOARD, usage_min = 0x00, usage_max = 0xDD) = {
            #[item_settings data,array,absolute] keycodes=input;
        };
    }
)]
pub struct NKROReport {
    pub modifier: u8,
    pub reserved: u8,
    pub keycodes: [u8; 42],
}

/* Physical Layout of Keeb:
 *
 *  Right side:
 * |  0  |  1  |  2  |  3  |  4 | 5  |
 * |  6  |  7  |  8  |  9  | 10 | 11 |
 * |  12 |  13 |  14 |  15 | 16 | 17 |
 * |  18 |  19 |  20 |
 *
 *
 *  Left side:
 * |  0  |  1  |  2  |  3  |  4 | 5  |
 * |  6  |  7  |  8  |  9  | 10 | 11 |
 * |  12 |  13 |  14 |  15 | 16 | 17 |
 *                 |  18 |  19 |  20 |
 */

pub struct KeebState {
    // each bit represents the current state of the physical switch
    // of the corresponding index. Since we have two separate boards,
    // each one gets its own bit-state field. Thus, we can have two
    // physical switches with the same ID (left physical switch #0 and
    // right physical switch #0)
    //
    // Ex: bit 0 of keys_left represents the state of physical switch 0
    // on the left-hand board, bit 20 represents the state of physical switch 20
    state: u32,
    pins: [DynPin; 21],
}

impl KeebState {
    pub fn new(pins: Pins) -> Self {
        let mut state = KeebState {
            state: 0,
            pins: [
                pins.gpio0.into(),
                pins.gpio1.into(),
                pins.gpio2.into(),
                pins.gpio3.into(),
                pins.gpio4.into(),
                pins.gpio5.into(),
                pins.gpio26.into(),
                pins.gpio22.into(),
                pins.gpio21.into(),
                pins.gpio20.into(),
                pins.gpio19.into(),
                pins.gpio18.into(),
                pins.gpio15.into(),
                pins.gpio14.into(),
                pins.gpio13.into(),
                pins.gpio12.into(),
                pins.gpio11.into(),
                pins.gpio10.into(),
                pins.gpio8.into(),
                pins.gpio7.into(),
                pins.gpio6.into(),
            ],
        };
        Self::setup_pins(&mut state);
        state
    }

    fn setup_pins(self: &mut Self) {
        for i in 0..21 {
            self.pins[i].into_pull_up_input();
        }
    }

    pub fn update_state(self: &mut Self) {
        for i in 0..21 {
            match self.pins[i].is_low() {
                Ok(_) => self.state |= 0b1 << i,
                Err(_) => self.state &= 0b0 << i,
            }
        }
    }

    pub fn get_switch_state(self: &Self, switch: u8) -> bool {
        let curr_state = self.state & (0b1 << switch);
        match curr_state {
            0 => false,
            _ => true,
        }
    }
}
