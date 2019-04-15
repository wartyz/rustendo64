use super::interconnect;

const NUM_GPR: usize = 32; // 32 registros

#[derive(Debug)]
pub struct Cpu {
    // Arreglo de registros de proposito general
    reg_gpr: [u64; NUM_GPR],
    // Arreglo de registros de coma flotante
    reg_fpr: [f64; NUM_GPR],
    // Contador de programa
    reg_pc: u64,
    // Para resultados de palabra doble??
    reg_hi: u64,
    // Para resultados de palabra doble??
    reg_lo: u64,

    reg_llbit: bool,
    //TODO: Enum type
    //Registro de Implementacion/Revision FCR0
    reg_fcr0: u32,
    // Registro de Control/Status FCR31
    reg_fcr31: u32,

    cp0: Cp0, // Coprocesador

    interconnect: interconnect::Interconnect, // En interconnect está la ROM y la RAM
}

// CPU
impl Cpu {
    pub fn new(interconnect: interconnect::Interconnect) -> Cpu {
        Cpu {
            reg_gpr: [0; NUM_GPR],      // 32 Registros de proposito general
            reg_fpr: [0.0; NUM_GPR],    // 32 Registros de coma flotante
            reg_pc: 0,                  // Contador de programa
            reg_hi: 0,                  // Registros de multiplicar y dividir
            reg_lo: 0,
            reg_llbit: false,           // Registro Load/Link de un bit
            reg_fcr0: 0,                // Registro Implementacion/revision coma flotante
            reg_fcr31: 0,               // Registro control/status coma flotante
            cp0: Cp0::default(),        // Coprocesador, CP0 cumple el trait "Default"

            interconnect: interconnect, // En interconnect está la ROM y la RAM
        }
    }

    pub fn power_on_reset(&mut self) {
        self.cp0.power_on_reset();

        self.reg_pc = 0xffff_ffff_bfc0_0000; // TODO: move to const
    }

    // TODO: Different interface
    pub fn run(&mut self) {
        loop {
            self.run_instruction();
        }
    }

    pub fn run_instruction(&mut self) {
        let instruction = self.read_word(self.reg_pc);

        //TODO: Check endian
        let opcode = (instruction >> 26) & 0b111111; // Opcode 6 bits altos
        match opcode {
            0b001111 => {
                // LUI, Load Upper Immediate en registro rt (numero de registro)
                println!("We got LUI!");
                let imm = instruction & 0xffff; // Valor inmediato
                let rt = (instruction >> 16) & 0b11111;
                println!("rt: {:#?}", rt);
                // TODO: Check 32 vs 64 bit mode for sign extend
                // (currently 32 bit mode is assumed)
                self.write_reg_gpr(rt as usize, (imm << 16) as u64);
            }
            _ => {
                panic!("Unrecognized instruction: {:#x}", instruction);
            }
        }
        self.reg_pc += 4;
    }

    fn read_word(&self, virt_addr: u64) -> u32 {
        let phys_addr = self.virt_addr_to_phys_addr(virt_addr);
        // TODO: Check endianness
        self.interconnect.read_word(phys_addr as u32)
    }

    fn virt_addr_to_phys_addr(&self, virt_addr: u64) -> u64 {
        // See Table 5-3 in the VR4300 User's Manual
        let addr_bit_values = (virt_addr >> 29) & 0b111;

        if addr_bit_values == 0b101 {
            // kseg1
            virt_addr - 0xffff_ffff_a000_0000
        } else {
            // TODO
            panic!("Unrecognized virtual address: {:#x}", virt_addr);
        }
    }

    fn write_reg_gpr(&mut self, index: usize, value: u64) {
        if index != 0 {
            self.reg_gpr[index] = value;
        }
    }
}

// TODO: Better name?
#[derive(Debug)]
enum RegConfigEp {
    D,
    // TODO: Better name?
    DxxDxx,
    // TODO: Better name?
    RFU,
}

impl Default for RegConfigEp {
    fn default() -> RegConfigEp {
        RegConfigEp::D
    }
}

// TODO: Better name?
#[derive(Debug)]
enum RegConfigBe {
    LittleEndian,
    BigEndian,
}

// BigEndian por defecto
impl Default for RegConfigBe {
    fn default() -> RegConfigBe {
        RegConfigBe::BigEndian
    }
}

#[derive(Debug, Default)]
struct RegConfig {
    reg_config_ep: RegConfigEp,
    reg_config_be: RegConfigBe,
}

impl RegConfig {
    fn power_on_reset(&mut self) {
        self.reg_config_ep = RegConfigEp::D;
        self.reg_config_be = RegConfigBe::BigEndian;
    }
}

// Coprocesador
#[derive(Debug, Default)]
struct Cp0 {
    reg_config: RegConfig,
}
//    // Puntero programable a arreglo TLB
//    reg_index: u64,
//    // Puntero pseudo rmdom a arreglo TLB (solo lectura)
//    reg_random: u64,
//    //Mitad baja de entrada TLB para direccion virtual par
//    reg_entry_lo0: u64,
//    //Mitad baja de entrada TLB para direccion virtual impar
//    reg_entry_lo1: u64,
//    // Puntero entrada a la tabla de pagina virtual del kernel (PTE) en modo 32 bits
//    reg_context: u64,
//    // Especificacion de tamaño de pagina
//    reg_page_mask: u64,
//    // Numero de entradas wired TLB
//    reg_wired: u64,
//    //Presenta direccion virtual ocurrida en el ultimo error
//    reg_bad_v_addr: u64,
//    //Contador timer
//    reg_count: u64,
//    // Mitad alta de entrada TLB (incluyendo ASID)
//    reg_entry_hi: u64,
//    // Valor de comparacion de timer
//    reg_compare: u64,
//    // Establece operacion de status
//    reg_status: u64,
//    // Presenta la causa de la ultima excepcion
//    reg_cause: u64,
//    // Contador de programa de excepcion
//    reg_epc: u64,
//    //Identificador de revision de procesador
//    reg_pr_id: u64,
//    //Establece modo de sistema de memoria
//    reg_config: u64,
//    // Carga direccion de de intruccion linkada del display
//    reg_ll_addr: u64,
//    // bits bajos de trap de referencia de memoria
//    reg_watch_lo: u64,
//    // bits altos de trap de referencia de memoria
//    reg_watch_hi: u64,
//    // Puntero a tabla virtual PTE del Kernel en modo 64 bits
//    reg_x_context: u64,
//    // Registro bajo de error de cache
//    reg_tag_lo: u64,
//    // Registro alto de error de cache
//    reg_tag_hi: u64,
//    // Contador de programna de error de excepcion
//    reg_error_epc: u64,

impl Cp0 {
    fn power_on_reset(&mut self) {
        self.reg_config.power_on_reset();
    }
}
