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
