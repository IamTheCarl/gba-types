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

macro_rules! bitstruct_newtype {
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

bitstruct_newtype! {
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

bitstruct_newtype! {
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

bitstruct_newtype! {
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
    NoEffect(0),
    AlphaBlend(1),
    BrightnessIncrease(2),
    BrightnessDecrease(3),
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
    Normal(0b00),
    Affine(0b01),
    Disabled(0b10),
    DoubleSizeAffine(0b11),
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
    _12_5(0),
    _25(1),
    _50(2),
    _75(3),
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
    _0(0),
    _100(1),
    _50(2),
    _25(3),
    _75(0b100),
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
    _9bit(0),
    _8bit(1),
    _7bit(2),
    _6bit(3),
  }
}

bitstruct_newtype! {
  SoundBiasSetting(u16) {
    [1-9: bias_level, set_bias_level],
    [14-15 => SoundBiasSamplingSetting: sampling_cycle, set_sampling_cycle],
  }
}

const_enum! {
  TimerScaleFactor(u8) {
    _1(0),
    _64(1),
    _256(2),
    _1024(3),
  }
}

bitstruct_newtype! {
  TimerControlSetting(u8) {
    [0-1 => TimerScaleFactor: scale_factor, set_scale_factor],
    [2: overflow_counting, set_overflow_counting],
    [6: interrupt_on_overflow, set_interrupt_on_overflow],
    [7: enabled, set_enabled],
  }
}

const_enum! {
  DmaDestinationAddressControl(u16) {
    Increment(0),
    Decrement(1),
    Fixed(2),
    IncrementReload(3),
  }
}

const_enum! {
  DmaSourceAddressControl(u16) {
    Increment(0),
    Decrement(1),
    Fixed(2),
  }
}

const_enum! {
  DmaStartTiming(u16) {
    Immediate(0),
    Vblank(1),
    Hblank(2),
    /// Start time depends on the DMA used.
    ///
    /// ## Safety
    /// * This value is prohibited for DMA0
    Special(3),
  }
}

bitstruct_newtype! {
  DmaControlSetting(u16) {
    [5-6 => DmaDestinationAddressControl: dst_addr_control, set_dst_addr_control],
    [7-8 => DmaSourceAddressControl: src_addr_control, set_src_addr_control],
    [9: repeating, set_repeating],
    [10: transfer32, set_transfer32],
    [12-13: start_timing, set_start_timing],
    [14: interrupt_when_complete, set_interrupt_when_complete],
    [15: enabled, set_enabled],
  }
}

bitstruct_newtype! {
  KeyInputLowActive(u16) {
    [0: a_released, set_a_released],
    [1: b_released, set_b_released],
    [2: select_released, set_select_released],
    [3: start_released, set_start_released],
    [4: right_released, set_right_released],
    [5: left_released, set_left_released],
    [6: up_released, set_up_released],
    [7: down_released, set_down_released],
    [8: r_released, set_r_released],
    [9: l_released, set_l_released],
  }
}

bitstruct_newtype! {
  KeyInterruptBits(u16) {
    [0: a_selected, set_a_selected],
    [1: b_selected, set_b_selected],
    [2: select_selected, set_select_selected],
    [3: start_selected, set_start_selected],
    [4: right_selected, set_right_selected],
    [5: left_selected, set_left_selected],
    [6: up_selected, set_up_selected],
    [7: down_selected, set_down_selected],
    [8: r_selected, set_r_selected],
    [9: l_selected, set_l_selected],
    [14: key_interrupts_enabled, set_key_interrupts_enabled],
    [15: interrupt_requires_all_bits, set_interrupt_requires_all_bits],
  }
}

bitstruct_newtype! {
  InterruptFlagBits(u16) {
    [0: vblank, set_vblank],
    [1: hblank, set_hblank],
    [2: vcount_match, set_vcount_match],
    [3: timer0, set_timer0],
    [4: timer1, set_timer1],
    [5: timer2, set_timer2],
    [6: timer3, set_timer3],
    [7: serial_communication, set_serial_communication],
    [8: dma0, set_dma0],
    [9: dma1, set_dma1],
    [10: dma2, set_dma2],
    [11: dma3, set_dma3],
    [12: keypad, set_keypad],
    [13: game_pak, set_game_pak],
  }
}

const_enum! {
  WaitControlCycles(u16) {
    _4(0),
    _3(1),
    _2(2),
    _8(3),
  }
}

const_enum! {
  PhiTerminalOutput(u16) {
    Disabled(0),
    _4_19MHz(1),
    _8_38MHz(2),
    _16_78MHz(3),
  }
}

bitstruct_newtype! {
  WaitControlSetting(u16) {
    [0-1 => WaitControlCycles: sram_wait, set_sram_wait],
    [2-3 => WaitControlCycles: wait0_first_access, set_wait0_first_access],
    [4: wait0_second_access_1cycle, set_wait0_second_access_1cycle],
    [5-6 => WaitControlCycles: wait1_first_access, set_wait1_first_access],
    [7: wait1_second_access_1cycle, set_wait1_second_access_1cycle],
    [8-9 => WaitControlCycles: wait2_first_access, set_wait2_first_access],
    [10: wait2_second_access_1cycle, set_wait2_second_access_1cycle],
    [11-12 => PhiTerminalOutput: phi_terminal, set_phi_terminal],
    [14: game_pak_prefetch_enabled, set_game_pak_prefetch_enabled],
  }
}
