// TUI owns stdout, so printing debug values corrupts the render.
// This appends them to debug.log instead. Tail it in a second terminal.
#[macro_export]
macro_rules! dbg_log {
    ($($arg:tt)*) => {{
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
        {
            let _ = writeln!(file, $($arg)*);
        }
    }};
}
