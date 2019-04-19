use super::super::interconnect;
use super::cp0::cp0;

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

    cp0: cp0::Cp0,

    interconnect: interconnect::Interconnect,
}

// CPU
impl Cpu {
    pub fn new(interconnect: interconnect::Interconnect) -> Cpu {
        Cpu {
            reg_gpr: [0; NUM_GPR],   // Registros de proposito general
            reg_fpr: [0.0; NUM_GPR], // Registros de coma flotante
            reg_pc: 0,               // Contador de programa
            reg_hi: 0,               // Registros de multiplicar y dividie
            reg_lo: 0,
            reg_llbit: false,        // Registro Load/Link
            reg_fcr0: 0,             // Registro Implementacion/revision coma flotante
            reg_fcr31: 0,            // Registro control/status coma flotante
            cp0: cp0::Cp0::default(),

            interconnect: interconnect,
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
        let rs = (instruction >> 21) & 0b11111; // Registro source
        let rt = (instruction >> 16) & 0b11111; // Registro destino (target)
        let imm = instruction & 0xffff; // Valor inmediato

        // rt => reg source, rd => reg destino, rt => reg destino/fuente
        // Tipo R-> opcode  rs    rt    rd   shamt funct
        //          XXXXXX XXXXX XXXXX XXXXX XXXXX XXXXXX
        // Tipo I-> opcode  rs    rt    immediate(imm)
        //          XXXXXX XXXXX XXXXX XXXXXXXXXXXXXXXX
        // Tipo J-> opcode        address
        //          XXXXXX XXXXXXXXXXXXXXXXXXXXXXXXXX

        match opcode {
            0b001101 => {
                // Tipo I, ori -> imm OR Registro que dice rs ->registro que dice rt
                let res = self.read_reg_gpr(rs as usize) | (imm as u64);
                self.write_reg_gpr(rt as usize, res);
            }
            0b001111 => {
                // Tipo I, lui, Load Upper Immediate en registro que dice rt (numero de registro)
                let value = ((imm << 16) as u64);
                self.write_reg_gpr(rt as usize, value);
            }
            0b010000 => {
                // mtc0: Ejemplo mtc0 t1, $12 _> carga lo que hay en t1 al registro 12 de C0
                let rd = (instruction >> 11) & 0b11111;
                let data = self.read_reg_gpr(rt as usize);
                self.cp0.write_reg(rd, data);
            }
            0b100011 => {
                // Tipo I, lw // rs = base añadida al offset => dirección virtual hacia reg rt
                let base = rs;
                let offset = imm;

                let sign_extended_offset = (offset as i16) as u64;
                let virt_addr = sign_extended_offset + self.read_reg_gpr(base as usize);
                let mem = (self.read_word(virt_addr) as i32) as u64;
                self.write_reg_gpr(rt as usize, mem);
            }

            _ => panic!("Unrecognized instruction: {:#x}", instruction)
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

    // Escribe en el registro general con número index, el valor value
    fn write_reg_gpr(&mut self, index: usize, value: u64) {
        if index != 0 {
            self.reg_gpr[index] = value;
        }
    }

    // Devuelve el valor en el registro general del numero indicado
    fn read_reg_gpr(&self, index: usize) -> u64 {
        match index {
            0 => 0,
            _ => self.reg_gpr[index]
        }
    }
}