//! DMA on the GBA is very different from most embedded devices. It does not run
//! in parallel to the CPU, but rather stops the CPU to preform the memory
//! transfer. Dispite the fact that it stops the CPU, it is still generally
//! faster than having the CPU copy data by itself.
//!
//! There are 4 DMA channels, DMA0, DMA1, DMA2, and DMA3.
//!
//! DMA0 is the highest priority channel and will always complete its job before
//! any other channel. This is ideal for time critical operations, such as
//! copying data to a horizontal scanline. It has the restriction of only being
//! able to access internal memory, so it cannot access the game pak.
//!
//! DMA1 and DMA2 are intended to be used for feeding sound data into the audio
//! FIFOs.
//!
//! DMA3 is special because it is capable of writing to game pak ROM/FlashROM,
//! but is unable to write to game pak SRAM.
//!
//! The DMA channels do not need to be used for these exact purposes, you can
//! generally do whatever you want with them as long as it falls within their
//! address constraints.

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
    /// Starts the DMA 2 cycles after setting the enable bit in the control
    /// register. The processor will be halted during these two cycles, so you
    /// don't need to worry about messing up the DMA settings.
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
