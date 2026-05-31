use std::ops::BitOr;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Bmp8Flags(u8);

impl Bmp8Flags {
    pub const RAW_BMP_UNALIGNED: Self = Self(1 << 0);
    pub const DIB_BITMAP: Self = Self(1 << 1);
    pub const SPLICED: Self = Self(1 << 2);
}

impl BitOr for Bmp8Flags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}
