pub const DEFAULT_DEBOUNCE_MS: u64 = 50;
pub const DEFAULT_DOUBLE_TAP_MS: u64 = 300;
pub const DEFAULT_LONG_HOLD_MS: u64 = 800;

#[derive(Debug, Clone, Copy)]
pub struct ButtonTiming {
    pub debounce_ms: u64,
    pub double_tap_ms: u64,
    pub long_hold_ms: u64,
}

impl Default for ButtonTiming {
    fn default() -> Self {
        Self {
            debounce_ms: DEFAULT_DEBOUNCE_MS,
            double_tap_ms: DEFAULT_DOUBLE_TAP_MS,
            long_hold_ms: DEFAULT_LONG_HOLD_MS,
        }
    }
}
