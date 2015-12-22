// from https://raw.githubusercontent.com/BurntSushi/byteorder/master/src/lib.rs

use core::mem::transmute;
use core::ptr::copy_nonoverlapping;

#[inline]
fn extend_sign(val: u64, nbytes: usize) -> i64 {
    let shift = (8 - nbytes) * 8;
    (val << shift) as i64 >> shift
}

#[inline]
fn unextend_sign(val: i64, nbytes: usize) -> u64 {
    let shift = (8 - nbytes) * 8;
    (val << shift) as u64 >> shift
}

#[inline]
fn pack_size(n: u64) -> usize {
    if n < 1 << 8 {
        1
    } else if n < 1 << 16 {
        2
    } else if n < 1 << 24 {
        3
    } else if n < 1 << 32 {
        4
    } else if n < 1 << 40 {
        5
    } else if n < 1 << 48 {
        6
    } else if n < 1 << 56 {
        7
    } else {
        8
    }
}

/// ByteOrder describes types that can serialize integers as bytes.
///
/// Note that `Self` does not appear anywhere in this trait's definition!
/// Therefore, in order to use it, you'll need to use syntax like
/// `T::read_u16(&[0, 1])` where `T` implements `ByteOrder`.
///
/// This crate provides two types that implement `ByteOrder`: `BigEndian`
/// and `LittleEndian`.
///
/// # Examples
///
/// Write and read `u32` numbers in little endian order:
///
/// ```rust
/// use byteorder::{ByteOrder, LittleEndian};
///
/// let mut buf = [0; 4];
/// LittleEndian::write_u32(&mut buf, 1_000_000);
/// assert_eq!(1_000_000, LittleEndian::read_u32(&buf));
/// ```
///
/// Write and read `i16` numbers in big endian order:
///
/// ```rust
/// use byteorder::{ByteOrder, BigEndian};
///
/// let mut buf = [0; 2];
/// BigEndian::write_i16(&mut buf, -50_000);
/// assert_eq!(-50_000, BigEndian::read_i16(&buf));
/// ```
pub trait ByteOrder {
    /// Reads an unsigned 16 bit integer from `buf`.
    ///
    /// Panics when `buf.len() < 2`.
    fn read_u16(buf: &[u8]) -> u16;

    /// Reads an unsigned 32 bit integer from `buf`.
    ///
    /// Panics when `buf.len() < 4`.
    fn read_u32(buf: &[u8]) -> u32;

    /// Reads an unsigned 64 bit integer from `buf`.
    ///
    /// Panics when `buf.len() < 8`.
    fn read_u64(buf: &[u8]) -> u64;

    /// Reads an unsigned n-bytes integer from `buf`.
    ///
    /// Panics when `nbytes < 1` or `nbytes > 8` or
    /// `buf.len() < nbytes`
    fn read_uint(buf: &[u8], nbytes: usize) -> u64;

    /// Writes an unsigned 16 bit integer `n` to `buf`.
    ///
    /// Panics when `buf.len() < 2`.
    fn write_u16(buf: &mut [u8], n: u16);

    /// Writes an unsigned 32 bit integer `n` to `buf`.
    ///
    /// Panics when `buf.len() < 4`.
    fn write_u32(buf: &mut [u8], n: u32);

    /// Writes an unsigned 64 bit integer `n` to `buf`.
    ///
    /// Panics when `buf.len() < 8`.
    fn write_u64(buf: &mut [u8], n: u64);

    /// Writes an unsigned integer `n` to `buf` using only `nbytes`.
    ///
    /// If `n` is not representable in `nbytes`, or if `nbytes` is `> 8`, then
    /// this method panics.
    fn write_uint(buf: &mut [u8], n: u64, nbytes: usize);

    /// Reads a signed 16 bit integer from `buf`.
    ///
    /// Panics when `buf.len() < 2`.
    #[inline]
    fn read_i16(buf: &[u8]) -> i16 {
        Self::read_u16(buf) as i16
    }

    /// Reads a signed 32 bit integer from `buf`.
    ///
    /// Panics when `buf.len() < 4`.
    #[inline]
    fn read_i32(buf: &[u8]) -> i32 {
        Self::read_u32(buf) as i32
    }

    /// Reads a signed 64 bit integer from `buf`.
    ///
    /// Panics when `buf.len() < 8`.
    #[inline]
    fn read_i64(buf: &[u8]) -> i64 {
        Self::read_u64(buf) as i64
    }

    /// Reads a signed n-bytes integer from `buf`.
    ///
    /// Panics when `nbytes < 1` or `nbytes > 8` or
    /// `buf.len() < nbytes`
    #[inline]
    fn read_int(buf: &[u8], nbytes: usize) -> i64 {
        extend_sign(Self::read_uint(buf, nbytes), nbytes)
    }

    /// Reads a IEEE754 single-precision (4 bytes) floating point number.
    ///
    /// Panics when `buf.len() < 4`.
    #[inline]
    fn read_f32(buf: &[u8]) -> f32 {
        unsafe { transmute(Self::read_u32(buf)) }
    }

    /// Reads a IEEE754 double-precision (8 bytes) floating point number.
    ///
    /// Panics when `buf.len() < 8`.
    #[inline]
    fn read_f64(buf: &[u8]) -> f64 {
        unsafe { transmute(Self::read_u64(buf)) }
    }

    /// Writes a signed 16 bit integer `n` to `buf`.
    ///
    /// Panics when `buf.len() < 2`.
    #[inline]
    fn write_i16(buf: &mut [u8], n: i16) {
        Self::write_u16(buf, n as u16)
    }

    /// Writes a signed 32 bit integer `n` to `buf`.
    ///
    /// Panics when `buf.len() < 4`.
    #[inline]
    fn write_i32(buf: &mut [u8], n: i32) {
        Self::write_u32(buf, n as u32)
    }

    /// Writes a signed 64 bit integer `n` to `buf`.
    ///
    /// Panics when `buf.len() < 8`.
    #[inline]
    fn write_i64(buf: &mut [u8], n: i64) {
        Self::write_u64(buf, n as u64)
    }

    /// Writes a signed integer `n` to `buf` using only `nbytes`.
    ///
    /// If `n` is not representable in `nbytes`, or if `nbytes` is `> 8`, then
    /// this method panics.
    #[inline]
    fn write_int(buf: &mut [u8], n: i64, nbytes: usize) {
        Self::write_uint(buf, unextend_sign(n, nbytes), nbytes)
    }

    /// Writes a IEEE754 single-precision (4 bytes) floating point number.
    ///
    /// Panics when `buf.len() < 4`.
    #[inline]
    fn write_f32(buf: &mut [u8], n: f32) {
        Self::write_u32(buf, unsafe { transmute(n) })
    }

    /// Writes a IEEE754 double-precision (8 bytes) floating point number.
    ///
    /// Panics when `buf.len() < 8`.
    #[inline]
    fn write_f64(buf: &mut [u8], n: f64) {
        Self::write_u64(buf, unsafe { transmute(n) })
    }
}

/// Defines big-endian serialization.
///
/// Note that this type has no value constructor. It is used purely at the
/// type level.
#[allow(missing_copy_implementations)] pub enum BigEndian {}

/// Defines little-endian serialization.
///
/// Note that this type has no value constructor. It is used purely at the
/// type level.
#[allow(missing_copy_implementations)] pub enum LittleEndian {}

/// Defines system native-endian serialization.
///
/// Note that this type has no value constructor. It is used purely at the
/// type level.
#[cfg(target_endian = "little")]
pub type NativeEndian = LittleEndian;

/// Defines system native-endian serialization.
///
/// Note that this type has no value constructor. It is used purely at the
/// type level.
#[cfg(target_endian = "big")]
pub type NativeEndian = BigEndian;

macro_rules! read_num_bytes {
    ($ty:ty, $size:expr, $src:expr, $which:ident) => ({
        assert!($size <= $src.len());
        unsafe {
            (*($src.as_ptr() as *const $ty)).$which()
        }
    });
}

macro_rules! write_num_bytes {
    ($ty:ty, $size:expr, $n:expr, $dst:expr, $which:ident) => ({
        assert!($size <= $dst.len());
        unsafe {
            // N.B. https://github.com/rust-lang/rust/issues/22776
            let bytes = transmute::<_, [u8; $size]>($n.$which());
            copy_nonoverlapping((&bytes).as_ptr(), $dst.as_mut_ptr(), $size);
        }
    });
}

impl ByteOrder for BigEndian {
    #[inline]
    fn read_u16(buf: &[u8]) -> u16 {
        read_num_bytes!(u16, 2, buf, to_be)
    }

    #[inline]
    fn read_u32(buf: &[u8]) -> u32 {
        read_num_bytes!(u32, 4, buf, to_be)
    }

    #[inline]
    fn read_u64(buf: &[u8]) -> u64 {
        read_num_bytes!(u64, 8, buf, to_be)
    }

    #[inline]
    fn read_uint(buf: &[u8], nbytes: usize) -> u64 {
        assert!(1 <= nbytes && nbytes <= 8 && nbytes <= buf.len());
        let mut out = [0u8; 8];
        let ptr_out = out.as_mut_ptr();
        unsafe {
            copy_nonoverlapping(
                buf.as_ptr(), ptr_out.offset((8 - nbytes) as isize), nbytes);
            (*(ptr_out as *const u64)).to_be()
        }
    }

    #[inline]
    fn write_u16(buf: &mut [u8], n: u16) {
        write_num_bytes!(u16, 2, n, buf, to_be);
    }

    #[inline]
    fn write_u32(buf: &mut [u8], n: u32) {
        write_num_bytes!(u32, 4, n, buf, to_be);
    }

    #[inline]
    fn write_u64(buf: &mut [u8], n: u64) {
        write_num_bytes!(u64, 8, n, buf, to_be);
    }

    #[inline]
    fn write_uint(buf: &mut [u8], n: u64, nbytes: usize) {
        assert!(pack_size(n) <= nbytes && nbytes <= 8);
        assert!(nbytes <= buf.len());
        unsafe {
            let bytes: [u8; 8] = transmute(n.to_be());
            copy_nonoverlapping(
                bytes.as_ptr().offset((8 - nbytes) as isize),
                buf.as_mut_ptr(),
                nbytes);
        }
    }
}

impl ByteOrder for LittleEndian {
    #[inline]
    fn read_u16(buf: &[u8]) -> u16 {
        read_num_bytes!(u16, 2, buf, to_le)
    }

    #[inline]
    fn read_u32(buf: &[u8]) -> u32 {
        read_num_bytes!(u32, 4, buf, to_le)
    }

    #[inline]
    fn read_u64(buf: &[u8]) -> u64 {
        read_num_bytes!(u64, 8, buf, to_le)
    }

    #[inline]
    fn read_uint(buf: &[u8], nbytes: usize) -> u64 {
        assert!(1 <= nbytes && nbytes <= 8 && nbytes <= buf.len());
        let mut out = [0u8; 8];
        let ptr_out = out.as_mut_ptr();
        unsafe {
            copy_nonoverlapping(buf.as_ptr(), ptr_out, nbytes);
            (*(ptr_out as *const u64)).to_le()
        }
    }

    #[inline]
    fn write_u16(buf: &mut [u8], n: u16) {
        write_num_bytes!(u16, 2, n, buf, to_le);
    }

    #[inline]
    fn write_u32(buf: &mut [u8], n: u32) {
        write_num_bytes!(u32, 4, n, buf, to_le);
    }

    #[inline]
    fn write_u64(buf: &mut [u8], n: u64) {
        write_num_bytes!(u64, 8, n, buf, to_le);
    }

    #[inline]
    fn write_uint(buf: &mut [u8], n: u64, nbytes: usize) {
        assert!(pack_size(n as u64) <= nbytes && nbytes <= 8);
        assert!(nbytes <= buf.len());
        unsafe {
            let bytes: [u8; 8] = transmute(n.to_le());
            copy_nonoverlapping(bytes.as_ptr(), buf.as_mut_ptr(), nbytes);
        }
    }
}
