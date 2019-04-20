use super::byteorder::{BigEndian, ByteOrder};
use super::mem_map::*;
use super::rsp::Rsp;

use std::fmt;

const RAM_SIZE: usize = 4 * 1024 * 1024; // 4 Megas

pub struct Interconnect {
    rsp: Rsp,
    pif_rom: Box<[u8]>,
    ram: Box<[u16]>,
}

impl Interconnect {
    pub fn new(pif_rom: Box<[u8]>) -> Interconnect {
        Interconnect {
            rsp: Rsp::default(),
            pif_rom: pif_rom,
            // Convierte arreglo a slice de punteros Box
            ram: vec![0; RAM_SIZE].into_boxed_slice(),
        }
    }

    /// Lee de memoria fisica (no virtual)
    pub fn read_word(&self, addr: u32) -> u32 {
        // Lee 32 bits de memoria
        // TODO: Replace constants with useful names
        if addr >= PIF_ROM_START && addr < PIF_ROM_END {
            let rel_addr = addr - PIF_ROM_START;
            BigEndian::read_u32(&self.pif_rom[rel_addr as usize..])
        } else {
            match addr {
                SP_STATUS_REG => self.rsp.read_status_reg(),
                _ => panic!("Unrecognized physical address: {:#x}", addr),
            }
        }
    }
}

impl fmt::Debug for Interconnect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TODO: Impl Debug for Interconnect")
    }
}