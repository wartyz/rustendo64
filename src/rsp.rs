#[derive(Default)]
pub struct Rsp;

impl Rsp {
    // TODO: Read general regs
    pub fn read_status_reg(&self) -> u32 {
        0 // TODO: Properly implement this
    }
}
