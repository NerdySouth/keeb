use heapless::{FnvIndexMap, IndexMap};
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
 * TODO: Give ascii art representation of the physical
 * layout, with the switch ID's in their proper places corresponding
 * to their respective GPIO pin numbers when possible
 *
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
    keys_left: u32,
    keys_right: u32,
    // maps physical keys (GPIO pins) to an array of keycodes. We use an array,
    // because we can map multiple values to the same key, and switch between
    // them using 'layers'
    //
    // We make this big enough for 64 keys b/c the heapless crate requires that
    // the size of an IndexMap be a power of two, otherwise it could have been
    // smaller.
    key_map: FnvIndexMap<u8, [u8; 3], 64>,
}

impl KeebState {
    pub fn new() -> Self {
        KeebState {
            keys_left: 0,
            keys_right: 0,
            key_map: FnvIndexMap::<u8, [u8; 3], 64>::new(),
        }
    }
}
