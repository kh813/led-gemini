use std::io::{self, Write};
use base64::{Engine as _, engine::general_purpose};

pub fn set_clipboard(text: &str) -> io::Result<()> {
    // OSC 52
    let b64 = general_purpose::STANDARD.encode(text);
    let osc52 = format!("\x1b]52;c;{}\x07", b64);
    io::stdout().write_all(osc52.as_bytes())?;
    io::stdout().flush()?;

    // Platform clipboard (placeholder for now)
    // In a real implementation, we might use a crate like `arboard`
    
    Ok(())
}
