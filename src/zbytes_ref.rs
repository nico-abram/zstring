use super::ZBytesCreationError;
use core::{marker::PhantomData, ptr::NonNull, slice};

/// Borrows a non-null **const** pointer to zero-termianted bytes.
///
/// The bytes have no enforced encoding.
///
/// Because this is a "thin" pointer it's suitable for direct use with FFI.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ZBytesRef<'a> {
  pub(crate) nn: NonNull<u8>,
  pub(crate) marker: PhantomData<&'a [u8]>,
}
impl_zbytes_fmt!(
  ZBytesRef<'a>: Binary, Debug, Display, LowerExp, LowerHex, Octal, UpperExp, UpperHex
);
impl<'a> core::fmt::Pointer for ZBytesRef<'a> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "ZBytesRef({:p})", self.nn.as_ptr())
  }
}
impl<'a> TryFrom<&'a [u8]> for ZBytesRef<'a> {
  type Error = ZBytesCreationError;

  #[inline]
  fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
    match value.split_last() {
      None => Err(ZBytesCreationError::NullTerminatorMissing),
      Some((terminator, data)) => {
        if terminator != &0 {
          Err(ZBytesCreationError::NullTerminatorMissing)
        } else if data.iter().any(|b| b == &0) {
          Err(ZBytesCreationError::InteriorNull)
        } else {
          Ok(Self {
            nn: unsafe { NonNull::new_unchecked(value.as_ptr() as _) },
            marker: PhantomData,
          })
        }
      }
    }
  }
}
impl<'a> TryFrom<&'a str> for ZBytesRef<'a> {
  type Error = ZBytesCreationError;

  #[inline]
  fn try_from(value: &'a str) -> Result<Self, Self::Error> {
    Self::try_from(value.as_bytes())
  }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for ZBytesRef<'a> {
  type Error = ZBytesCreationError;

  #[inline]
  fn try_from(value: &'a [u8; N]) -> Result<Self, Self::Error> {
    Self::try_from(value.as_ref())
  }
}

impl<'a> ZBytesRef<'a> {
  /// Turns a `NonNull` into a `ZBytesRef`
  ///
  /// ## Safety
  /// * The NonNull must point to a series of bytes that is null-terminated.
  #[inline]
  #[must_use]
  pub const unsafe fn from_non_null_unchecked(nn: NonNull<u8>) -> Self {
    Self { nn, marker: PhantomData }
  }

  /// Gets an iterator over the bytes.
  #[inline]
  #[must_use]
  pub const fn iter(&self) -> ZBytesRefIter<'a> {
    ZBytesRefIter { nn: self.nn, marker: PhantomData }
  }

  /// Gets the full slice this points to, including the null byte.
  ///
  /// **Caution:** This takes linear time to compute the slice!
  #[must_use]
  pub fn as_slice_including_null(&self) -> &'a [u8] {
    let mut count = 1;
    let mut p = self.nn.as_ptr();
    while unsafe { *p } != 0 {
      count += 1;
      p = unsafe { p.add(1) };
    }
    unsafe { slice::from_raw_parts(self.nn.as_ptr(), count) }
  }
}

/// Iterator over a [ZBytesRef]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ZBytesRefIter<'a> {
  nn: NonNull<u8>,
  marker: PhantomData<&'a [u8]>,
}
impl<'a> Iterator for ZBytesRefIter<'a> {
  type Item = &'a u8;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    match unsafe { self.nn.as_ref() } {
      0 => None,
      other => {
        self.nn = unsafe { NonNull::new_unchecked(self.nn.as_ptr().add(1)) };
        Some(other)
      }
    }
  }
}

#[test]
fn zbytes_basics() {
  let _ = ZBytesRef::try_from("hello\0").unwrap();
  let _ = ZBytesRef::try_from(b"hello\0").unwrap();
  let _ = ZBytesRef::try_from(b"hello\0".as_ref()).unwrap();

  assert_eq!(format!("{}", ZBytesRef::try_from(" A\0").unwrap()), "[32, 65]");
}