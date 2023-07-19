extern crate keyberon;
use keyberon::key_code::KeyCode;

/// This is our custom report descriptor. It defines a report with a packed byte
/// for the modifier keys, a single reserved byte of 0's, then the 6-byte array
/// of keycodes used for boot-compliant drivers, followed by a bitpacked
pub const NKRO_REPORT_DESCRIPTOR: &[u8] = &[
    0x05, 0x01, // Usage Page (Generic Desktop),
    0x09, 0x06, // Usage (Keyboard),
    0xA1, 0x01, // Collection (Application),
    0x85, 0x04, //   Report ID (4),
    // hybrid of modifiers
    0x75, 0x01, //   Report Size (1),
    0x95, 0x08, //   Report Count (8),
    0x05, 0x07, //   Usage Page (Key Codes),
    0x19, 0xE0, //   Usage Minimum (224),
    0x29, 0xE7, //   Usage Maximum (231),
    0x15, 0x00, //   Logical Minimum (0),
    0x25, 0x01, //   Logical Maximum (1),
    0x81, 0x02, //   Input (Data, Variable, Absolute), ;Modifier byte
    // Padding / fake boot keyboard
    0x95, 0x38, //   Report Count (56),
    0x75, 0x01, //   Report Size (1),
    0x81, 0x01, //   Input (Const,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
    // hybrid of keys
    0x95, 0x68, //  Report Count (104),
    0x75, 0x01, //  Report Size (1),
    0x15, 0x00, //  Logical Minimum (0),
    0x25, 0x01, //  Logical Maximum(1),
    0x05, 0x07, //  Usage Page (Key Codes),
    0x19, 0x00, //  Usage Minimum (0),
    0x29, 0x68, //   Usage Maximum (104),
    0x81, 0x02, //  Input (Data, Variable, Absolute),
    0xc0, //  End Collection
];

/// Struct representing our custom report descriptor.
/// The first byte is a bitfield of modifiers, followed by a
/// padding byte, and 6 bytes for BOOT protocol scancodes. A BIOS/UEFI
/// system will either properly parse our report descriptor and treat
/// the 'boot' scancode array as padding, or it will ignore our report
/// descriptor and read the first 8 bytes of our report as if it follows
/// the BOOT protocol. This allows us to have NKRO behavior once an OS
/// boots with a full USB HID implementation, but still be able to use
/// the keyboard during boot for BIOS/UEFI systems that do not properly
/// or fully implement the HID specification.
///
/// If HID is properly implemented (like in linux or OSX), then the host
/// will skip the reserved padding and boot array, and only use our
/// NKRO bitmap. This bitmap represents the first 104 keys defined by
/// the HID usage table for keyboards. This is enough for most people in
/// the US, and definitely enough for my personal use.
#[repr(C)]
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct NKROReport([u8; 21]);

impl core::iter::FromIterator<KeyCode> for NKROReport {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = KeyCode>,
    {
        let mut res = Self::default();
        for kc in iter {
            res.pressed(kc);
        }
        res
    }
}

impl NKROReport {
    /// Returns the report as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Add the given key code to the report. This will mainly
    /// modify the last 13 bytes of the NKROReport, which is our bitmap
    /// of keycodes (From 0 - 104 in the HID Keyboard usage table),
    /// however, it will also update the modifer bitmap, and the BOOT
    /// protocol array that is within the first 8 bytes of the report.
    /// This is so that the keyboard still works during boot with buggy
    /// BIOS/UEFI implementations.
    pub fn pressed(&mut self, kc: KeyCode) {
        use KeyCode::*;
        match kc {
            No => (),
            ErrorRollOver | PostFail | ErrorUndefined => self.set_all(kc),
            kc if kc.is_modifier() => self.0[0] |= kc.as_modifier_bit(),
            _ => {
                // handle boot scancode array first
                self.0[2..]
                    .iter_mut()
                    .find(|c| **c == 0)
                    .map(|c| *c = kc as u8)
                    .unwrap_or_else(|| self.set_all(ErrorRollOver));

                // handle the NKRO bitmap
                let bits = &mut self.0[8..];
                let kc_bit = kc as u8;
                match kc_bit {
                    0..=3 => (),
                    4..=7 => {
                        bits[0] |= 1 << kc_bit;
                    }
                    8..=15 => {
                        bits[1] |= 1 << (kc_bit - 8);
                    }
                    16..=23 => {
                        bits[2] |= 1 << (kc_bit - 16);
                    }
                    24..=31 => {
                        bits[3] |= 1 << (kc_bit - 24);
                    }
                    32..=39 => {
                        bits[4] |= 1 << (kc_bit - 32);
                    }
                    40..=47 => {
                        bits[5] |= 1 << (kc_bit - 40);
                    }
                    48..=55 => {
                        bits[6] |= 1 << (kc_bit - 48);
                    }
                    56..=63 => {
                        bits[7] |= 1 << (kc_bit - 56);
                    }
                    64..=71 => {
                        bits[8] |= 1 << (kc_bit - 64);
                    }
                    72..=79 => {
                        bits[9] |= 1 << (kc_bit - 72);
                    }
                    80..=87 => {
                        bits[10] |= 1 << (kc_bit - 80);
                    }
                    88..=95 => {
                        bits[11] |= 1 << (kc_bit - 88);
                    }
                    96..=103 => {
                        bits[12] |= 1 << (kc_bit - 96);
                    }
                    _ => (),
                }
            }
        }
    }

    fn set_all(&mut self, kc: KeyCode) {
        // set all within BOOT array
        // Since we cant roll-over, or get PostFail outside
        // of a buggy HID impl (BIOS/UEFI), we wont worry
        // about needing to set those within out bitmap
        let boot = &mut self.0[2..8];
        for c in boot {
            *c = kc as u8;
        }
    }
}
