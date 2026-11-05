//! # Termix
//!
//! `termix` is a terminal formatting and coloring crate that provides convenient macros
//! for styling terminal output using ANSI escape codes.
//!
//! Unlike simple string concatenation, `termix` tracks active color states on a
//! thread-local stack, allowing nested styles to automatically restore their parent
//! color context rather than resetting to default terminal styling.
//!
//! ## Features
//!
//! - **Standard Colors**: 8 foreground colors (`red!`, `green!`, `blue!`, `yellow!`, `magenta!`, `cyan!`, `white!`, `black!`)
//! - **Background Colors**: 8 background colors (`bg_red!`, `bg_green!`, `bg_blue!`, etc.)
//! - **Text Styles**: `bold!`, `italic!`, and `underline!`
//! - **24-bit Truecolor**: `rgb!` and `bg_rgb!` for full RGB color support
//! - **Color Spaces**: `hsl!`, `hsv!`, `bg_hsl!`, and `bg_hsv!` for HSL/HSV color models
//! - **Context-Aware Nesting**: Inner styles restore outer styles upon closing
//!
//! ## Quick Start
//!
//! ```rust
//! use termix::{bg_yellow, bold, green, red};
//!
//! // Basic color output
//! println!("{}", red!("Error: something went wrong"));
//!
//! // Nested styles: green text resets back to red, not default terminal color
//! println!("{}", red!("Warning: {} occurred", green!("retry")));
//!
//! // Combining styles and backgrounds
//! println!("{}", bg_yellow!("{}", bold!("Important Notice")));
//! ```

use std::cell::RefCell;

#[cfg(test)]
mod test;

// Thread-local stack maintaining active ANSI color codes for nested macro calls.
// Defaults to standard white (\x1b[37m).
thread_local! {
    static COLOR_CONTEXT: RefCell<Vec<String>> = RefCell::new(vec![String::from("\x1b[37m")]);
}

/// Thread-local color stack manager.
///
/// Because ANSI escape codes do not have closing tags (only a global reset `\x1b[0m`),
/// resetting inside a nested format string would unintentionally clear any outer color.
/// `ColorContext` maintains a stack of active colors so nested macros can re-apply
/// the enclosing color when an inner formatted block ends.
#[allow(dead_code)]
pub struct ColorContext;

#[allow(dead_code)]
impl ColorContext {
    /// Pushes an ANSI color code onto the current thread's context stack.
    pub fn push(color: &str) {
        COLOR_CONTEXT.with(|ctx| {
            ctx.borrow_mut().push(color.to_string());
        });
    }

    /// Pops the most recent ANSI color code from the current thread's context stack.
    pub fn pop() {
        COLOR_CONTEXT.with(|ctx| {
            ctx.borrow_mut().pop();
        });
    }

    /// Returns the active ANSI color code, defaulting to white (`\x1b[37m`) if empty.
    pub fn current_color() -> String {
        COLOR_CONTEXT.with(|ctx| {
            ctx.borrow()
                .last()
                .cloned()
                .unwrap_or_else(|| String::from("\x1b[37m"))
        })
    }
}

/// Converts HSL (Hue, Saturation, Lightness) color values to 8-bit RGB `(r, g, b)`.
///
/// # Arguments
///
/// * `h` - Hue angle in degrees, in range `[0.0, 360.0]`.
/// * `s` - Saturation ratio in range `[0.0, 1.0]`.
/// * `l` - Lightness ratio in range `[0.0, 1.0]`.
///
/// # Returns
///
/// An 8-bit RGB tuple `(r, g, b)` with components in `0..=255`.
pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let c: f64 = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x: f64 = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m: f64 = l - c / 2.0;

    let (r, g, b): (f64, f64, f64) = match h {
        h if h >= 0.0 && h < 60.0 => (c, x, 0.0),
        h if h >= 60.0 && h < 120.0 => (x, c, 0.0),
        h if h >= 120.0 && h < 180.0 => (0.0, c, x),
        h if h >= 180.0 && h < 240.0 => (0.0, x, c),
        h if h >= 240.0 && h < 300.0 => (x, 0.0, c),
        h if h >= 300.0 && h <= 360.0 => (c, 0.0, x),
        _ => (0.0, 0.0, 0.0),
    };

    let r: u8 = ((r + m) * 255.0).round() as u8;
    let g: u8 = ((g + m) * 255.0).round() as u8;
    let b: u8 = ((b + m) * 255.0).round() as u8;

    (r, g, b)
}

/// Converts HSV (Hue, Saturation, Value) color values to 8-bit RGB `(r, g, b)`.
///
/// # Arguments
///
/// * `h` - Hue angle in degrees, in range `[0.0, 360.0]`.
/// * `s` - Saturation ratio in range `[0.0, 1.0]`.
/// * `v` - Value (brightness) ratio in range `[0.0, 1.0]`.
///
/// # Returns
///
/// An 8-bit RGB tuple `(r, g, b)` with components in `0..=255`.
pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let c: f64 = v * s;
    let x: f64 = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m: f64 = v - c;

    let (r, g, b): (f64, f64, f64) = match h {
        h if h >= 0.0 && h < 60.0 => (c, x, 0.0),
        h if h >= 60.0 && h < 120.0 => (x, c, 0.0),
        h if h >= 120.0 && h < 180.0 => (0.0, c, x),
        h if h >= 180.0 && h < 240.0 => (0.0, x, c),
        h if h >= 240.0 && h < 300.0 => (x, 0.0, c),
        h if h >= 300.0 && h <= 360.0 => (c, 0.0, x),
        _ => (0.0, 0.0, 0.0),
    };

    let r: u8 = ((r + m) * 255.0).round() as u8;
    let g: u8 = ((g + m) * 255.0).round() as u8;
    let b: u8 = ((b + m) * 255.0).round() as u8;

    (r, g, b)
}

/// Returns the ANSI escape sequence to reset all text attributes and colors (`\x1b[0m`).
pub fn reset_all() -> &'static str {
    "\x1b[0m"
}

/// Formats text with an ANSI escape code while maintaining nested color context.
///
/// Pushes `color_code` onto [`ColorContext`], formats the string, appends an ANSI reset,
/// pops the context, and restores the enclosing parent color code.
///
/// # Example
///
/// ```
/// use termix::apply_color;
///
/// let text = apply_color!("\x1b[31m", "Error: {}", 404);
/// ```
#[macro_export]
macro_rules! apply_color {
    ($color_code:expr, $($arg:tt)*) => {{
        $crate::ColorContext::push($color_code);
        let result = format!("{}{}{}", $color_code, format!($($arg)*), $crate::reset_all());
        $crate::ColorContext::pop();
        format!("{}{}", result, $crate::ColorContext::current_color())
    }};
}

// -----------------------------------------------------------------------------
// Standard Foreground Colors
// -----------------------------------------------------------------------------

/// Formats text with red foreground color (`\x1b[31m`).
///
/// # Example
///
/// ```
/// use termix::red;
///
/// println!("{}", red!("Failed to connect: {}", "timeout"));
/// ```
#[macro_export]
macro_rules! red {
    ($($arg:tt)*) => {{
        $crate::apply_color!("\x1b[31m", $($arg)*)
    }};
}

/// Formats text with green foreground color (`\x1b[32m`).
///
/// # Example
///
/// ```
/// use termix::green;
///
/// println!("{}", green!("Build succeeded: {} tasks completed", 12));
/// ```
#[macro_export]
macro_rules! green {
    ($($arg:tt)*) => {{
        $crate::apply_color!("\x1b[32m", $($arg)*)
    }};
}

/// Formats text with blue foreground color (`\x1b[34m`).
///
/// # Example
///
/// ```
/// use termix::blue;
///
/// println!("{}", blue!("Info: reading configuration"));
/// ```
#[macro_export]
macro_rules! blue {
    ($($arg:tt)*) => {{
        $crate::apply_color!("\x1b[34m", $($arg)*)
    }};
}

/// Formats text with white foreground color (`\x1b[37m`).
///
/// # Example
///
/// ```
/// use termix::white;
///
/// println!("{}", white!("Normal message"));
/// ```
#[macro_export]
macro_rules! white {
    ($($arg:tt)*) => {{
        $crate::apply_color!("\x1b[37m", $($arg)*)
    }};
}

/// Formats text with black foreground color (`\x1b[30m`).
///
/// # Example
///
/// ```
/// use termix::black;
///
/// println!("{}", black!("Subtle text"));
/// ```
#[macro_export]
macro_rules! black {
    ($($arg:tt)*) => {{
        $crate::apply_color!("\x1b[30m", $($arg)*)
    }};
}

/// Formats text with yellow foreground color (`\x1b[33m`).
///
/// # Example
///
/// ```
/// use termix::yellow;
///
/// println!("{}", yellow!("Warning: deprecated configuration key"));
/// ```
#[macro_export]
macro_rules! yellow {
    ($($arg:tt)*) => {{
        $crate::apply_color!("\x1b[33m", $($arg)*)
    }};
}

/// Formats text with magenta foreground color (`\x1b[35m`).
///
/// # Example
///
/// ```
/// use termix::magenta;
///
/// println!("{}", magenta!("Highlighted token: {}", "Syntax"));
/// ```
#[macro_export]
macro_rules! magenta {
    ($($arg:tt)*) => {{
        $crate::apply_color!("\x1b[35m", $($arg)*)
    }};
}

/// Formats text with cyan foreground color (`\x1b[36m`).
///
/// # Example
///
/// ```
/// use termix::cyan;
///
/// println!("{}", cyan!("Debug details: pid {}", 4242));
/// ```
#[macro_export]
macro_rules! cyan {
    ($($arg:tt)*) => {{
        $crate::apply_color!("\x1b[36m", $($arg)*)
    }};
}

// -----------------------------------------------------------------------------
// Custom Foreground Colors (RGB, HSL, HSV)
// -----------------------------------------------------------------------------

/// Formats text with a 24-bit Truecolor (RGB) foreground (`\x1b[38;2;R;G;Bm`).
///
/// # Arguments
///
/// * `r` - Red component (`0..=255`)
/// * `g` - Green component (`0..=255`)
/// * `b` - Blue component (`0..=255`)
///
/// # Example
///
/// ```
/// use termix::rgb;
///
/// println!("{}", rgb!(255, 128, 0, "Custom orange text"));
/// ```
#[macro_export]
macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr, $($arg:tt)*) => {{
        let color_code = format!("\x1b[38;2;{};{};{}m", $r, $g, $b);
        $crate::apply_color!(&color_code, $($arg)*)
    }};
}

/// Formats text with an HSL foreground color by converting it to 24-bit RGB.
///
/// # Arguments
///
/// * `h` - Hue in degrees (`0.0..=360.0`)
/// * `s` - Saturation ratio (`0.0..=1.0`)
/// * `l` - Lightness ratio (`0.0..=1.0`)
///
/// # Example
///
/// ```
/// use termix::hsl;
///
/// println!("{}", hsl!(120.0, 1.0, 0.5, "Vibrant green text"));
/// ```
#[macro_export]
macro_rules! hsl {
    ($h:expr, $s:expr, $l:expr, $($arg:tt)*) => {{
        let (r, g, b) = $crate::hsl_to_rgb($h, $s, $l);
        $crate::rgb!(r, g, b, $($arg)*)
    }};
}

/// Formats text with an HSV foreground color by converting it to 24-bit RGB.
///
/// # Arguments
///
/// * `h` - Hue in degrees (`0.0..=360.0`)
/// * `s` - Saturation ratio (`0.0..=1.0`)
/// * `v` - Value / brightness ratio (`0.0..=1.0`)
///
/// # Example
///
/// ```
/// use termix::hsv;
///
/// println!("{}", hsv!(240.0, 1.0, 1.0, "Pure blue text"));
/// ```
#[macro_export]
macro_rules! hsv {
    ($h:expr, $s:expr, $v:expr, $($arg:tt)*) => {{
        let (r, g, b) = $crate::hsv_to_rgb($h, $s, $v);
        $crate::rgb!(r, g, b, $($arg)*)
    }};
}

// -----------------------------------------------------------------------------
// Standard Background Colors
// -----------------------------------------------------------------------------

/// Formats text with red background color (`\x1b[41m`).
///
/// # Example
///
/// ```
/// use termix::bg_red;
///
/// println!("{}", bg_red!("CRITICAL ERROR"));
/// ```
#[macro_export]
macro_rules! bg_red {
    ($($arg:tt)*) => {{
        $crate::apply_color!("\x1b[41m", $($arg)*)
    }};
}

/// Formats text with green background color (`\x1b[42m`).
///
/// # Example
///
/// ```
/// use termix::bg_green;
///
/// println!("{}", bg_green!("PASS"));
/// ```
#[macro_export]
macro_rules! bg_green {
    ($($arg:tt)*) => {{
        $crate::apply_color!("\x1b[42m", $($arg)*)
    }};
}

/// Formats text with blue background color (`\x1b[44m`).
///
/// # Example
///
/// ```
/// use termix::bg_blue;
///
/// println!("{}", bg_blue!("INFO BANNER"));
/// ```
#[macro_export]
macro_rules! bg_blue {
    ($($arg:tt)*) => {{
        $crate::apply_color!("\x1b[44m", $($arg)*)
    }};
}

/// Formats text with white background color (`\x1b[47m`).
///
/// # Example
///
/// ```
/// use termix::bg_white;
///
/// println!("{}", bg_white!("Light highlight"));
/// ```
#[macro_export]
macro_rules! bg_white {
    ($($arg:tt)*) => {{
        $crate::apply_color!("\x1b[47m", $($arg)*)
    }};
}

/// Formats text with black background color (`\x1b[40m`).
///
/// # Example
///
/// ```
/// use termix::bg_black;
///
/// println!("{}", bg_black!("Dark badge"));
/// ```
#[macro_export]
macro_rules! bg_black {
    ($($arg:tt)*) => {{
        $crate::apply_color!("\x1b[40m", $($arg)*)
    }};
}

/// Formats text with yellow background color (`\x1b[43m`).
///
/// # Example
///
/// ```
/// use termix::bg_yellow;
///
/// println!("{}", bg_yellow!("ATTENTION"));
/// ```
#[macro_export]
macro_rules! bg_yellow {
    ($($arg:tt)*) => {{
        $crate::apply_color!("\x1b[43m", $($arg)*)
    }};
}

/// Formats text with magenta background color (`\x1b[45m`).
///
/// # Example
///
/// ```
/// use termix::bg_magenta;
///
/// println!("{}", bg_magenta!("Notice"));
/// ```
#[macro_export]
macro_rules! bg_magenta {
    ($($arg:tt)*) => {{
        $crate::apply_color!("\x1b[45m", $($arg)*)
    }};
}

/// Formats text with cyan background color (`\x1b[46m`).
///
/// # Example
///
/// ```
/// use termix::bg_cyan;
///
/// println!("{}", bg_cyan!("Status header"));
/// ```
#[macro_export]
macro_rules! bg_cyan {
    ($($arg:tt)*) => {{
        $crate::apply_color!("\x1b[46m", $($arg)*)
    }};
}

// -----------------------------------------------------------------------------
// Text Style Modifiers
// -----------------------------------------------------------------------------

/// Formats text with bold styling (`\x1b[1m`).
///
/// # Example
///
/// ```
/// use termix::bold;
///
/// println!("{}", bold!("Bold message"));
/// ```
#[macro_export]
macro_rules! bold {
    ($($arg:tt)*) => {{
        $crate::apply_color!("\x1b[1m", $($arg)*)
    }};
}

/// Formats text with italic styling (`\x1b[3m`).
///
/// # Example
///
/// ```
/// use termix::italic;
///
/// println!("{}", italic!("Italic note"));
/// ```
#[macro_export]
macro_rules! italic {
    ($($arg:tt)*) => {{
        $crate::apply_color!("\x1b[3m", $($arg)*)
    }};
}

/// Formats text with underline styling (`\x1b[4m`).
///
/// # Example
///
/// ```
/// use termix::underline;
///
/// println!("{}", underline!("Underlined label"));
/// ```
#[macro_export]
macro_rules! underline {
    ($($arg:tt)*) => {{
        $crate::apply_color!("\x1b[4m", $($arg)*)
    }};
}

// -----------------------------------------------------------------------------
// Custom Background Colors (RGB, HSL, HSV)
// -----------------------------------------------------------------------------

/// Formats text with a 24-bit Truecolor (RGB) background (`\x1b[48;2;R;G;Bm`).
///
/// # Arguments
///
/// * `r` - Red component (`0..=255`)
/// * `g` - Green component (`0..=255`)
/// * `b` - Blue component (`0..=255`)
///
/// # Example
///
/// ```
/// use termix::bg_rgb;
///
/// println!("{}", bg_rgb!(40, 44, 52, "Custom dark background"));
/// ```
#[macro_export]
macro_rules! bg_rgb {
    ($r:expr, $g:expr, $b:expr, $($arg:tt)*) => {{
        let color_code = format!("\x1b[48;2;{};{};{}m", $r, $g, $b);
        $crate::apply_color!(&color_code, $($arg)*)
    }};
}

/// Formats text with an HSL background color by converting it to 24-bit RGB.
///
/// # Arguments
///
/// * `h` - Hue in degrees (`0.0..=360.0`)
/// * `s` - Saturation ratio (`0.0..=1.0`)
/// * `l` - Lightness ratio (`0.0..=1.0`)
///
/// # Example
///
/// ```
/// use termix::bg_hsl;
///
/// println!("{}", bg_hsl!(180.0, 0.5, 0.5, "Teal background"));
/// ```
#[macro_export]
macro_rules! bg_hsl {
    ($h:expr, $s:expr, $l:expr, $($arg:tt)*) => {{
        let (r, g, b) = $crate::hsl_to_rgb($h, $s, $l);
        $crate::bg_rgb!(r, g, b, $($arg)*)
    }};
}

/// Formats text with an HSV background color by converting it to 24-bit RGB.
///
/// # Arguments
///
/// * `h` - Hue in degrees (`0.0..=360.0`)
/// * `s` - Saturation ratio (`0.0..=1.0`)
/// * `v` - Value / brightness ratio (`0.0..=1.0`)
///
/// # Example
///
/// ```
/// use termix::bg_hsv;
///
/// println!("{}", bg_hsv!(270.0, 0.7, 0.9, "Purple background"));
/// ```
#[macro_export]
macro_rules! bg_hsv {
    ($h:expr, $s:expr, $v:expr, $($arg:tt)*) => {{
        let (r, g, b) = $crate::hsv_to_rgb($h, $s, $v);
        $crate::bg_rgb!(r, g, b, $($arg)*)
    }};
}
