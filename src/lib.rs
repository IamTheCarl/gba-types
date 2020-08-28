#![no_std]
//#![warn(missing_docs)]
#![feature(const_fn)]
#![feature(const_mut_refs)]

macro_rules! bit_get {
  ($val:expr, $mask:expr) => {
    // we do bit ops in `usize` because it sometimes optimizes better
    ($val as usize) & ($mask as usize)
  };
}

macro_rules! bit_set {
  ($val:expr, $mask:expr, $new:expr) => {{
    // we do bit ops in `usize` because it sometimes optimizes better
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
    #[allow(non_upper_case_globals)]
    impl $name {
      $( $(#[$const_attrs])* pub const $c: $name = $name($v); )+
    }
  }
}

macro_rules! phantom_field {
  // bools
  ($(#[$field_attrs:meta])* $inner:ty, $bit:literal : $g:ident, $s:ident) => {
    $(#[$field_attrs])*
    #[inline]
    pub const fn $s(&mut self, $g: bool) {
      *self =
        Self(bit_set!(self.0, 1 << $bit, ($g as usize) << $bit) as $inner);
    }
    #[allow(missing_docs)]
    #[inline]
    #[must_use]
    pub const fn $g(self) -> bool {
      bit_get!(self.0, 1 << $bit) != 0
    }
  };
  // raw ints
  ($(#[$field_attrs:meta])* $inner:ty, $start:literal - $end:literal : $g:ident, $s:ident) => {
    $(#[$field_attrs])*
    #[inline]
    pub const fn $s(&mut self, $g: $inner) {
      const MASK: $inner =
        ((((1_u64 << ($end + 1)) - 1) >> $start) << $start) as $inner;
      *self = Self(bit_set!(self.0, MASK, $g << $start) as $inner);
    }
    #[allow(missing_docs)]
    #[inline]
    #[must_use]
    pub const fn $g(self) -> $inner {
      const MASK: $inner =
        ((((1_u64 << ($end + 1)) - 1) >> $start) << $start) as $inner;
      (bit_get!(self.0, MASK) >> $start) as $inner
    }
  };
  // newtype'd ints
  ($(#[$field_attrs:meta])* $inner:ty, $start:literal - $end:literal => $nt:ident : $g:ident, $s:ident) => {
    $(#[$field_attrs])*
    #[inline]
    pub const fn $s(&mut self, $g: $nt) {
      const MASK: $inner =
        ((((1_u64 << ($end + 1)) - 1) >> $start) << $start) as $inner;
      *self = Self(bit_set!(self.0, MASK, $g.0) as $inner);
    }
    #[allow(missing_docs)]
    #[inline]
    #[must_use]
    pub const fn $g(self) -> $nt {
      const MASK: $inner =
        ((((1_u64 << ($end + 1)) - 1) >> $start) << $start) as $inner;
      $nt(bit_get!(self.0, MASK) as $inner)
    }
  };
}

macro_rules! register_newtype {
  ($(#[$ty_attrs:meta])* $name:ident($inner:ty) {
    $($(#[$field_attrs:meta])* [$($field_tokens:tt)*],)+
    $(,)*
  }) => {
    $(#[$ty_attrs])*
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(transparent)]
    pub struct $name($inner);
    impl $name {
      $(phantom_field!($(#[$field_attrs])* $inner, $($field_tokens)*);)+
    }
  }
}

const_enum! {
  /// One of the six video modes available on the GBA.
  VideoMode(u16) {
    /// Mode 0: bg0, bg1, bg2, and bg3 are textual.
    _0(0),
    /// Mode 1: bg0 and bg1 are textual, bg2 is affine
    _1(1),
    /// Mode 2: bg2 and bg3 are affine.
    _2(2),
    /// Mode 3: Single bitmap
    _3(3),
    /// Mode 4: Two full-resolution indexmaps.
    _4(4),
    /// Mode 5: Two partial-resolution bitmaps.
    _5(5),
  }
}

register_newtype! {
  DisplayControlSetting(u16) {
    [0-2 => VideoMode: video_mode, set_video_mode],
    [4: show_frame1, set_show_frame1],
    [5: hblank_oam_free, set_hblank_oam_free],
    [6: obj_vram_is_1d, set_obj_vram_is_1d],
    [7: forced_blank, set_forced_blank],
    [8: display_bg0, set_display_bg0],
    [9: display_bg1, set_display_bg1],
    [10: display_bg2, set_display_bg2],
    [11: display_bg3, set_display_bg3],
    [12: display_obj, set_display_obj],
    [13: display_win0, set_display_win0],
    [14: display_win1, set_display_win1],
    [15: display_obj_win, set_display_obj_win],
  }
}

register_newtype! {
  DisplayStatusSetting(u16) {
    [0: is_vblank, set_is_vblank],
    [1: is_hblank, set_is_hblank],
    [2: is_vcount_match, set_is_vcount_match],
    [3: vblank_irq_enabled, set_vblank_irq_enabled],
    [4: hblank_irq_enabled, set_hblank_irq_enabled],
    [5: vcount_match_irq_enabled, set_vcount_match_irq_enabled],
    [8-15: vcount_setting, set_vcount_setting],
  }
}

register_newtype! {
  BackgroundControlSetting(u16) {
    /// lower priority draws "closer" to the top. in case of tie, go by the bg#
    [0-1: background_priority, set_background_priority],
    [2-3: base_charblock, set_base_charblock],
    [6: use_mosaic, set_use_mosaic],
    [7: is_8bpp, set_is_8bpp],
    [8-12: base_screenblock, set_base_screenblock],
    [13: affine_overflow_wraparound, set_affine_overflow_wraparound],
    [14-15: screen_size, set_screen_size],
  }
}

register_newtype! {
  WindowContentSetting(u8) {
    [0: display_bg0, set_display_bg0],
    [1: display_bg1, set_display_bg1],
    [2: display_bg2, set_display_bg2],
    [3: display_bg3, set_display_bg3],
    [4: display_obj, set_display_obj],
    [5: display_special_effect, set_display_special_effect],
  }
}

register_newtype! {
  MosaicSetting(u8) {
    [0-3: horizontal_size, set_horizontal_size],
    [4-7: vertical_size, set_vertical_size],
  }
}

const_enum! {
  BlendEffect(u16) {
    NoEffect(0),
    AlphaBlend(1),
    BrightnessIncrease(2),
    BrightnessDecrease(3),
  }
}

register_newtype! {
  ColorBlendControlSetting(u16) {
    [0: first_target_bg0, set_first_target_bg0],
    [1: first_target_bg1, set_first_target_bg1],
    [2: first_target_bg2, set_first_target_bg2],
    [3: first_target_bg3, set_first_target_bg3],
    [4: first_target_obj, set_first_target_obj],
    [5: first_target_backdrop, set_first_target_backdrop],
    [6-7 => BlendEffect: blend_effect, set_blend_effect],
    [8: second_target_bg0, set_second_target_bg0],
    [9: second_target_bg1, set_second_target_bg1],
    [10: second_target_bg2, set_second_target_bg2],
    [11: second_target_bg3, set_second_target_bg3],
    [12: second_target_obj, set_second_target_obj],
    [13: second_target_backdrop, set_second_target_backdrop],
  }
}

register_newtype! {
  TextScreenEntry(u16) {
    [0-9: tile_id, set_tile_id],
    [10: horizontal_flip, set_horizontal_flip],
    [11: vertical_flip, set_vertical_flip],
    [12-15: palbank, set_palbank],
  }
}

register_newtype! {
  Color(u16) {
    [0-4: red, set_red],
    [5-9: green, set_green],
    [10-14: blue, set_blue],
  }
}

const_enum! {
  ObjDisplayMode(u16) {
    Normal(0b00),
    Affine(0b01),
    Disabled(0b10),
    DoubleSizeAffine(0b11),
  }
}

register_newtype! {
  ObjAttr0(u16) {
    [0-7: y_coordinate, set_y_coordinate],
    [8-9 => ObjDisplayMode: obj_display_mode, set_obj_display_mode],
    [10-11: obj_mode, set_obj_mode],
    [12: use_mosaic, set_use_mosaic],
    [13: is_8bpp, set_is_8bpp],
    [14-15: obj_shape, set_obj_shape],
  }
}

register_newtype! {
  ObjAttr1(u16) {
    [0-8: x_coordinate, set_x_coordinate],
    [9-13: affine_param, set_affine_param],
    [12: horizontal_flip, set_horizontal_flip],
    [13: vertical_flip, set_vertical_flip],
    [14-15: obj_size, set_obj_size],
  }
}

register_newtype! {
  ObjAttr2(u16) {
    [0-9: base_tile_id, set_base_tile_id],
    [10-11: priority, set_priority],
    [12-15: palbank, set_palbank],
  }
}

// TODO: continue work with the sound types
