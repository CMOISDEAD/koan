mod koan;

use std::error::Error;
use koan::KoanWM;

fn main() -> Result<(), Box<dyn Error>> {
    let mut wm = KoanWM::new()?;

    wm.init()?;

    Ok(wm.run()?)
}
