#![no_std]
#![warn(missing_docs)]
#![feature(const_fn)]
#![feature(const_mut_refs)]

//! `gba-types` contains data types for interacting with the GBA's MMIO
//! registers.
//!
//! The most important types here are generated by the `bitstruct_newtype!`
//! macro:
//! * It's an opaque struct that wraps a primitive unsigned integer value
//! * Each "field" in the struct is bit-packed inside of the integer data.
//! * A field is often less than 8 bits, but could be more than 8 bits, and even
//!   could be exactly 8 bits.
//!   * Single-bit fields are bools.
//!   * Some multi-bit fields are integers within a reduced range. For example,
//!     a `Color` value has RGB channels, but only 5 bits per channel, so only
//!     the range 0 through 31 is allowed.
//!   * Some multi-bit fields are basically enums, but for increased FFI safety,
//!     we use a "const_enum" macro (see below) instead of actual Rust `enum`
//!     types.
//! * Each field has both a getter and a setter. Many MMIO locations aren't both
//!   readable and writable in all fields, but even so every struct has getters
//!   and setters for all fields just to make the in-memory manipulation of a
//!   value as easy as possible.
//!
//! This crate also contains some helper types that are considered to be a
//! `const_enum!` type:
//! * This is an opaque value that should only be one of a specific list of
//!   associated consts.
//! * Like with a normal enum, the bit pattern of the value isn't important,
//!   what's important is some semantic meaning other than the literal number
//!   value.
//! * Unlike with a normal enum, because this is a wrapped integer it's more FFI
//!   friendly. If an illegal bit pattern *does* somehow get read in from a
//!   register then it won't instantly cause UB.
//!
//! ## Nightly Only
//!
//! * This crate utilizes the [const_mut_refs](https://github.com/rust-lang/rust/issues/57349)
//!   nightly feature to allow both the setter functions to be const. This will
//!   probably become stable "sooner rather than later", particularly compared
//!   to some of the other nightly features that GBA programming is likely to
//!   use.

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
    $(
      $(#[$const_attrs:meta])*
      $c:ident($v:expr)
    ),+ $(,)?
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

macro_rules! phantom_field_get {
  // bools
  ($(#[$field_attrs:meta])* $inner:ty, $bit:literal : $g:ident, $s:ident) => {
    $(#[$field_attrs])*
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
    #[must_use]
    pub const fn $g(self) -> $nt {
      const MASK: $inner =
        ((((1_u64 << ($end + 1)) - 1) >> $start) << $start) as $inner;
      $nt(bit_get!(self.0, MASK) as $inner)
    }
  };
}

macro_rules! phantom_field_set {
  // bools
  ($inner:ty, $bit:literal : $g:ident, $s:ident) => {
    ///
    #[inline]
    pub const fn $s(&mut self, $g: bool) {
      *self =
        Self(bit_set!(self.0, 1 << $bit, ($g as usize) << $bit) as $inner);
    }
  };
  // raw ints
  ($inner:ty, $start:literal - $end:literal : $g:ident, $s:ident) => {
    ///
    #[inline]
    pub const fn $s(&mut self, $g: $inner) {
      const MASK: $inner =
        ((((1_u64 << ($end + 1)) - 1) >> $start) << $start) as $inner;
      *self = Self(bit_set!(self.0, MASK, $g << $start) as $inner);
    }
  };
  // newtype'd ints
  ($inner:ty, $start:literal - $end:literal => $nt:ident : $g:ident, $s:ident) => {
    ///
    #[inline]
    pub const fn $s(&mut self, $g: $nt) {
      const MASK: $inner =
        ((((1_u64 << ($end + 1)) - 1) >> $start) << $start) as $inner;
      *self = Self(bit_set!(self.0, MASK, $g.0) as $inner);
    }
  };
}

macro_rules! bitstruct_newtype {
  ($(#[$ty_attrs:meta])* $name:ident($inner:ty) {
    $(
      $(#[$field_attrs:meta])*
      [$($field_tokens:tt)*],
    )+ $(,)?
  }) => {
    $(#[$ty_attrs])*
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(transparent)]
    pub struct $name($inner);
    impl $name {
      $(phantom_field_get!($(#[$field_attrs])* $inner, $($field_tokens)*);)+
      $(phantom_field_set!(/*no attrs on the setter*/ $inner, $($field_tokens)*);)+
    }
  }
}

const_enum! {
  /// One of the six video modes available on the GBA.
  VideoMode(u16) {
    /// Mode 0: bg0, bg1, bg2, and bg3 are textual.
    _0(0),
    /// Mode 1: bg0 and bg1 are textual, bg2 is affine.
    _1(1),
    /// Mode 2: bg2 and bg3 are affine.
    _2(2),
    /// Mode 3: Single bitmap.
    _3(3),
    /// Mode 4: Two full-resolution indexmaps.
    _4(4),
    /// Mode 5: Two partial-resolution bitmaps.
    _5(5),
  }
}

bitstruct_newtype! {
  /// The Display Control register value.
  ///
  /// This sets the video mode, which affects almost all other aspects of the
  /// display, and also directly controls what layers are displayed or not.
  DisplayControlSetting(u16) {
    /// The PPU's video mode. More details are on the [`VideoMode`] type.
    [0-2 => VideoMode: video_mode, set_video_mode],

    /// Determines if Frame 0 or Frame 1 is shown when using video mode 4 or 5.
    /// Otherwise this has no effect.
    [4: show_frame1, set_show_frame1],

    /// This sets the OAM memory to be free for CPU editing during Hblank.
    ///
    /// The downside is that this gives the PPU less time for object display
    /// calculation, so the number of objects that can be displayed per line
    /// goes down slightly (by about 20%).
    [5: hblank_oam_free, set_hblank_oam_free],

    /// Determines if the object tile memory region should be "2d" or "1d" when
    /// displaying multi-tile objects.
    ///
    /// In 2d mode, the object video memory is considered to be a single 256x256
    /// area, so each row is 32 tiles offset. In 1d mode, the object video
    /// memory is considered more like a standard array of tiles.
    [6: obj_vram_is_1d, set_obj_vram_is_1d],

    /// If this bit is active, the ppu is blanked regardless of its actual position.
    ///
    /// While force-blank is on, the ppu only draws white pixels.
    [7: forced_blank, set_forced_blank],

    /// Display background 0. Only effective in video modes 0 and 1.
    [8: display_bg0, set_display_bg0],

    /// Display background 1. Only effective in video modes 0 and 1.
    [9: display_bg1, set_display_bg1],

    /// Display background 2. BG2 has some sort of content in all video modes.
    [10: display_bg2, set_display_bg2],

    /// Display background 3. Only effective in video modes 0 and 2.
    [11: display_bg3, set_display_bg3],

    /// Display the object layer. Available in all video modes.
    [12: display_obj, set_display_obj],

    /// Display window 0 content.
    [13: display_win0, set_display_win0],

    /// Display window 1 content.
    [14: display_win1, set_display_win1],

    /// Display object window content.
    [15: display_obj_win, set_display_obj_win],
  }
}

bitstruct_newtype! {
  /// The Display Status register value.
  ///
  /// This register combines info about the current status of the display/PPU as
  /// well as controlling if/when the display can generates interrupts.
  DisplayStatusSetting(u16) {
    /// Is the PPU currently in vertical blank?
    [0: is_vblank, set_is_vblank],

    /// Is the PPU currently in horizontal blank?
    [1: is_hblank, set_is_hblank],

    /// Is the current vcount a match with the vcount setting?
    [2: is_vcount_match, set_is_vcount_match],

    /// If set, the PPU fires an interrupt when vblank starts.
    [3: vblank_irq_enabled, set_vblank_irq_enabled],

    /// If set, the PPU fires an interrupt when hblank starts.
    [4: hblank_irq_enabled, set_hblank_irq_enabled],

    /// If set, the PPU fires an interrupt when the vcount matches the vcount setting.
    [5: vcount_match_irq_enabled, set_vcount_match_irq_enabled],

    /// This primarily lets you trigger an interrupt at a variable row position.
    [8-15: vcount_setting, set_vcount_setting],
  }
}

bitstruct_newtype! {
  BackgroundControlSetting(u16) {
    /// Lower priority draws "closer" to the top.
    ///
    /// In case of tie, you then sort by the BG's layer number:
    /// ```txt
    /// bg_z_dist = (bg_priority << 2) + bg_num;
    /// ```
    [0-1: background_priority, set_background_priority],
    [2-3: base_charblock, set_base_charblock],
    [6: use_mosaic, set_use_mosaic],
    [7: is_8bpp, set_is_8bpp],
    [8-12: base_screenblock, set_base_screenblock],
    [13: affine_overflow_wraparound, set_affine_overflow_wraparound],
    [14-15: screen_size, set_screen_size],
  }
}

bitstruct_newtype! {
  WindowContentSetting(u8) {
    [0: display_bg0, set_display_bg0],
    [1: display_bg1, set_display_bg1],
    [2: display_bg2, set_display_bg2],
    [3: display_bg3, set_display_bg3],
    [4: display_obj, set_display_obj],
    [5: display_special_effect, set_display_special_effect],
  }
}

bitstruct_newtype! {
  MosaicSetting(u8) {
    [0-3: horizontal_size, set_horizontal_size],
    [4-7: vertical_size, set_vertical_size],
  }
}

const_enum! {
  BlendEffect(u16) {
    NoEffect(0 << 6),
    AlphaBlend(1 << 6),
    BrightnessIncrease(2 << 6),
    BrightnessDecrease(3 << 6),
  }
}

bitstruct_newtype! {
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

bitstruct_newtype! {
  TextScreenEntry(u16) {
    [0-9: tile_id, set_tile_id],
    [10: horizontal_flip, set_horizontal_flip],
    [11: vertical_flip, set_vertical_flip],
    [12-15: palbank, set_palbank],
  }
}

bitstruct_newtype! {
  Color(u16) {
    [0-4: red, set_red],
    [5-9: green, set_green],
    [10-14: blue, set_blue],
  }
}

const_enum! {
  ObjDisplayMode(u16) {
    Normal(0b00 << 8),
    Affine(0b01 << 8),
    Disabled(0b10 << 8),
    DoubleSizeAffine(0b11 << 8),
  }
}

bitstruct_newtype! {
  ObjAttr0(u16) {
    [0-7: y_coordinate, set_y_coordinate],
    [8-9 => ObjDisplayMode: obj_display_mode, set_obj_display_mode],
    [10-11: obj_mode, set_obj_mode],
    [12: use_mosaic, set_use_mosaic],
    [13: is_8bpp, set_is_8bpp],
    [14-15: obj_shape, set_obj_shape],
  }
}

bitstruct_newtype! {
  ObjAttr1(u16) {
    [0-8: x_coordinate, set_x_coordinate],
    [9-13: affine_param, set_affine_param],
    [12: horizontal_flip, set_horizontal_flip],
    [13: vertical_flip, set_vertical_flip],
    [14-15: obj_size, set_obj_size],
  }
}

bitstruct_newtype! {
  ObjAttr2(u16) {
    [0-9: base_tile_id, set_base_tile_id],
    [10-11: priority, set_priority],
    [12-15: palbank, set_palbank],
  }
}

bitstruct_newtype! {
  ToneSweep(u8) {
    [0-2: sweep_shift_count, set_sweep_shift_count],
    [3: decreasing_sweep, set_decreasing_sweep],
    [4-6: sweep_time_chunk, set_sweep_time_chunk],
  }
}

const_enum! {
  ToneWavePatternDuty(u16) {
    _12_5(0 << 6),
    _25(1 << 6),
    _50(2 << 6),
    _75(3 << 6),
  }
}

bitstruct_newtype! {
  ToneDutyLenEnvelope(u16) {
    [0-5: sound_length, set_sound_length],
    [6-7 => ToneWavePatternDuty: decreasing_sweep, set_decreasing_sweep],
    [8-10: envelope_step_time, set_envelope_step_time],
    [11: increasing_envelope, set_increasing_envelope],
    [12-15: initial_volume, set_initial_volume],
  }
}

bitstruct_newtype! {
  ToneFrequencyControl(u16) {
    [0-10: frequency, set_frequency],
    [14: stop_at_end, set_stop_at_end],
    [15: init, set_init],
  }
}

bitstruct_newtype! {
  WaveRamSelect(u8) {
    [5: two_banks, set_two_banks],
    [6: using_bank1, set_using_bank1],
    [7: playing, set_playing],
  }
}

const_enum! {
  WaveVolume(u8) {
    _0(0 << 5),
    _100(1 << 5),
    _50(2 << 5),
    _25(3 << 5),
    _75(0b100 << 5),
  }
}

bitstruct_newtype! {
  WaveVolumeSetting(u8) {
    [5-7 => WaveVolume: volume, set_volume],
  }
}

bitstruct_newtype! {
  WaveFrequencyControl(u16) {
    [0-10: sample_rate, set_sample_rate],
    [14: stop_at_end, set_stop_at_end],
    [15: init, set_init],
  }
}

bitstruct_newtype! {
  NoiseLengthEnvelope(u16) {
    [0-5: length, set_length],
    [8-10: envelope_step_time, set_envelope_step_time],
    [11: increasing_envelope, set_increasing_envelope],
    [12-15: initial_volume, set_initial_volume],
  }
}

bitstruct_newtype! {
  NoiseFrequencyControl(u16) {
    /// 0 => 0.5 instead
    [0-2: dividing_ratio, set_dividing_ratio],
    [3: step_width_7bits, set_step_width_7bits],
    [4-7: shift_clock_frequency, set_shift_clock_frequency],
    [14: stop_at_end, set_stop_at_end],
    [15: init, set_init],
  }
}

bitstruct_newtype! {
  GeneratedSoundLeftRightMainVolume(u8) {
    [0-2: right, set_right],
    [4-6: left, set_left],
  }
}

bitstruct_newtype! {
  GeneratedSoundLeftRightEnabled(u8) {
    [0: right_sound_1_enabled, set_right_sound_1_enabled],
    [1: right_sound_2_enabled, set_right_sound_2_enabled],
    [2: right_sound_3_enabled, set_right_sound_3_enabled],
    [3: right_sound_4_enabled, set_right_sound_4_enabled],
    [4: left_sound_1_enabled, set_left_sound_1_enabled],
    [5: left_sound_2_enabled, set_left_sound_2_enabled],
    [6: left_sound_3_enabled, set_left_sound_3_enabled],
    [7: left_sound_4_enabled, set_left_sound_4_enabled],
  }
}

const_enum! {
  GeneratedSoundMixingVolume(u8) {
    _25(0),
    _50(1),
    _100(2),
  }
}

bitstruct_newtype! {
  DmaSoundMixVolumeControl(u8) {
    [0-1 => GeneratedSoundMixingVolume: generated_volume, set_generated_volume],
    [2: sound_a_full, set_sound_a_full],
    [3: sound_b_full, set_sound_b_full],
  }
}

bitstruct_newtype! {
  DmaSoundControlBits(u8) {
    [0: sound_a_right, set_sound_a_right],
    [1: sound_a_left, set_sound_a_left],
    [2: sound_a_timer1, set_sound_a_timer1],
    [3: sound_a_fifo_reset, set_sound_a_fifo_reset],
    [4: sound_b_right, set_sound_b_right],
    [5: sound_b_left, set_sound_b_left],
    [6: sound_b_timer1, set_sound_b_timer1],
    [7: sound_b_fifo_reset, set_sound_b_fifo_reset],
  }
}

// Note(Lokathor): PSG = Programmable Sound Generator

bitstruct_newtype! {
  GeneratedSoundActiveBits(u8) {
    [0: sound_1_active, set_sound_1_active],
    [1: sound_2_active, set_sound_2_active],
    [2: sound_3_active, set_sound_3_active],
    [3: sound_4_active, set_sound_4_active],
    [7: sound_enabled, set_sound_enabled],
  }
}

const_enum! {
  SoundBiasSamplingSetting(u16) {
    _9bit(0 << 14),
    _8bit(1 << 14),
    _7bit(2 << 14),
    _6bit(3 << 14),
  }
}

bitstruct_newtype! {
  SoundBiasSetting(u16) {
    [1-9: bias_level, set_bias_level],
    [14-15 => SoundBiasSamplingSetting: sampling_cycle, set_sampling_cycle],
  }
}

const_enum! {
  /// A scale factor that sets the base frequency of the timer.
  TimerScaleFactor(u8) {
    /// 16.78MHz, 59.59 ns period.
    _1(0),
    /// 262.21kHz, 3.815 μs period.
    _64(1),
    /// 65.536kHz, 15.26 μs period.
    _256(2),
    /// 16.384kHz, 61.04 μs period.
    _1024(3),
  }
}

bitstruct_newtype! {
  TimerControlSetting(u8) {
    /// The prescale scale factor that the timer will count at.
    [0-1 => TimerScaleFactor: scale_factor, set_scale_factor],
    /// Instead of incrementing the count with each prescaled clock cycle, will instead increment whenever the lower number timer overflows.
    /// Timer0 cannot make effective use of this feature because there is no lower number timer than it.
    [2: overflow_counting, set_overflow_counting],
    /// Set to 1 to generate an interrupt when this timer overflows.
    [6: interrupt_on_overflow, set_interrupt_on_overflow],
    /// Set to 1 to enable the timer. Will clear the timer when enabled.
    [7: enabled, set_enabled],
  }
}

/// DMA on the GBA is very different from most embedded devices. It does not run in parallel to the CPU, but rather
/// stops the CPU to preform the memory transfer. Dispite the fact that it stops the CPU, it is still generally faster
/// than having the CPU copy data by itself.
/// 
/// There are 4 DMA channels, DMA0, DMA1, DMA2, and DMA3.
/// 
/// DMA0 is the highest priority channel and will always complete its job before any other channel. This is ideal for
/// time critical operations, such as copying data to a horizontal scanline. It has the restriction of only being able
/// to access internal memory, so it cannot access the game pak.
/// 
/// DMA1 and DMA2 are intended to be used for feeding sound data into the audio FIFOs.
/// 
/// DMA3 is special because it is capable of writing to game pak ROM/FlashROM, but is unable to write to game pak SRAM.
/// 
/// However it should be noted that these are just the intended use of the DMAs. You can use them for other purposes
/// as you see fit.
mod dma {
  const_enum! {
    /// Destination control settings.
    DmaDestinationAddressControl(u16) {
      /// Increment the address with each copy.
      Increment(0 << 5),
      /// Decrement the address with each copy.
      Decrement(1 << 5),
      /// Do not move.
      Fixed(2 << 5),
      /// Reloads the original value after the DMA completes.
      IncrementReload(3 << 5),
    }
  }
  
  const_enum! {
    /// Source control settings.
    DmaSourceAddressControl(u16) {
      /// Increment the address with each copy.
      Increment(0),
      /// Decrement the address with each copy.
      Decrement(1),
      /// Do not move.
      Fixed(2),
    }
  }
  
  const_enum! {
    /// Which event to trigger the DMA on.
    DmaStartTiming(u16) {
      /// Starts the DMA as soon as you set enabled to true.
      Immediate(0),
      /// Start the DMA on a vblank interrupt.
      Vblank(1),
      /// Start the DMA on an hblank interrupt.
      Hblank(2),
      /// Start time depends on the DMA used.
      ///
      /// DMA0: prohibited. Do not use.
      /// DMA1/2: Sound FIFO
      /// DMA3: Video Capture
      /// ## Safety
      /// * This value is prohibited for DMA0
      Special(3),
    }
  }
  
  bitstruct_newtype! {
    /// use to control a DMA channel.
    DmaControlSetting(u16) {
      /// Settings for how to treat the destination address.
      [5-6 => DmaDestinationAddressControl: dst_addr_control, set_dst_addr_control],
      /// Settings for how to treat the source address.
      [7-8 => DmaSourceAddressControl: src_addr_control, set_src_addr_control],
      /// If cleared to 0, then the enabled bit (15) will be cleared as well when DMA is complete.
      /// If set to 1, then the enable bit will remain set and the DMA will repeat when its start event happens again.
      [9: repeating, set_repeating],
      /// Set to 1 to preform a transfer at 32bits at a time. Clear to 0 to transfer 16bits at a time.
      [10: transfer32, set_transfer32],
      /// Set the event to trigger the DMA.
      [12-13 => DmaStartTiming: start_timing, set_start_timing],
      /// Set to 1 to trigger an interrupt when complete.
      [14: interrupt_when_complete, set_interrupt_when_complete],
      /// Set to 1 to enable.
      [15: enabled, set_enabled],
    }
  }
}

bitstruct_newtype! {
  /// Indicates which buttons are pressed. A button with a value of 0 is pressed, and a value of 1 is released.
  /// It is recommended that you check these during vblank interrupts. It's an effective way to filter out bounce.
  KeyInputLowActive(u16) {
    // A button.
    [0: a_released, set_a_released],
    // B button.
    [1: b_released, set_b_released],
    // Select button.
    [2: select_released, set_select_released],
    // Start button.
    [3: start_released, set_start_released],
    // Right direction button.
    [4: right_released, set_right_released],
    // Left direction button.
    [5: left_released, set_left_released],
    // Up direction button.
    [6: up_released, set_up_released],
    // Down direction button.
    [7: down_released, set_down_released],
    // Right bumper.
    [8: r_released, set_r_released],
    // Left bumper.
    [9: l_released, set_l_released],
  }
}

bitstruct_newtype! {
  /// Is used for handling keypad interrupts. This is not a good way to handle key input while a game is running. It is recommended you use
  /// simple polling from within the VBlank interrupt handler to do that.
  /// 
  /// The intention of this interrupt is to wake the GBA from very low power stop mode.
  /// 
  /// Setting a feild to 1 will enable that key to trigger the interrupt.
  KeyInterruptBits(u16) {
    /// Enable the A button.
    [0: a_selected, set_a_selected],
    /// Enable the B button.
    [1: b_selected, set_b_selected],
    /// Enable the select button.
    [2: select_selected, set_select_selected],
    /// Enable the start button.
    [3: start_selected, set_start_selected],
    /// Enable the right directional pad button.
    [4: right_selected, set_right_selected],
    /// Enable the left directional pad button.
    [5: left_selected, set_left_selected],
    /// Enable the up directional pad button.
    [6: up_selected, set_up_selected],
    /// Enable the down directional pad button.
    [7: down_selected, set_down_selected],
    /// Enable the right bumper button.
    [8: r_selected, set_r_selected],
    /// Enable the left bumper button.
    [9: l_selected, set_l_selected],
    /// Enable key interrupts.
    [14: key_interrupts_enabled, set_key_interrupts_enabled],
    /// When set to 1, all enbabled buttons must be pressed to trigger the interrupt.
    /// When set to 0, pressing any enabled button will trigger the interrupt.
    [15: interrupt_requires_all_bits, set_interrupt_requires_all_bits],
  }
}

bitstruct_newtype! {
  /// A bag of bits for working with interrupts. Used in more than one register.
  ///
  /// * When used with `IE`, the set bits determine the interrupts that will be accepted. Each interrupt source must be separately configured
  ///   to also send the appropriate interrupt via other registers.
  /// * When used with `IF`:
  ///   * Read the register to see what interrupts are currently pending.
  ///   * Write the register with active bits to clear a pending interrupt. It's slightly confusing that you write a 1-bit to change a
  ///     1-bit to a 0-bit, but that's really how it works.
  InterruptFlagBits(u16) {
    /// Vertical Blank Interrupt.
    [0: vblank, set_vblank],
    /// Horizontal Blank Interrupt.
    [1: hblank, set_hblank],
    /// Vertical count match Interrupt.
    [2: vcount_match, set_vcount_match],
    /// Timer 0 overflow.
    [3: timer0, set_timer0],
    /// Timer 1 overflow.
    [4: timer1, set_timer1],
    /// timer 2 overflow.
    [5: timer2, set_timer2],
    /// Timer 3 overflow.
    [6: timer3, set_timer3],
    /// Serial Communication
    [7: serial_communication, set_serial_communication],
    /// Direct Memory Access channel 0
    [8: dma0, set_dma0],
    /// Direct Memory Access channel 1
    [9: dma1, set_dma1],
    /// Direct Memory Access channel 2
    [10: dma2, set_dma2],
    /// Direct Memory Access channel 3
    [11: dma3, set_dma3],
    /// Keypad
    [12: keypad, set_keypad],
    /// This interrupt is generated by hardware inside the game pak itself, such as a co-processor, camera, or other hardware.
    [13: game_pak, set_game_pak],
  }
}

const_enum! {
  /// Valid wait cycle settings for the SRAM of the game pak.
  SramWaitControlCycles(u16) {
    _4(0),
    _3(1),
    _2(2),
    _8(3),
  }
}

const_enum! {
  /// Valid wait cycle settings for wait state 0.
  Rom0WaitControlCycles(u16) {
    _4(0 << 2),
    _3(1 << 2),
    _2(2 << 2),
    _8(3 << 2),
  }
}

const_enum! {
  /// Valid wait cycle settings for wait state 1.
  Rom1WaitControlCycles(u16) {
    _4(0 << 5),
    _3(1 << 5),
    _2(2 << 5),
    _8(3 << 5),
  }
}

const_enum! {
  /// Valid wait cycle settings for wait state 2.
  Rom2WaitControlCycles(u16) {
    _4(0 << 8),
    _3(1 << 8),
    _2(2 << 8),
    _8(3 << 8),
  }
}

const_enum! {
  /// Valid settings for the phy terminal output speed.
  PhiTerminalOutput(u16) {
    Disabled(0 << 11),
    _4_19MHz(1 << 11),
    _8_38MHz(2 << 11),
    _16_78MHz(3 << 11),
  }
}

bitstruct_newtype! {
  /// Waitstate control register value
  /// 
  /// This controls the access timing to the addresses 0x08000000, 0x0A000000, and 0x0C000000.
  /// Address 0x08000000 is known as wait state 0, 0x0A000000 as wait state 1, and 0x0C000000 as wait state 2.
  /// All of these addresses mirror content of the game pak ROM.
  WaitControlSetting(u16) {
    /// SRAM wait control
    [0-1 => SramWaitControlCycles: sram_wait, set_sram_wait],
    /// Wait State 0 First Access timing.
    [2-3 => Rom0WaitControlCycles: wait0_first_access, set_wait0_first_access],
    /// Wait State 0 Second Access timing.
    [4: wait0_second_access_1cycle, set_wait0_second_access_1cycle],
    /// Wait State 1 First Access timing.
    [5-6 => Rom1WaitControlCycles: wait1_first_access, set_wait1_first_access],
    /// Wait State 1 Second Access timing.
    [7: wait1_second_access_1cycle, set_wait1_second_access_1cycle],
    /// Wait State 2 First Access timing.
    [8-9 => Rom2WaitControlCycles: wait2_first_access, set_wait2_first_access],
    /// Wait State 2 Second Access timing.
    [10: wait2_second_access_1cycle, set_wait2_second_access_1cycle],
    /// PHI Terminal Output speed. Usage documentation is unclear. Is likely for strange Nintendo peripherals.
    [11-12 => PhiTerminalOutput: phi_terminal, set_phi_terminal],
    /// Game Pak Prefetch Buffer enable. When enabled, the GBA will attempt to fetch the next CPU instruction
    /// before it is needed, thus brining the wait down to zero cycles. This is of course, dependent on the bus
    /// being free at some point long enough to do this.
    [14: game_pak_prefetch_enabled, set_game_pak_prefetch_enabled],
  }
}
