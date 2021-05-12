use std::cmp::Ordering;
use std::ops;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Byte(u8);

impl Byte {
    pub const fn new(n: u8) -> Self {
        Self(n)
    }

    pub fn u8(&self) -> u8 {
        self.0
    }

    pub fn nth(&self, n: u8) -> u8 {
        self.0.wrapping_shr(n as u32) & 1
    }
}

impl From<u8> for Byte {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<Byte> for u8 {
    fn from(value: Byte) -> Self {
        value.0 as Self
    }
}

impl From<Byte> for u16 {
    fn from(value: Byte) -> Self {
        value.0 as Self
    }
}

impl Into<i8> for Byte {
    fn into(self) -> i8 {
        self.0 as i8
    }
}

impl Into<i16> for Byte {
    fn into(self) -> i16 {
        self.0 as i16
    }
}

impl Into<i32> for Byte {
    fn into(self) -> i32 {
        self.0 as i32
    }
}

impl Into<i64> for Byte {
    fn into(self) -> i64 {
        self.0 as i64
    }
}

impl ops::Add for Byte {
    type Output = Self;

    fn add(self, Self(rhs): Byte) -> Byte {
        Self(self.0.wrapping_add(rhs))
    }
}

impl ops::Add<u8> for Byte {
    type Output = Self;

    fn add(self, rhs: u8) -> Byte {
        Self(self.0.wrapping_add(rhs))
    }
}

impl ops::AddAssign<u8> for Byte {
    fn add_assign(&mut self, other: u8) {
        *self = Self(self.0.wrapping_add(other))
    }
}

impl ops::Sub for Byte {
    type Output = Self;

    fn sub(self, Self(rhs): Byte) -> Byte {
        Self(self.0.wrapping_sub(rhs))
    }
}

impl ops::Sub<u8> for Byte {
    type Output = Self;

    fn sub(self, rhs: u8) -> Byte {
        Self(self.0.wrapping_sub(rhs))
    }
}

impl ops::SubAssign<u8> for Byte {
    fn sub_assign(&mut self, other: u8) {
        *self = Self(self.0.wrapping_sub(other))
    }
}

impl ops::Mul for Byte {
    type Output = Self;

    fn mul(self, Self(rhs): Self) -> Self {
        Self(self.0.wrapping_mul(rhs))
    }
}

impl ops::Mul<u8> for Byte {
    type Output = Self;

    fn mul(self, rhs: u8) -> Self {
        Self(self.0.wrapping_mul(rhs))
    }
}

impl PartialOrd for Byte {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl ops::BitAnd for Byte {
    type Output = Self;

    fn bitand(self, Self(rhs): Self) -> Self::Output {
        Self(self.0 & rhs)
    }
}

impl ops::BitAnd<u8> for Byte {
    type Output = Self;

    fn bitand(self, rhs: u8) -> Self::Output {
        Self(self.0 & rhs)
    }
}

impl ops::BitAndAssign for Byte {
    fn bitand_assign(&mut self, Self(rhs): Self) {
        *self = Self(self.0 & rhs)
    }
}

impl ops::BitAndAssign<u8> for Byte {
    fn bitand_assign(&mut self, rhs: u8) {
        *self = Self(self.0 & rhs)
    }
}

impl ops::BitOr for Byte {
    type Output = Self;

    fn bitor(self, Self(rhs): Self) -> Self::Output {
        Self(self.0 | rhs)
    }
}

impl ops::BitOr<u8> for Byte {
    type Output = Self;

    fn bitor(self, rhs: u8) -> Self::Output {
        Self(self.0 | rhs)
    }
}

impl ops::BitOrAssign for Byte {
    fn bitor_assign(&mut self, Self(rhs): Self) {
        *self = Self(self.0 | rhs)
    }
}

impl ops::BitOrAssign<u8> for Byte {
    fn bitor_assign(&mut self, rhs: u8) {
        *self = Self(self.0 | rhs)
    }
}

impl ops::BitXor for Byte {
    type Output = Self;

    fn bitxor(self, Self(rhs): Self) -> Self::Output {
        Self(self.0 ^ rhs)
    }
}

impl ops::BitXor<u8> for Byte {
    type Output = Self;

    fn bitxor(self, rhs: u8) -> Self::Output {
        Self(self.0 ^ rhs)
    }
}

impl ops::BitXorAssign for Byte {
    fn bitxor_assign(&mut self, Self(rhs): Self) {
        *self = Self(self.0 ^ rhs)
    }
}

impl ops::Not for Byte {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl ops::Shl<u8> for Byte {
    type Output = Self;

    fn shl(self, rhs: u8) -> Self::Output {
        Self(self.0 << rhs)
    }
}

impl ops::ShlAssign<u8> for Byte {
    fn shl_assign(&mut self, rhs: u8) {
        *self = Self(self.0 << rhs)
    }
}

impl ops::Shr<u8> for Byte {
    type Output = Self;

    fn shr(self, rhs: u8) -> Self::Output {
        Self(self.0 >> rhs)
    }
}

impl ops::ShrAssign<u8> for Byte {
    fn shr_assign(&mut self, rhs: u8) {
        *self = Self(self.0 >> rhs)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Word(u16);

impl From<u8> for Word {
    fn from(value: u8) -> Self {
        Self(value as u16)
    }
}

impl From<u16> for Word {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<i32> for Word {
    fn from(value: i32) -> Self {
        Self(value as u16)
    }
}

impl From<Word> for u16 {
    fn from(value: Word) -> Self {
        value.0
    }
}

impl From<Byte> for Word {
    fn from(Byte(value): Byte) -> Self {
        Self(value as u16)
    }
}

impl Into<i16> for Word {
    fn into(self) -> i16 {
        self.0 as i16
    }
}

impl Into<i32> for Word {
    fn into(self) -> i32 {
        self.0 as i32
    }
}

impl Into<i64> for Word {
    fn into(self) -> i64 {
        self.0 as i64
    }
}

impl Word {
    pub const fn new(n: u16) -> Self {
        Self(n)
    }

    pub fn byte(&self) -> Byte {
        Byte(self.0 as u8)
    }

    pub fn nth(&self, n: u8) -> u16 {
        self.0.wrapping_shr(n as u32) & 1
    }
}

impl ops::Add for Word {
    type Output = Self;

    fn add(self, Self(rhs): Self) -> Word {
        Self(self.0.wrapping_add(rhs))
    }
}

impl ops::Add<u16> for Word {
    type Output = Self;

    fn add(self, rhs: u16) -> Word {
        Self(self.0.wrapping_add(rhs))
    }
}

impl ops::Add<Byte> for Word {
    type Output = Self;

    fn add(self, Byte(rhs): Byte) -> Self {
        Self(self.0.wrapping_add(rhs.into()))
    }
}

impl ops::AddAssign for Word {
    fn add_assign(&mut self, Self(other): Self) {
        *self = Self(self.0.wrapping_add(other))
    }
}

impl ops::AddAssign<u16> for Word {
    fn add_assign(&mut self, other: u16) {
        *self = Self(self.0.wrapping_add(other))
    }
}

impl ops::Sub for Word {
    type Output = Self;

    fn sub(self, Self(rhs): Self) -> Self::Output {
        Self(self.0.wrapping_sub(rhs))
    }
}

impl ops::Sub<u16> for Word {
    type Output = Self;

    fn sub(self, rhs: u16) -> Self::Output {
        Self(self.0.wrapping_sub(rhs))
    }
}

impl ops::Shr<u16> for Word {
    type Output = Self;

    fn shr(self, rhs: u16) -> Self::Output {
        Self(self.0 >> rhs)
    }
}

impl ops::Mul<u16> for Word {
    type Output = Self;

    fn mul(self, rhs: u16) -> Self {
        Self(self.0.wrapping_mul(rhs))
    }
}

impl ops::Shl<u16> for Word {
    type Output = Self;

    fn shl(self, rhs: u16) -> Self::Output {
        Self(self.0 << rhs)
    }
}

impl ops::ShlAssign<u16> for Word {
    fn shl_assign(&mut self, rhs: u16) {
        *self = Self(self.0 << rhs)
    }
}

impl ops::BitAnd<u16> for Word {
    type Output = Self;

    fn bitand(self, rhs: u16) -> Self::Output {
        Self(self.0 & rhs)
    }
}

impl ops::BitOr for Word {
    type Output = Self;

    fn bitor(self, Self(rhs): Word) -> Self::Output {
        Self(self.0 | rhs)
    }
}

impl ops::BitOr<u16> for Word {
    type Output = Self;

    fn bitor(self, rhs: u16) -> Self::Output {
        Self(self.0 | rhs)
    }
}

impl ops::BitXor<u16> for Word {
    type Output = Self;

    fn bitxor(self, rhs: u16) -> Self::Output {
        Self(self.0 ^ rhs)
    }
}
