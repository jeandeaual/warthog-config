use bitflags::bitflags;
use std::fmt;

// Found using lsusb or Zadig
pub const VID: u16 = 0x044f;
// pub const JOYSTICK_PID: u16 = 0x0402;
pub const THROTTLE_PID: u16 = 0x0404;

bitflags! {
    pub struct ThrottleLEDState: u8 {
        const BACKLIGHT = 0b000_1000;
        const LED_1     = 0b000_0100;
        const LED_2     = 0b000_0010;
        const LED_3     = 0b001_0000;
        const LED_4     = 0b000_0001;
        const LED_5     = 0b100_0000;
    }
}

/// By default, only the backlight is turned on.
impl Default for ThrottleLEDState {
    fn default() -> Self {
        Self::BACKLIGHT
    }
}

impl fmt::Display for ThrottleLEDState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut leds = Vec::new();

        if self.contains(Self::BACKLIGHT) {
            leds.push("Backlight".to_string());
        }
        if self.contains(Self::LED_1) {
            leds.push("1".to_string());
        }
        if self.contains(Self::LED_2) {
            leds.push("2".to_string());
        }
        if self.contains(Self::LED_3) {
            leds.push("3".to_string());
        }
        if self.contains(Self::LED_4) {
            leds.push("4".to_string());
        }
        if self.contains(Self::LED_5) {
            leds.push("5".to_string());
        }

        if leds.is_empty() {
            write!(f, "None")
        } else {
            write!(f, "{}", leds.join(", "))
        }
    }
}

impl From<u8> for ThrottleLEDState {
    #[inline]
    fn from(item: u8) -> Self {
        Self { bits: item }
    }
}

impl From<ThrottleLEDState> for u8 {
    #[inline]
    fn from(item: ThrottleLEDState) -> Self {
        item.bits
    }
}

impl std::ops::BitOr<u8> for ThrottleLEDState {
    type Output = u8;

    /// Returns the union of the two sets of flags.
    #[inline]
    fn bitor(self, rhs: u8) -> Self::Output {
        self.bits | rhs
    }
}

impl std::ops::BitOrAssign<u8> for ThrottleLEDState {
    /// Adds the set of flags.
    #[inline]
    fn bitor_assign(&mut self, rhs: u8) {
        self.bits |= rhs;
    }
}

impl std::ops::BitXor<u8> for ThrottleLEDState {
    type Output = u8;

    /// Returns the left flags, but with all the right flags toggled.
    #[inline]
    fn bitxor(self, rhs: u8) -> Self::Output {
        self.bits ^ rhs
    }
}

impl std::ops::BitXorAssign<u8> for ThrottleLEDState {
    /// Toggles the set of flags.
    #[inline]
    fn bitxor_assign(&mut self, rhs: u8) {
        self.bits ^= rhs;
    }
}

impl std::ops::BitAnd<u8> for ThrottleLEDState {
    type Output = u8;

    /// Returns the intersection between the two sets of flags.
    #[inline]
    fn bitand(self, rhs: u8) -> Self::Output {
        self.bits & rhs
    }
}

impl std::ops::BitAndAssign<u8> for ThrottleLEDState {
    /// Disables all flags disabled in the set.
    #[inline]
    fn bitand_assign(&mut self, rhs: u8) {
        self.bits &= rhs;
    }
}

impl std::ops::Sub<u8> for ThrottleLEDState {
    type Output = u8;

    /// Returns the set difference of the two sets of flags.
    #[inline]
    fn sub(self, rhs: u8) -> Self::Output {
        self.bits & !rhs
    }
}

impl std::ops::SubAssign<u8> for ThrottleLEDState {
    /// Disables all flags enabled in the set.
    #[inline]
    fn sub_assign(&mut self, rhs: u8) {
        self.bits &= !rhs;
    }
}
