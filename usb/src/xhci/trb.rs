#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Bit {
    Zero,
    One,
}

impl Bit {
    pub fn as_byte(self) -> u8 {
        match self {
            Bit::Zero => 0,
            Bit::One => 1,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TRBBase {
    parameter: u64,
    status: u32,
    control: u16,
    // 15 ~ 10: trb type
    // 9  ~ 1 :
    // 0  ~ 0 : circle bit
    remain: u16,
}

impl TRBBase {
    pub fn set_circle(mut self, bit: Bit) -> Self {
        self.remain |= bit.as_byte() as u16;
        self
    }

    pub fn trb_type(&self) -> TrbType {
        let ty = self.remain >> 10;
        TrbType::from_u8(ty as u8)
    }
}

impl From<Link> for TRBBase {
    fn from(value: Link) -> Self {
        todo!()
    }
}

impl From<TRB> for TRBBase {
    fn from(value: TRB) -> Self {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TrbType {
    Normal,
    SetupStage,
    DataStage,
    StatusStage,
    Link,
    NoOp,
    EnableSlotCommand,
    AddressDeviceCommand,
    ConfigureEndpoint,
    NoOpCommand,
    TransferEvent,
    CommandConpletionEvent,
    PortStatusChangeEvent,
    Unknown(u8),
}

impl TrbType {
    pub fn from_u8(v: u8) -> Self {
        use TrbType::*;
        match v {
            1 => Normal,
            2 => SetupStage,
            3 => DataStage,
            4 => StatusStage,
            6 => Link,
            8 => NoOp,
            10 => EnableSlotCommand,
            11 => AddressDeviceCommand,
            12 => ConfigureEndpoint,
            23 => NoOpCommand,
            32 => TransferEvent,
            33 => CommandConpletionEvent,
            34 => PortStatusChangeEvent,
            x => Unknown(x),
        }
    }

    pub fn as_u8(self) -> u8 {
        use TrbType::*;
        match self {
            Normal => 1,
            SetupStage => 2,
            DataStage => 3,
            StatusStage => 4,
            Link => 6,
            NoOp => 8,
            EnableSlotCommand => 10,
            AddressDeviceCommand => 11,
            ConfigureEndpoint => 12,
            NoOpCommand => 23,
            TransferEvent => 32,
            CommandConpletionEvent => 33,
            PortStatusChangeEvent => 34,
            Unknown(x) => todo!(),
        }
    }
}

pub struct Link {}

pub enum TRB {
    Normal,
    SetupStage,
    DataStage,
    StatusStage,
    Link(Link),
    NoOp,
    EnableSlotCommand,
    AddressDeviceCommand,
    ConfigureEndpoint,
    NoOpCommand,
    TransferEvent,
    CommandConpletionEvent,
    PortStatusChangeEvent,
    Unknown(u8),
}

impl From<TRBBase> for TRB {
    fn from(value: TRBBase) -> Self {
        todo!()
    }
}
