//#![no_std]
#![feature(const_fn)]
#![feature(const_mut_refs)]

#[allow(unused_macros)]
macro_rules! bit_get {
  ($val:expr, $mask:expr) => {
    // we do bit ops in `usize` because it optimizes better
    ($val as usize) & ($mask as usize)
  };
}

#[allow(unused_macros)]
macro_rules! bit_set {
  ($val:expr, $mask:expr, $new:expr) => {{
    // we do bit ops in `usize` because it optimizes better
    let a = $val as usize;
    let b = $new as usize;
    let mask = $mask as usize;
    a ^ ((a ^ b) & mask)
  }};
}

/// Declares a newtype with a private field and a series of named constants.
///
/// This is far more FFI safe than a rust `enum` type.
macro_rules! const_enum {
  ($(#[$ty_attrs:meta])* $name:ident($inner:ty) {
    $($(#[$const_attrs:meta])* $c:ident($v:expr)),+ $(,)*
  }) => {
    $(#[$ty_attrs])*
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(transparent)]
    pub struct $name($inner);
    impl $name {
      $( $(#[$const_attrs])* pub const $c: $name = $name($v); )+
    }
  }
}

macro_rules! phantom_field {
  // bools
  ($inner:ty, $bit:literal : $g:ident, $s:ident) => {
    pub const fn $g(self) -> bool {
      bit_get!(self.0, 1 << $bit) != 0
    }
    pub const fn $s(&mut self, $g: bool) {
      *self =
        Self(bit_set!(self.0, 1 << $bit, ($g as usize) << $bit) as $inner);
    }
  };
  // raw ints
  ($inner:ty, $start:literal - $end:literal : $g:ident, $s:ident) => {
    pub const fn $g(self) -> $inner {
      const MASK: $inner =
        ((((1_u64 << ($end + 1)) - 1) >> $start) << $start) as $inner;
      (bit_get!(self.0, MASK) >> $start) as $inner
    }
    pub const fn $s(&mut self, $g: $inner) {
      const MASK: $inner =
        ((((1_u64 << ($end + 1)) - 1) >> $start) << $start) as $inner;
      *self = Self(bit_set!(self.0, MASK, $g << $start) as $inner);
    }
  };
  // newtyped ints
  ($inner:ty, $start:literal - $end:literal => $nt:ident : $g:ident, $s:ident) => {
    pub const fn $g(self) -> $nt {
      const MASK: $inner =
        ((((1_u64 << ($end + 1)) - 1) >> $start) << $start) as $inner;
      $nt(bit_get!(self.0, MASK) as $inner)
    }
    pub const fn $s(&mut self, $g: $nt) {
      const MASK: $inner =
        ((((1_u64 << ($end + 1)) - 1) >> $start) << $start) as $inner;
      *self = Self(bit_set!(self.0, MASK, $g.0) as $inner);
    }
  };
}

macro_rules! register_newtype {
  ($(#[$ty_attrs:meta])* $name:ident($inner:ty) {
    $([$($field_tokens:tt)*],)+
    $(,)*
  }) => {
    $(#[$ty_attrs])*
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(transparent)]
    pub struct $name($inner);
    impl $name {
      $(phantom_field!($inner, $($field_tokens)*);)+
    }
  }
}

const_enum! {
  /// doc
  VideoMode(u16) {
    /// field doc
    _0(0),
    _1(1),
    _2(2),
    _3(3),
    _4(4),
    _5(5),
  }
}

register_newtype! {
  /// doc
  DisplayControlSetting(u16) {
    [1-3 => VideoMode: video_mode, set_video_mode],
    [4-5: vcount_setting, set_vcount_setting],
    [6: hblank_interval_oam_free, set_hblank_interval_oam_free],
  }
}

#[test]
fn test_1() {
  let mut dcs = DisplayControlSetting(0);

  // check bool
  assert!(!dcs.hblank_interval_oam_free());
  dcs.set_hblank_interval_oam_free(true);
  std::eprintln!("0b{:016b}", dcs.0);
  assert!(dcs.hblank_interval_oam_free());
  assert_eq!(dcs.0, 0b1_00_000_0);

  // check raw int
  assert_eq!(dcs.vcount_setting(), 0);
  dcs.set_vcount_setting(0b1111);
  std::eprintln!("0b{:016b}", dcs.0);
  assert_eq!(dcs.vcount_setting(), 0b11);
  assert_eq!(dcs.0, 0b1_11_000_0);

  // check newtype int
  assert_eq!(dcs.video_mode(), VideoMode(0));
  dcs.set_video_mode(VideoMode(u16::MAX));
  std::eprintln!("0b{:016b}", dcs.0);
  assert_eq!(dcs.video_mode(), VideoMode(0b111_0));
  assert_eq!(dcs.0, 0b1_11_111_0);
}
