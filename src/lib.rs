mod tables;
use tables::{REGISTER_TABLE, WIDE_REGISTER_TABLE};

// OPs
const MOV: u8 = 0b10_0010;
const IMMEDIATE_TO_REG_OR_MEM: u8 = 0b0110_0011;
const IMMEDITAE_TO_REG: u8 = 0b1011;
const MEM_TO_ACC: u8 = 0b101;
const ACC_TO_MEM: u8 = 0b0101_0001;
const ADD: u8 = 0b0000_0000;

// d
const REG_IS_DEST: u8 = 0b0000_0010;

// w
const WIDE: u8 = 0b0000_0001;

// mod
const MEM_MODE: u8 = 0b0000_0000;
const MEM_MODE_BYTE_DIS: u8 = 0b0100_0000;
const MEM_MODE_WORD_DIS: u8 = 0b1000_0000;
const REG_MODE: u8 = 0b1100_0000;

static mut AX: u16 = 0x00;
static mut BX: u16 = 0x00;
static mut CX: u16 = 0x00;
static mut DX: u16 = 0x00;
static mut SP: u16 = 0x00;
static mut BP: u16 = 0x00;
static mut SI: u16 = 0x00;
static mut DI: u16 = 0x00;

fn rm_to_rg(rm: u8) -> String {
    match rm {
        0b0000_0000 => "[bx + si".into(),
        0b0000_0001 => "[bx + di".into(),
        0b0000_0010 => "[bp + si".into(),
        0b0000_0011 => "[bp + di".into(),
        0b0000_0100 => "[si".into(),
        0b0000_0101 => "[di".into(),
        // direction address, with potential offset!
        0b0000_0110 => "[bp".into(),
        0b0000_0111 => "[bx".into(),
        _ => panic!("invalid rm"),
    }
}

fn get_displacement_byte(dis: i8) -> String {
    let mut buffer = String::new();
    if dis.is_negative() {
        buffer.push_str(" - ");
    } else {
        buffer.push_str(" + ");
    }
    buffer.push_str(&format!("{}", dis.abs()));

    buffer
}

fn get_displacement_word(dis: [u8; 2]) -> String {
    let mut buffer = String::new();
    let dis = i16::from_le_bytes(dis);
    if dis.is_negative() {
        buffer.push_str(" - ");
    } else {
        buffer.push_str(" + ");
    }
    buffer.push_str(&format!("{}", dis.abs()));

    buffer
}

enum Mode {
    Mem,
    MemByteDis,
    MemWordDis,
    Reg,
}

impl From<u8> for Mode {
    fn from(value: u8) -> Self {
        if value & REG_MODE == REG_MODE {
            return Mode::Reg;
        }
        if value & MEM_MODE_WORD_DIS == MEM_MODE_WORD_DIS {
            return Mode::MemWordDis;
        }
        if value & MEM_MODE_BYTE_DIS == MEM_MODE_BYTE_DIS {
            return Mode::MemByteDis;
        }
        if value & MEM_MODE == MEM_MODE {
            return Mode::Mem;
        }
        panic!("invald mode")
    }
}

fn mem_mode(buffer: &[u8], bp: &mut usize, buffer_out: &mut String) {
    let reg_is_dest = buffer[*bp] & REG_IS_DEST == REG_IS_DEST;
    let is_wide = buffer[*bp] & WIDE == WIDE;
    let reg = (buffer[*bp + 1] & 0b0011_1000) >> 3;
    let reg_mem = buffer[*bp + 1] & 0b0000_0111;
    if is_wide {
        if reg_is_dest {
            buffer_out.push_str(WIDE_REGISTER_TABLE.get(&reg).unwrap());
            buffer_out.push_str(", ");
            if reg_mem == 0b0000_0110 {
                buffer_out.push_str(&format!(
                    "[{}",
                    u16::from_le_bytes([buffer[*bp + 2], buffer[*bp + 3]])
                ));
                *bp += 4;
            } else {
                buffer_out.push_str(&rm_to_rg(reg_mem));
                *bp += 2;
            }
            buffer_out.push(']');
        } else {
            // special case for direct address if reg is not the dest
            // 16 bit displacement follows
            if reg_mem == 0b0000_0110 {
                buffer_out.push_str(&format!(
                    "[{}",
                    u16::from_le_bytes([buffer[*bp + 2], buffer[*bp + 3]])
                ));
                *bp += 4;
            } else {
                buffer_out.push_str(&rm_to_rg(reg_mem));
                *bp += 2;
            }
            buffer_out.push_str("], ");
            buffer_out.push_str(WIDE_REGISTER_TABLE.get(&reg).unwrap());
        }
    } else {
        if reg_is_dest {
            buffer_out.push_str(REGISTER_TABLE.get(&reg).unwrap());
            buffer_out.push_str(", ");
            // special case for direct address
            if reg_mem == 0b0000_0110 {
                buffer_out.push_str(&format!("[{reg_mem:0}"));
            } else {
                buffer_out.push_str(&rm_to_rg(reg_mem));
            }
            buffer_out.push(']');
        } else {
            // special case for direct address
            if reg_mem == 0b0000_0110 {
                buffer_out.push_str(&format!("[{reg_mem:0}"));
            } else {
                buffer_out.push_str(&rm_to_rg(reg_mem));
            }
            buffer_out.push_str("], ");
            buffer_out.push_str(REGISTER_TABLE.get(&reg).unwrap());
        }
        *bp += 2;
    }
}

fn mem_mode_word_dis(buffer: &[u8], bp: &mut usize, buffer_out: &mut String) {
    let reg_is_dest = buffer[*bp] & REG_IS_DEST == REG_IS_DEST;
    let is_wide = buffer[*bp] & WIDE == WIDE;
    let reg = (buffer[*bp + 1] & 0b0011_1000) >> 3;
    let reg_mem = buffer[*bp + 1] & 0b0000_0111;
    if is_wide {
        if reg_is_dest {
            buffer_out.push_str(WIDE_REGISTER_TABLE.get(&reg).unwrap());
            buffer_out.push_str(", ");
            buffer_out.push_str(&rm_to_rg(reg_mem));
            buffer_out.push_str(&get_displacement_word([buffer[*bp + 2], buffer[*bp + 3]]));
            buffer_out.push(']');
        } else {
            buffer_out.push_str(&rm_to_rg(reg_mem));
            buffer_out.push_str(&get_displacement_word([buffer[*bp + 2], buffer[*bp + 3]]));
            buffer_out.push_str("], ");
            buffer_out.push_str(WIDE_REGISTER_TABLE.get(&reg).unwrap());
        }
    } else if reg_is_dest {
        buffer_out.push_str(REGISTER_TABLE.get(&reg).unwrap());
        buffer_out.push_str(", ");
        buffer_out.push_str(&rm_to_rg(reg_mem));
        buffer_out.push_str(&get_displacement_word([buffer[*bp + 2], buffer[*bp + 3]]));
        buffer_out.push(']');
    } else {
        buffer_out.push_str(&rm_to_rg(reg_mem));
        buffer_out.push_str(&get_displacement_word([buffer[*bp + 2], buffer[*bp + 3]]));
        buffer_out.push_str("], ");
        buffer_out.push_str(REGISTER_TABLE.get(&reg).unwrap());
    }
    *bp += 4;
}

fn mem_mode_byte_dis(buffer: &[u8], bp: &mut usize, buffer_out: &mut String) {
    let reg_is_dest = buffer[*bp] & REG_IS_DEST == REG_IS_DEST;
    let is_wide = buffer[*bp] & WIDE == WIDE;
    let reg = (buffer[*bp + 1] & 0b0011_1000) >> 3;
    let reg_mem = buffer[*bp + 1] & 0b0000_0111;
    if is_wide {
        if reg_is_dest {
            buffer_out.push_str(WIDE_REGISTER_TABLE.get(&reg).unwrap());
            buffer_out.push_str(", ");
            buffer_out.push_str(&rm_to_rg(reg_mem));
            if buffer[*bp + 2] != 0 {
                buffer_out.push_str(&get_displacement_byte(buffer[*bp + 2] as i8));
            }
            buffer_out.push(']');
        } else {
            buffer_out.push_str(&rm_to_rg(reg_mem));
            if buffer[*bp + 2] != 0 {
                buffer_out.push_str(&get_displacement_byte(buffer[*bp + 2] as i8));
            }
            buffer_out.push_str("], ");
            buffer_out.push_str(WIDE_REGISTER_TABLE.get(&reg).unwrap());
        }
    } else if reg_is_dest {
        buffer_out.push_str(REGISTER_TABLE.get(&reg).unwrap());
        buffer_out.push_str(", ");
        buffer_out.push_str(&rm_to_rg(reg_mem));
        if buffer[*bp + 2] != 0 {
            buffer_out.push_str(&get_displacement_byte(buffer[*bp + 2] as i8));
        }
        buffer_out.push(']');
    } else {
        buffer_out.push_str(&rm_to_rg(reg_mem));
        if buffer[*bp + 2] != 0 {
            buffer_out.push_str(&get_displacement_byte(buffer[*bp + 2] as i8));
        }
        buffer_out.push_str("], ");
        buffer_out.push_str(REGISTER_TABLE.get(&reg).unwrap());
    }
    *bp += 3;
}

fn reg_mode(buffer: &[u8], bp: &mut usize, buffer_out: &mut String) {
    let reg_is_dest = buffer[*bp] & REG_IS_DEST == REG_IS_DEST;
    let is_wide = buffer[*bp] & WIDE == WIDE;
    let reg = (buffer[*bp + 1] & 0b0011_1000) >> 3;
    let reg_mem = buffer[*bp + 1] & 0b0000_0111;
    if is_wide {
        if reg_is_dest {
            buffer_out.push_str(&format!(
                "{}, {}",
                WIDE_REGISTER_TABLE.get(&reg).unwrap(),
                WIDE_REGISTER_TABLE.get(&reg_mem).unwrap(),
            ));
        } else {
            buffer_out.push_str(&format!(
                "{}, {}",
                WIDE_REGISTER_TABLE.get(&reg_mem).unwrap(),
                WIDE_REGISTER_TABLE.get(&reg).unwrap()
            ));
        }
    } else if reg_is_dest {
        buffer_out.push_str(&format!(
            "{}, {}",
            REGISTER_TABLE.get(&reg).unwrap(),
            REGISTER_TABLE.get(&reg_mem).unwrap(),
        ));
    } else {
        buffer_out.push_str(&format!(
            "{}, {}",
            REGISTER_TABLE.get(&reg_mem).unwrap(),
            REGISTER_TABLE.get(&reg).unwrap()
        ));
    }
    // Advance two to account for the OP and register bytes
    *bp += 2;
}

pub fn disassemble(buffer: Vec<u8>, is_executing: bool) -> String {
    let mut buffer_out = String::from("bits 16 \n\n");

    // buffer pointer.
    let mut bp = 0;

    while bp < buffer.len() {
        if buffer[bp] >> 2 == MOV {
            buffer_out.push_str("mov ");

            if buffer[bp + 1] & REG_MODE == REG_MODE {
                reg_mode(&buffer, &mut bp, &mut buffer_out);
            } else if buffer[bp + 1] & MEM_MODE_BYTE_DIS == MEM_MODE_BYTE_DIS {
                mem_mode_byte_dis(&buffer, &mut bp, &mut buffer_out);
            } else if buffer[bp + 1] & MEM_MODE_WORD_DIS == MEM_MODE_WORD_DIS {
                mem_mode_word_dis(&buffer, &mut bp, &mut buffer_out);
            }
            // this must be last!!
            else if buffer[bp + 1] & MEM_MODE == MEM_MODE {
                mem_mode(&buffer, &mut bp, &mut buffer_out);
            }
        } else if buffer[bp] >> 1 == IMMEDIATE_TO_REG_OR_MEM {
            buffer_out.push_str("mov ");
            let is_wide = buffer[bp] & 1 == 1;
            let reg_mem = buffer[bp + 1] & 0b0000_0111;
            if buffer[bp + 1] & MEM_MODE_BYTE_DIS == MEM_MODE_BYTE_DIS {
                todo!()
            } else if buffer[bp + 1] & MEM_MODE_WORD_DIS == MEM_MODE_WORD_DIS {
                if is_wide {
                    buffer_out.push_str(&rm_to_rg(reg_mem));
                    buffer_out.push_str(&get_displacement_word([buffer[bp + 2], buffer[bp + 3]]));
                    buffer_out.push_str(&format!(
                        "], word {}",
                        i16::from_le_bytes([buffer[bp + 4], buffer[bp + 5]])
                    ));
                    bp += 6;
                } else {
                    buffer_out.push_str(&rm_to_rg(reg_mem));
                    buffer_out.push_str(&get_displacement_word([buffer[bp + 2], buffer[bp + 3]]));
                    buffer_out.push_str(&format!("], byte {}", buffer[bp + 4]));
                    bp += 5;
                }
            } else if buffer[bp + 1] & MEM_MODE == MEM_MODE {
                if is_wide {
                    // special case for direct address if reg is not the dest
                    // 16 bit displacement follows
                    if reg_mem == 0b0000_0110 {
                        buffer_out.push_str(&format!(
                            "[{}",
                            u16::from_le_bytes([reg_mem, buffer[bp + 2]])
                        ));
                    } else {
                        buffer_out.push_str(&rm_to_rg(reg_mem));
                    }
                    buffer_out.push_str(&format!(
                        "], word {}",
                        i16::from_le_bytes([buffer[bp + 2], buffer[bp + 3]])
                    ));
                    bp += 4;
                } else {
                    // special case for direct address
                    if reg_mem == 0b0000_0110 {
                        buffer_out.push_str(&format!(
                            "[{}",
                            u16::from_le_bytes([reg_mem, buffer[bp + 2]])
                        ));
                        bp += 4;
                    } else {
                        buffer_out.push_str(&rm_to_rg(reg_mem));
                    }
                    buffer_out.push_str(&format!("], byte {}", buffer[bp + 2]));
                    bp += 3;
                }
            }
        } else if buffer[bp] >> 4 == IMMEDITAE_TO_REG {
            buffer_out.push_str("mov ");

            // wide is different in immediate to register
            let is_wide = buffer[bp] & 0b0000_1000 == 0b0000_1000;

            let reg = buffer[bp] & 0b0000_0111;

            if is_wide {
                buffer_out.push_str(WIDE_REGISTER_TABLE.get(&reg).unwrap());
                buffer_out.push_str(", ");
                buffer_out.push_str(&format!(
                    "{}",
                    i16::from_le_bytes([buffer[bp + 1], buffer[bp + 2]])
                ));
                bp += 3;
            } else {
                buffer_out.push_str(REGISTER_TABLE.get(&reg).unwrap());
                buffer_out.push_str(", ");
                buffer_out.push_str(&format!("{}", buffer[bp + 1] as i8));
                bp += 2;
            }
        } else if buffer[bp] >> 1 == ACC_TO_MEM {
            buffer_out.push_str("mov ");
            // if it's wide
            if buffer[bp] & 1 == 1 {
                buffer_out.push_str(&format!(
                    "[{}], ax",
                    u16::from_le_bytes([buffer[bp + 1], buffer[bp + 2]])
                ));
                bp += 3;
            } else {
                buffer_out.push_str(&format!("[{}], al", buffer[bp + 1]));
                bp += 2;
            }
        } else if buffer[bp] >> 5 == MEM_TO_ACC {
            // if it's wide
            if buffer[bp] & 1 == 1 {
                buffer_out.push_str(&format!(
                    "mov ax, [{}]",
                    u16::from_le_bytes([buffer[bp + 1], buffer[bp + 2]])
                ));
                bp += 3;
            } else {
                buffer_out.push_str(&format!("mov al, [{}]", buffer[bp + 1]));
                bp += 2;
            }
        } else if buffer[bp] >> 2 == 0b0
            || buffer[bp] >> 2 == 0b001010
            || buffer[bp] >> 2 == 0b001110
        {
            // SUB
            if buffer[bp] >> 2 == 0b001010 {
                buffer_out.push_str("sub ");
            }
            // CMP
            else if buffer[bp] >> 2 == 0b001110 {
                buffer_out.push_str("cmp ");
            }
            // ADD - Must be last! Always true essentially
            else if buffer[bp] & ADD == ADD {
                buffer_out.push_str("add ");
            }
            match Mode::from(buffer[bp + 1]) {
                Mode::Mem => {
                    mem_mode(&buffer, &mut bp, &mut buffer_out);
                }
                Mode::MemByteDis => {
                    mem_mode_byte_dis(&buffer, &mut bp, &mut buffer_out);
                }
                Mode::MemWordDis => {
                    mem_mode_word_dis(&buffer, &mut bp, &mut buffer_out);
                }
                Mode::Reg => {
                    reg_mode(&buffer, &mut bp, &mut buffer_out);
                }
            }
        }
        // Immediate to reg/mem
        else if buffer[bp] >> 2 == 0b0010_0000 {
            if buffer[bp + 1] >> 3 & 0b00111 == 0 {
                buffer_out.push_str("add ");
            }
            if buffer[bp + 1] >> 3 & 0b00111 == 0b00101 {
                buffer_out.push_str("sub ");
            }
            if buffer[bp + 1] >> 3 & 0b00111 == 0b00111 {
                buffer_out.push_str("cmp ");
            }
            let is_wide = buffer[bp] & 1 == 1;
            let is_signed = buffer[bp] & 0b0000_0010 == 0b0000_0010;
            let reg_mem = buffer[bp + 1] & 0b0000_0111;
            match Mode::from(buffer[bp + 1]) {
                Mode::MemWordDis => {
                    if is_wide {
                        if is_signed {
                            buffer_out.push_str("word ");
                            buffer_out.push_str(&rm_to_rg(reg_mem));
                            buffer_out
                                .push_str(&get_displacement_word([buffer[bp + 2], buffer[bp + 3]]));
                            buffer_out.push_str(&format!("], {}", buffer[bp + 4]));
                            bp += 5;
                        } else {
                            buffer_out.push_str("word ");
                            buffer_out.push_str(&rm_to_rg(reg_mem));
                            buffer_out
                                .push_str(&get_displacement_word([buffer[bp + 2], buffer[bp + 3]]));
                            buffer_out.push_str(&format!(
                                "], {}",
                                u16::from_le_bytes([buffer[bp + 4], buffer[bp + 5]])
                            ));
                            bp += 6;
                        }
                    } else {
                        // TODO: add signed variants
                        buffer_out.push_str("byte ");
                        buffer_out.push_str(&rm_to_rg(reg_mem));
                        buffer_out
                            .push_str(&get_displacement_word([buffer[bp + 2], buffer[bp + 3]]));
                        buffer_out.push_str(&format!("], byte {}", buffer[bp + 4]));
                        bp += 5;
                    }
                }
                Mode::MemByteDis => {
                    if is_wide {
                        buffer_out.push_str(&rm_to_rg(reg_mem));
                        buffer_out.push_str(&get_displacement_byte(buffer[bp + 2] as i8));
                        buffer_out.push_str(&format!(
                            "], word {}",
                            i16::from_le_bytes([buffer[bp + 4], buffer[bp + 5]])
                        ));
                        bp += 5;
                    } else {
                        buffer_out.push_str(&rm_to_rg(reg_mem));
                        buffer_out.push_str(&get_displacement_byte(buffer[bp + 2] as i8));
                        buffer_out.push_str(&format!("], byte {}", buffer[bp + 3]));
                        bp += 4;
                    }
                }
                Mode::Reg => {
                    let is_wide = buffer[bp] & WIDE == WIDE;
                    let reg_mem = buffer[bp + 1] & 0b0000_0111;
                    if is_wide {
                        if is_signed {
                            buffer_out.push_str(WIDE_REGISTER_TABLE.get(&reg_mem).unwrap());
                            buffer_out.push_str(", ");
                            buffer_out.push_str(&format!("{}", buffer[bp + 2]));
                            bp += 3;
                        } else {
                            buffer_out.push_str(WIDE_REGISTER_TABLE.get(&reg_mem).unwrap());
                            buffer_out.push_str(", ");
                            buffer_out.push_str(&format!(
                                "{}",
                                u16::from_le_bytes([buffer[bp + 2], buffer[bp + 3]])
                            ));
                            bp += 4;
                        }
                    } else {
                        bp += 2;
                    }
                    // Advance two to account for the OP and register bytes
                }
                Mode::Mem => {
                    let is_wide = buffer[bp] & WIDE == WIDE;
                    let reg_mem = buffer[bp + 1] & 0b0000_0111;
                    if is_wide {
                        if is_signed {
                            buffer_out.push_str("word ");
                            if reg_mem == 0b0000_0110 {
                                buffer_out.push_str(&format!(
                                    "[{}], ",
                                    u16::from_le_bytes([buffer[bp + 2], buffer[bp + 3]])
                                ));
                                buffer_out.push_str(&format!("{}", buffer[bp + 4]));
                                bp += 5;
                            } else {
                                buffer_out.push_str(&rm_to_rg(reg_mem));
                                buffer_out.push_str(&format!("], {}", buffer[bp + 2]));
                                bp += 3;
                            }
                        } else {
                            buffer_out.push_str("word ");
                            // special case for direct address if reg is not the dest
                            // 16 bit displacement follows
                            if reg_mem == 0b0000_0110 {
                                buffer_out.push_str(&format!(
                                    "[{}",
                                    u16::from_le_bytes([buffer[bp + 2], buffer[bp + 3]])
                                ));
                                buffer_out.push_str(&format!(
                                    "{}",
                                    u16::from_le_bytes([buffer[bp + 4], buffer[bp + 5]])
                                ));
                                bp += 6;
                            } else {
                                buffer_out.push_str(&rm_to_rg(reg_mem));
                                buffer_out.push_str(&format!(
                                    "{}",
                                    u16::from_le_bytes([buffer[bp + 2], buffer[bp + 3]])
                                ));
                                bp += 4;
                            }
                        }
                    } else if is_signed {
                        buffer_out.push_str("byte ");
                        // special case for direct address
                        if reg_mem == 0b0000_0110 {
                            buffer_out.push_str(&format!("[{reg_mem:0}"));
                        } else {
                            buffer_out.push_str(&rm_to_rg(reg_mem));
                        }
                        buffer_out.push_str(&format!(", {}", buffer[bp + 2] as i8));
                        bp += 3;
                    } else {
                        buffer_out.push_str("byte ");
                        // special case for direct address
                        if reg_mem == 0b0000_0110 {
                            buffer_out.push_str(&format!("[{reg_mem:0}"));
                        } else {
                            buffer_out.push_str(&rm_to_rg(reg_mem));
                        }
                        buffer_out.push_str(&format!("], {}", buffer[bp + 2]));
                        bp += 3;
                    }
                }
            }
        }
        // Immediate to acc
        else if buffer[bp] >> 1 == 0b0000_0010
            || buffer[bp] >> 1 == 0b0000_10110
            || buffer[bp] >> 1 == 0b0000_11110
        {
            if buffer[bp] >> 1 == 0b0000_0010 {
                buffer_out.push_str("add ");
            }
            if buffer[bp] >> 1 == 0b0001_0110 {
                buffer_out.push_str("sub ");
            }
            if buffer[bp] >> 1 == 0b0001_1110 {
                buffer_out.push_str("cmp ");
            }
            let is_wide = buffer[bp] & 1 == 1;
            if is_wide {
                buffer_out.push_str(&format!(
                    "ax, {}",
                    i16::from_le_bytes([buffer[bp + 1], buffer[bp + 2]])
                ));
                bp += 3;
            } else {
                buffer_out.push_str(&format!("al, {}", buffer[bp + 1] as i8));
                bp += 2;
            }
        } else if buffer[bp] >> 1 == 0b0001_0110 {
            buffer_out.push_str("sub ");
            let is_wide = buffer[bp] & 1 == 1;
            if is_wide {
                buffer_out.push_str(&format!(
                    "ax, {}",
                    i16::from_le_bytes([buffer[bp + 1], buffer[bp + 2]])
                ));
                bp += 3;
            } else {
                buffer_out.push_str(&format!("al, {}", buffer[bp + 1] as i8));
                bp += 2;
            }
        }
        // JNE - jump not equal
        else if buffer[bp] == 0b0111_0101 {
            buffer_out.push_str(&format!("jne {}", buffer[bp + 1] as i8));
            bp += 2;
        }
        // JE - jump equal
        else if buffer[bp] == 0b0111_0100 {
            buffer_out.push_str(&format!("je {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_1100 {
            buffer_out.push_str(&format!("jl {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_1110 {
            buffer_out.push_str(&format!("jle {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_0010 {
            buffer_out.push_str(&format!("jb {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_0110 {
            buffer_out.push_str(&format!("jbe {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b111_1010 {
            buffer_out.push_str(&format!("jp {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_0000 {
            buffer_out.push_str(&format!("jo {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_1000 {
            buffer_out.push_str(&format!("js {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_1101 {
            buffer_out.push_str(&format!("jnl {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_1111 {
            buffer_out.push_str(&format!("jnle {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_0011 {
            buffer_out.push_str(&format!("jnb {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_0111 {
            buffer_out.push_str(&format!("jnbe {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_1011 {
            buffer_out.push_str(&format!("jnp {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_0001 {
            buffer_out.push_str(&format!("jno {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b0111_1001 {
            buffer_out.push_str(&format!("jns {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b1110_0010 {
            buffer_out.push_str(&format!("loop {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b1110_0001 {
            buffer_out.push_str(&format!("loopz {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b1110_0000 {
            buffer_out.push_str(&format!("loopnz {}", buffer[bp + 1] as i8));
            bp += 2;
        } else if buffer[bp] == 0b1110_0011 {
            buffer_out.push_str(&format!("jcxz {}", buffer[bp + 1] as i8));
            bp += 2;
        } else {
            // while developing to prevent endless looping when nothing matches
            bp += 2;
        }
        buffer_out.push('\n');
    }

    buffer_out
}
