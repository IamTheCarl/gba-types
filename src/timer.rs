//! There are 4 timers in the GBA, each one has a control register and a
//! counter register.
//!
//! The counter registers are simply u16 fields, so we do not bother to create
//! a special representation of them. Reading from these counter registers will
//! give you the current count of how many cycles the base frequency has
//! cycled. Writing to this register will not immediately change its value, but
//! rather will set the value to be loaded into it the next time the timer is
//! enabled/re-enabled.

const_enum! {
  /// A scale factor that sets the base frequency of the timer.
  /// Divide the clock frequency of the processor (16.78MHz) by the selected scale factor to get the output scale.
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
    /// The pre-scale scale factor that the timer will count at.
    [0-1 => TimerScaleFactor: scale_factor, set_scale_factor],
    /// Instead of incrementing the count with each pre-scaled clock cycle, will instead increment whenever the lower number timer overflows.
    /// Timer0 cannot make effective use of this feature because there is no lower number timer than it.
    [2: overflow_counting, set_overflow_counting],
    /// Set to 1 to generate an interrupt when this timer overflows.
    [6: interrupt_on_overflow, set_interrupt_on_overflow],
    /// Set to 1 to enable the timer. Will clear the timer when enabled.
    [7: enabled, set_enabled],
  }
}
