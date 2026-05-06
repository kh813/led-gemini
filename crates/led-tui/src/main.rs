mod app;
mod renderer;
mod layout;
mod clipboard;
mod widgets;

use app::App;
use anyhow::Result;

fn main() -> Result<()> {
    let mut app = App::new()?;
    app.run()
}
