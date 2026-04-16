//! Figure/icon constants matching src/constants/figures.ts

// Platform-aware: on Windows use â— (U+25CF), elsewhere âº (U+23FA)
pub fn black_circle() -> &'static str {
    if cfg!(target_os = "windows") { "â—" } else { "âº" }
}
pub const BULLET_OPERATOR: &str = "âˆ™";       // U+2219
pub const TEARDROP_ASTERISK: &str = "âœ»";     // U+273B - used for thinking/compact
pub const UP_ARROW: &str = "â†‘";              // U+2191
pub const DOWN_ARROW: &str = "â†“";            // U+2193
pub const LIGHTNING_BOLT: &str = "â†¯";        // U+21AF - fast mode
pub const EFFORT_LOW: &str = "â—‹";            // U+25CB
pub const EFFORT_MEDIUM: &str = "â—";         // U+25D0
pub const EFFORT_HIGH: &str = "â—";           // U+25CF
pub const EFFORT_MAX: &str = "â—‰";            // U+25C9
pub const PLAY_ICON: &str = "—¶";             // U+25B6
pub const PAUSE_ICON: &str = "â¸";            // U+23F8
pub const REFRESH_ARROW: &str = "â†»";         // U+21BB
pub const FORK_GLYPH: &str = "â‘‚";            // U+2442
pub const DIAMOND_OPEN: &str = "â—‡";          // U+25C7 - running/pending review
pub const DIAMOND_FILLED: &str = "â—†";        // U+25C6 - completed review
pub const REFERENCE_MARK: &str = "â€»";        // U+203B - away summary marker
pub const FLAG_ICON: &str = "âš‘";             // U+2691
pub const BLOCKQUOTE_BAR: &str = "—Ž";        // U+258E - blockquote left bar
pub const HEAVY_HORIZONTAL: &str = "â”";      // U+2501
pub const THEREFORE: &str = "âˆ´";             // U+2234 - alternative thinking symbol
pub const NEW_MESSAGES_DOWN: &str = "â†“";
pub const BRIDGE_READY: &str = "Â· âœ” Â·";
pub const BRIDGE_FAILED: &str = "×";
