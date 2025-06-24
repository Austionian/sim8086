use once_cell::unsync::Lazy;
use std::{cell::RefCell, collections::HashMap, fmt::Display, sync::LazyLock};

// Registers
const AL: u8 = 0b0000_0000;
const CL: u8 = 0b0000_0001;
const DL: u8 = 0b0000_0010;
const BL: u8 = 0b0000_0011;
const AH: u8 = 0b0000_0100;
const CH: u8 = 0b0000_0101;
const DH: u8 = 0b0000_0110;
const BH: u8 = 0b0000_0111;

// Wide registers
const AX: u8 = 0b0000_0000;
const CX: u8 = 0b0000_0001;
const DX: u8 = 0b0000_0010;
const BX: u8 = 0b0000_0011;
const SP: u8 = 0b0000_0100;
const BP: u8 = 0b0000_0101;
const SI: u8 = 0b0000_0110;
const DI: u8 = 0b0000_0111;

pub static REGISTER_TABLE: LazyLock<HashMap<u8, Registers>> = LazyLock::new(|| {
    let mut register_table = HashMap::new();

    register_table.insert(AL, Registers::_AL);
    register_table.insert(BL, Registers::_BL);
    register_table.insert(CL, Registers::_CL);
    register_table.insert(DL, Registers::_DL);
    register_table.insert(AH, Registers::_AH);
    register_table.insert(BH, Registers::_BH);
    register_table.insert(CH, Registers::_CH);
    register_table.insert(DH, Registers::_DH);

    register_table
});

pub static WIDE_REGISTER_TABLE: LazyLock<HashMap<u8, Registers>> = LazyLock::new(|| {
    let mut wide_register_table = HashMap::new();

    wide_register_table.insert(AX, Registers::_AX);
    wide_register_table.insert(CX, Registers::_CX);
    wide_register_table.insert(DX, Registers::_DX);
    wide_register_table.insert(BX, Registers::_BX);
    wide_register_table.insert(SP, Registers::_SP);
    wide_register_table.insert(BP, Registers::_BP);
    wide_register_table.insert(SI, Registers::_SI);
    wide_register_table.insert(DI, Registers::_DI);

    wide_register_table
});

#[derive(Debug)]
pub struct Register {
    pub value: u16,
}

thread_local! {
    // actual register values
    static _AX: Lazy<RefCell<Register>> = Lazy::new(|| RefCell::new(Register { value: 0x00 }));
    static _BX: Lazy<RefCell<Register>> = Lazy::new(|| RefCell::new(Register { value: 0x00 }));
    static _CX: Lazy<RefCell<Register>> = Lazy::new(|| RefCell::new(Register { value: 0x00 }));
    static _DX: Lazy<RefCell<Register>> = Lazy::new(|| RefCell::new(Register { value: 0x00 }));
    static _SP: Lazy<RefCell<Register>> = Lazy::new(|| RefCell::new(Register { value: 0x00 }));
    static _BP: Lazy<RefCell<Register>> = Lazy::new(|| RefCell::new(Register { value: 0x00 }));
    static _SI: Lazy<RefCell<Register>> = Lazy::new(|| RefCell::new(Register { value: 0x00 }));
    static _DI: Lazy<RefCell<Register>> = Lazy::new(|| RefCell::new(Register { value: 0x00 }));
}

#[derive(Debug)]
pub enum Registers {
    _AX,
    _BX,
    _CX,
    _DX,
    _SP,
    _BP,
    _SI,
    _DI,
    _AL,
    _AH,
    _BL,
    _BH,
    _CL,
    _CH,
    _DL,
    _DH,
}

impl Registers {
    pub fn update_wide(&self, value: u16) {
        match self {
            Self::_AX => _AX.with(|register| register.borrow_mut().set(value)),
            Self::_BX => _BX.with(|register| register.borrow_mut().set(value)),
            Self::_CX => _CX.with(|register| register.borrow_mut().set(value)),
            Self::_DX => _DX.with(|register| register.borrow_mut().set(value)),
            Self::_SP => _SP.with(|register| register.borrow_mut().set(value)),
            Self::_BP => _BP.with(|register| register.borrow_mut().set(value)),
            Self::_SI => _SI.with(|register| register.borrow_mut().set(value)),
            Self::_DI => _DI.with(|register| register.borrow_mut().set(value)),
            Self::_AL
            | Self::_AH
            | Self::_BL
            | Self::_BH
            | Self::_CL
            | Self::_CH
            | Self::_DL
            | Self::_DH => panic!("can't set 8 bit register with a u16"),
        }
    }

    pub fn update(&self, value: u8) {
        match self {
            Self::_AL => _AX.with(|register| register.borrow_mut().set_low(value)),
            Self::_AH => _AX.with(|register| register.borrow_mut().set_high(value)),
            Self::_BL => _BX.with(|register| register.borrow_mut().set_low(value)),
            Self::_BH => _BX.with(|register| register.borrow_mut().set_high(value)),
            Self::_CL => _CX.with(|register| register.borrow_mut().set_low(value)),
            Self::_CH => _CX.with(|register| register.borrow_mut().set_high(value)),
            Self::_DL => _DX.with(|register| register.borrow_mut().set_low(value)),
            Self::_DH => _DX.with(|register| register.borrow_mut().set_high(value)),
            Self::_AX
            | Self::_BX
            | Self::_CX
            | Self::_DX
            | Self::_SP
            | Self::_BP
            | Self::_SI
            | Self::_DI => panic!("can't set a 16 bit register with a u8"),
        }
    }

    pub fn get_value(&self) -> u16 {
        match self {
            Self::_DI => _DI.with(|register| register.borrow().value),
            Self::_SI => _SI.with(|register| register.borrow().value),
            Self::_AX => _AX.with(|register| register.borrow().value),
            Self::_BX => _BX.with(|register| register.borrow().value),
            Self::_CX => _CX.with(|register| register.borrow().value),
            Self::_DX => _DX.with(|register| register.borrow().value),
            Self::_AL => _AX.with(|register| register.borrow().value),
            Self::_AH => _AX.with(|register| register.borrow().value),
            Self::_BL => _BX.with(|register| register.borrow().value),
            Self::_BH => _BX.with(|register| register.borrow().value),
            Self::_CL => _CX.with(|register| register.borrow().value),
            Self::_CH => _CX.with(|register| register.borrow().value),
            Self::_DL => _DX.with(|register| register.borrow().value),
            Self::_DH => _DX.with(|register| register.borrow().value),
            Self::_SP => _SP.with(|register| register.borrow().value),
            Self::_BP => _BP.with(|register| register.borrow().value),
        }
    }

    pub fn updated_value(&self) -> String {
        match self {
            Self::_DI => format!("di {:#x}", _DI.with(|register| register.borrow().value)),
            Self::_SI => format!("si {:#x}", _SI.with(|register| register.borrow().value)),
            Self::_AX => format!("ax {:#x}", _AX.with(|register| register.borrow().value)),
            Self::_BX => format!("bx {:#x}", _BX.with(|register| register.borrow().value)),
            Self::_CX => format!("cx {:#x}", _CX.with(|register| register.borrow().value)),
            Self::_DX => format!("dx {:#x}", _DX.with(|register| register.borrow().value)),
            Self::_AL => format!("al {:#x}", _AX.with(|register| register.borrow().value)),
            Self::_AH => format!("ah {:#x}", _AX.with(|register| register.borrow().value)),
            Self::_BL => format!("bl {:#x}", _BX.with(|register| register.borrow().value)),
            Self::_BH => format!("bh {:#x}", _BX.with(|register| register.borrow().value)),
            Self::_CL => format!("cl {:#x}", _CX.with(|register| register.borrow().value)),
            Self::_CH => format!("ch {:#x}", _CX.with(|register| register.borrow().value)),
            Self::_DL => format!("dl {:#x}", _DX.with(|register| register.borrow().value)),
            Self::_DH => format!("dh {:#x}", _DX.with(|register| register.borrow().value)),
            Self::_SP => format!("sp {:#x}", _SP.with(|register| register.borrow().value)),
            Self::_BP => format!("bp {:#x}", _BP.with(|register| register.borrow().value)),
        }
    }

    pub fn print() {
        println!("ax: {:#04x}", _AX.with(|register| register.borrow().value));
        println!("bx: {:#04x}", _BX.with(|register| register.borrow().value));
        println!("cx: {:#04x}", _CX.with(|register| register.borrow().value));
        println!("dx: {:#04x}", _DX.with(|register| register.borrow().value));
        println!("sp: {:#04x}", _SP.with(|register| register.borrow().value));
        println!("bp: {:#04x}", _BP.with(|register| register.borrow().value));
        println!("si: {:#04x}", _SI.with(|register| register.borrow().value));
        println!("di: {:#04x}", _DI.with(|register| register.borrow().value));
        println!("\n\n");
    }
}

impl Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::_DI => write!(f, "di"),
            Self::_SI => write!(f, "si"),
            Self::_AX => write!(f, "ax"),
            Self::_BX => write!(f, "bx"),
            Self::_CX => write!(f, "cx"),
            Self::_DX => write!(f, "dx"),
            Self::_AL => write!(f, "al"),
            Self::_AH => write!(f, "ah"),
            Self::_BL => write!(f, "bl"),
            Self::_BH => write!(f, "bh"),
            Self::_CL => write!(f, "cl"),
            Self::_CH => write!(f, "ch"),
            Self::_DL => write!(f, "dl"),
            Self::_DH => write!(f, "dh"),
            Self::_SP => write!(f, "sp"),
            Self::_BP => write!(f, "bp"),
        }
    }
}

impl Register {
    fn set(&mut self, value: u16) {
        self.value = value;
    }

    // little endian
    fn set_low(&mut self, value: u8) {
        self.value &= !0xff00;
        self.value |= (value as u16) << 8;
    }

    fn set_high(&mut self, value: u8) {
        self.value &= !0xff;
        self.value |= value as u16
    }
}
