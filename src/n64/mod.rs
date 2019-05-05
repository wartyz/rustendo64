//mod n64;
//pub mod cpu;
//mod pif;
//mod rsp;
//mod rdp;
//mod audio_interface;
//mod video_interface;
//mod peripheral_interface;
//mod serial_interface;
//mod interconnect;
//pub mod mem_map;
//
//pub use self::n64::N64;

mod audio_interface;
pub mod cpu;
mod interconnect;
pub mod mem_map;
mod n64;
mod peripheral_interface;
mod pif;
mod rdp;
mod rsp;
mod serial_interface;
mod video_interface;

pub use self::audio_interface::AudioInterface;
pub use self::cpu::Cpu;
pub use self::interconnect::Interconnect;
pub use self::n64::N64;
pub use self::peripheral_interface::PeripheralInterface;
pub use self::pif::Pif;
pub use self::rdp::Rdp;
pub use self::rsp::Rsp;
pub use self::serial_interface::SerialInterface;
pub use self::video_interface::VideoInterface;
