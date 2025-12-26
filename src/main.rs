mod rwm;

use std::error::Error;
use rwm::MiniWM;

fn main() -> Result<(), Box<dyn Error>> {
    let mut wm = MiniWM::new()?;

    wm.init()?;

    Ok(wm.run()?)
}
