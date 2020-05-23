mod registers;

use registers::*;
pub use registers::InterruptRequest;

pub struct InterruptController {
    pub enable: InterruptEnable,
    pub master_enable: InterruptMasterEnable,
    pub request: InterruptRequest,
}

impl InterruptController {
    pub fn new() -> InterruptController {
        InterruptController {
            enable: InterruptEnable::empty(),
            master_enable: InterruptMasterEnable::empty(),
            request: InterruptRequest::empty(),
        }
    }
}
