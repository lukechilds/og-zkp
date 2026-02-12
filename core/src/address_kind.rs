#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum AddressKind {
    P2pkh = 0,
    P2sh = 1,
    P2wpkh = 2,
    P2tr = 3,
}

impl TryFrom<i32> for AddressKind {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::P2pkh),
            1 => Ok(Self::P2sh),
            2 => Ok(Self::P2wpkh),
            3 => Ok(Self::P2tr),
            _ => Err("invalid address type"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_valid_values() {
        let variants = [
            (0, AddressKind::P2pkh),
            (1, AddressKind::P2sh),
            (2, AddressKind::P2wpkh),
            (3, AddressKind::P2tr),
        ];
        for (raw, expected) in variants {
            // Forward: i32 → AddressKind
            assert_eq!(AddressKind::try_from(raw), Ok(expected));
            // Round-trip: variant → as i32 → try_from
            assert_eq!(AddressKind::try_from(expected as i32), Ok(expected));
        }
    }

    #[test]
    fn try_from_invalid() {
        for val in [-1, 4, 99] {
            assert!(AddressKind::try_from(val).is_err());
        }
    }
}
