/* ------------------- RegConfig ------------------------*/
#[derive(Debug, Default)]
pub struct RegConfig {
    // EP
    data_transfer_pattern: DataTransferPattern,
    // BE (Big Endian por defecto)
    endianness: Endianness,
    cu: bool,
    kseg0_cache_enable_bits: [bool; 3],
    //kseg0_cache_enabled: bool,
}

impl RegConfig {
    fn kseg0_cache_enabled(&self) -> bool {
        !(!self.kseg0_cache_enable_bits[0] &&
            self.kseg0_cache_enable_bits[1] &&
            !self.kseg0_cache_enable_bits[2])
    }
}

impl From<u32> for RegConfig {
    fn from(value: u32) -> Self {
        RegConfig {
            data_transfer_pattern: value.into(),

            endianness: value.into(),

            cu: (value & (1 << 3)) != 0,
            kseg0_cache_enable_bits: [
                (value & (1 << 0)) != 0,
                (value & (1 << 1)) != 0,
                (value & (1 << 2)) != 0,
            ],
        }
    }
}

/* ------------------- DataTransferPattern ------------------------*/
#[derive(Debug)]
enum DataTransferPattern {
    // D
    Normal,
    DxxDxx,
}

impl Default for DataTransferPattern {
    fn default() -> Self {
        DataTransferPattern::Normal
    }
}

impl From<u32> for DataTransferPattern {
    fn from(value: u32) -> Self {
        match (value >> 24) & 0b1111 {
            0 => DataTransferPattern::Normal,
            6 => DataTransferPattern::DxxDxx,
            _ => panic!("Invalid data transfer pattern (EP): {:#x}", value)
        }
    }
}

/* ------------------- Endianness ------------------------*/
#[derive(Debug)]
enum Endianness {
    Little,
    Big,
}

impl Default for Endianness {
    fn default() -> Self {
        Endianness::Big
    }
}

impl From<u32> for Endianness {
    fn from(value: u32) -> Self {
        match (value >> 15) & 0b1 {
            0 => Endianness::Little,
            1 => Endianness::Big,
            _ => unreachable!()
        }
    }
}