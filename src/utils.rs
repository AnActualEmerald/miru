use std::io::stdout;

use crossterm::terminal::{self, ClearType};
use crossterm::ExecutableCommand;
use spinners::Spinner;

pub fn clear_spinner(sp: Spinner) {
    sp.stop();
    let mut stdout = stdout();
    stdout
        .execute(terminal::Clear(ClearType::CurrentLine))
        .expect("Unable to clear line");
    print!("\r");
}
