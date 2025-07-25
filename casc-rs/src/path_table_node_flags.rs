use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

/// Flags for nodes in the TVFS path table.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathTableNodeFlags(pub u32);

impl PathTableNodeFlags {
    pub const NONE: Self = Self(0x0000);
    pub const PATH_SEPARATOR_PRE: Self = Self(0x0001);
    pub const PATH_SEPARATOR_POST: Self = Self(0x0002);
    pub const IS_NODE_VALUE: Self = Self(0x0004);

    /// Checks if the flag is set.
    pub fn has_flag(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl BitOr for PathTableNodeFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        PathTableNodeFlags(self.0 | rhs.0)
    }
}

impl BitOrAssign for PathTableNodeFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for PathTableNodeFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        PathTableNodeFlags(self.0 & rhs.0)
    }
}

impl BitAndAssign for PathTableNodeFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitXor for PathTableNodeFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        PathTableNodeFlags(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for PathTableNodeFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Not for PathTableNodeFlags {
    type Output = Self;
    fn not(self) -> Self::Output {
        PathTableNodeFlags(!self.0)
    }
}
