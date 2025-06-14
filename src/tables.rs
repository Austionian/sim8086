use std::{cell::RefCell, collections::HashMap, sync::LazyLock};

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

struct Register {
    value: u16,
}

// actual register values
const _AX: RefCell<Register> = RefCell::new(Register { value: 0x00 });
const _BX: RefCell<Register> = RefCell::new(Register { value: 0x00 });
const _CX: RefCell<Register> = RefCell::new(Register { value: 0x00 });
const _DX: RefCell<Register> = RefCell::new(Register { value: 0x00 });
const _SP: RefCell<Register> = RefCell::new(Register { value: 0x00 });
const _BP: RefCell<Register> = RefCell::new(Register { value: 0x00 });
const _SI: RefCell<Register> = RefCell::new(Register { value: 0x00 });
const _DI: RefCell<Register> = RefCell::new(Register { value: 0x00 });

enum Registers {
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
    fn update_wide(&self, value: u16) {
        match self {
            Self::_AX => _AX.borrow_mut().value = value,
            Self::_BX => _BX.borrow_mut().value = value,
            Self::_CX => _CX.borrow_mut().value = value,
            Self::_DX => _DX.borrow_mut().value = value,
            Self::_SP => _SP.borrow_mut().value = value,
            Self::_BP => _BP.borrow_mut().value = value,
            Self::_SI => _SI.borrow_mut().value = value,
            Self::_DI => _DI.borrow_mut().value = value,
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

    fn update(&self, value: u8) {
        match self {
            Self::_AL => _AX.borrow_mut().set_low(value),
            Self::_AH => _AX.borrow_mut().set_high(value),
            Self::_BL => _BX.borrow_mut().set_low(value),
            Self::_BH => _BX.borrow_mut().set_high(value),
            Self::_CL => _CX.borrow_mut().set_low(value),
            Self::_CH => _CX.borrow_mut().set_high(value),
            Self::_DL => _DX.borrow_mut().set_low(value),
            Self::_DH => _DX.borrow_mut().set_high(value),
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
}

impl Register {
    fn to_string(&self) -> &'static str {
        "test"
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
