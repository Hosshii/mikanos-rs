use super::endian::{EndianFrom, EndianInto};
use macros::bitfield_struct;

pub(crate) const fn check_size<T>(size: usize) {
    if core::mem::size_of::<T>() != size {
        panic!("size unmatced")
    }
}

const _: () = check_size::<TrbRaw>(16);

bitfield_struct! {
    /// FFI types.
    /// fields are little endian.
    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[endian = "little"]
    pub struct TrbRaw {
        parameter0: u32,
        parameter1: u32,
        status: u32,
        remain: u16 => {
            #[bits(1)]
            cycle_bit: bool,
            #[bits(1)]
            evaluate_next_trb: bool,
            #[bits(8)]
            remain: u8,
            #[bits(6)]
            trb_type: TrbType,
        },
        control: u16,
    }
}

impl TrbRaw {
    pub fn new(parameter0: u32, parameter1: u32, status: u32, control: u16, remain: u16) -> Self {
        Self::default()
            .with_parameter0(parameter0)
            .with_parameter1(parameter1)
            .with_status(status)
            .with_control(control)
            .with_remain(remain)
    }

    pub fn zeroed() -> Self {
        Self::new(0, 0, 0, 0, 0)
    }
}

impl From<Link> for TrbRaw {
    fn from(_value: Link) -> Self {
        todo!()
    }
}

impl From<EnableSlotCommand> for TrbRaw {
    fn from(_value: EnableSlotCommand) -> Self {
        todo!()
    }
}

impl From<Trb> for TrbRaw {
    fn from(_value: Trb) -> Self {
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
            Unknown(x) => x,
        }
    }
}

impl EndianInto<u16> for TrbType {
    fn to_le(self) -> u16 {
        self.to_ne().to_le()
    }

    fn to_be(self) -> u16 {
        self.to_ne().to_be()
    }

    fn to_ne(self) -> u16 {
        self.as_u8() as u16
    }
}

impl EndianFrom<u16> for TrbType {
    fn from_le(v: u16) -> Self {
        Self::from_ne(u16::from_le(v))
    }

    fn from_be(v: u16) -> Self {
        Self::from_ne(u16::from_be(v))
    }

    fn from_ne(v: u16) -> Self {
        Self::from_u8(v as u8)
    }
}

bitfield_struct! {
    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[endian = "little"]
    pub struct Link {
        ring_segment_pointer_lo: u32 => {
            #[bits(4)]
            _rsvdz: u8,
            #[bits(28)]
            data: u32,
        },
        ring_segment_pointer_hi: u32,
        status: u32 => {
            #[bits(22)]
            _rsvdz: u32,
            #[bits(10)]
            interrupter_target: u16,
        },
        remain: u16 => {
            #[bits(1)]
            cycle_bit: bool,
            #[bits(1)]
            toggle_cycle: bool,
            #[bits(2)]
            _rsvdz1: u8,
            #[bits(1)]
            chain_bit: bool,
            #[bits(1)]
            interrupt_on_completion: bool,
            #[bits(4)]
            _rsvdz2: u8,
            #[bits(6)]
            trb_type: TrbType,
        },
        _rsvdz: u16,

    }
}

impl Link {
    pub const TYPE: TrbType = TrbType::Link;

    pub fn new(segment_ptr: *const ()) -> Self {
        let raw_ptr = segment_ptr as u64;
        let lo = raw_ptr as u32;
        let hi = (raw_ptr >> 32) as u32;

        Self::default()
            .with_remain_trb_type(Self::TYPE)
            .with_ring_segment_pointer_hi(hi)
            .with_ring_segment_pointer_lo_data(lo)
    }
}

impl Type for Link {
    fn get_type(self) -> TrbType {
        Self::TYPE
    }
}

bitfield_struct! {
    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[endian = "little"]
    pub struct EnableSlotCommand {
        _rsvdz1: u32,
        _rsvdz2: u32,
        _rsvdz3: u32,
        remain: u16 => {
            #[bits(1)]
            cycle_bit: bool,
            #[bits(9)]
            _rsvdz1: u16,
            #[bits(6)]
            trb_type: TrbType,
        },
        control: u16 => {
            #[bits(5)]
            slot_type: u8,
            #[bits(11)]
            _rsvdz: u16,
        }
    }
}

impl EnableSlotCommand {
    pub const TYPE: TrbType = TrbType::EnableSlotCommand;
}

impl Type for EnableSlotCommand {
    fn get_type(self) -> TrbType {
        Self::TYPE
    }
}

bitfield_struct! {
    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[endian = "little"]
    pub struct CommandCompletionEvent {
        params: u64 => {
            #[bits(4)]
            _rsvdz: u8,
            #[bits(60)]
            ptr: u64
        },
        status: u32 => {
            #[bits(24)]
            command_completion_parameter: u32,
            #[bits(8)]
            completion_code: u8,
        },
        remain: u16 => {
            #[bits(1)]
            cycle_bit: bool,
            #[bits(9)]
            _rsvdz: u16,
            #[bits(6)]
            trb_type: TrbType,
        },
        control: u16 => {
            #[bits(8)]
            vf_id: u8,
            #[bits(8)]
            slot_id: u8,
        }
    }
}

impl CommandCompletionEvent {
    pub const TYPE: TrbType = TrbType::CommandConpletionEvent;

    /// # Safety
    /// issuer ptr must be valid
    pub unsafe fn issuer(self) -> Trb {
        let ptr = self.get_params_ptr() as *const TrbRaw;
        Trb::from(unsafe { *ptr })
    }
}

impl Type for CommandCompletionEvent {
    fn get_type(self) -> TrbType {
        Self::TYPE
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Trb {
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
    CommandCompletionEvent(CommandCompletionEvent),
    PortStatusChangeEvent,
    Unknown(u8),
}

impl From<TrbRaw> for Trb {
    fn from(_value: TrbRaw) -> Self {
        todo!()
    }
}

impl Type for Trb {
    fn get_type(self) -> TrbType {
        match self {
            Trb::Normal => todo!(),
            Trb::SetupStage => todo!(),
            Trb::DataStage => todo!(),
            Trb::StatusStage => todo!(),
            Trb::Link(_) => Link::TYPE,
            Trb::NoOp => todo!(),
            Trb::EnableSlotCommand => todo!(),
            Trb::AddressDeviceCommand => todo!(),
            Trb::ConfigureEndpoint => todo!(),
            Trb::NoOpCommand => todo!(),
            Trb::TransferEvent => todo!(),
            Trb::CommandCompletionEvent(_) => CommandCompletionEvent::TYPE,
            Trb::PortStatusChangeEvent => todo!(),
            Trb::Unknown(x) => TrbType::Unknown(x),
        }
    }
}

trait Type {
    fn get_type(self) -> TrbType;
}
