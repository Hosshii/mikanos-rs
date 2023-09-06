use super::endian::{LeU16, LeU32, LeU64};

/// FFI types.
/// fields are little endian.
#[repr(C, packed)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TRBRaw {
    parameter: LeU64,
    status: LeU32,
    control: LeU16,
    // 15 ~ 10: trb type
    // 9  ~ 1 :
    // 0  ~ 0 : circle bit
    remain: LeU16,
}

impl TRBRaw {
    pub fn new(parameter: u64, status: u32, control: u16, remain: u16) -> Self {
        Self {
            parameter: LeU64::from(parameter),
            status: LeU32::from(status),
            control: LeU16::from(control),
            remain: LeU16::from(remain),
        }
    }
}

impl From<Link> for TRBRaw {
    fn from(value: Link) -> Self {
        todo!()
    }
}

impl From<TRB> for TRBRaw {
    fn from(value: TRB) -> Self {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TrbType {
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

pub struct Link {
    ring_segment_pointer_lo: u32, // only 28 bits are used. least 4 bits are not used.
    ring_segment_pointer_hi: u32,
    interrupter_target: u16, // Only 10 bits are used, so masking will be necessary.
    cycle_bit: bool,
    toggle_cycle: bool,
    chain_bit: bool,
    interrupt_on_completion: bool,
    trb_type: TrbType,
}

impl Link {
    pub const TYPE: TrbType = TrbType::Link;

    pub fn new(segment_ptr: *const ()) -> Self {
        let raw_ptr = (segment_ptr as u64) & 0xFFFFFFFFFFFFFFF0;
        let lo = raw_ptr as u32;
        let hi = (raw_ptr >> 32) as u32;

        Self {
            ring_segment_pointer_lo: lo,
            ring_segment_pointer_hi: hi,
            interrupter_target: 0,
            cycle_bit: false,
            toggle_cycle: false,
            chain_bit: false,
            interrupt_on_completion: false,
            trb_type: Self::TYPE,
        }
    }

    pub fn set_cycle(mut self, v: bool) -> Self {
        self.cycle_bit = v;
        self
    }

    pub fn set_toggle(mut self, v: bool) -> Self {
        self.toggle_cycle = v;
        self
    }
}

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

impl From<TRBRaw> for TRB {
    fn from(value: TRBRaw) -> Self {
        todo!()
    }
}
