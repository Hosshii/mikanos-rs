use super::endian::{EndianFrom, EndianInto};
use common::Zeroed;
use macros::bitfield_struct;

#[allow(dead_code)]
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
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroed)]
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
        Self::zeroed()
            .with_parameter0(parameter0)
            .with_parameter1(parameter1)
            .with_status(status)
            .with_control(control)
            .with_remain(remain)
    }
}

impl From<SetupStage> for TrbRaw {
    fn from(value: SetupStage) -> Self {
        Self::zeroed()
            .with_parameter0(value.parameter0)
            .with_parameter1(value.parameter1)
            .with_status(value.status)
            .with_remain(value.remain)
            .with_control(value.control)
    }
}

impl From<DataStage> for TrbRaw {
    fn from(value: DataStage) -> Self {
        Self::zeroed()
            .with_parameter0(value.buf_ptr_lo)
            .with_parameter1(value.buf_ptr_hi)
            .with_status(value.status)
            .with_remain(value.remain)
            .with_control(value.control)
    }
}

impl From<StatusStage> for TrbRaw {
    fn from(value: StatusStage) -> Self {
        Self::zeroed()
            .with_parameter0(value._rsvdz1)
            .with_parameter1(value._rsvdz2)
            .with_status(value.status)
            .with_remain(value.remain)
            .with_control(value.control)
    }
}

impl From<Link> for TrbRaw {
    fn from(_value: Link) -> Self {
        todo!()
    }
}

impl From<EnableSlotCommand> for TrbRaw {
    fn from(value: EnableSlotCommand) -> Self {
        Self::zeroed()
            .with_parameter0(value._rsvdz1)
            .with_parameter1(value._rsvdz2)
            .with_status(value._rsvdz3)
            .with_remain(value.remain)
            .with_control(value.control)
    }
}

impl From<AddressDeviceCommand> for TrbRaw {
    fn from(value: AddressDeviceCommand) -> Self {
        Self::zeroed()
            .with_parameter0(value.params as u32)
            .with_parameter1((value.params >> 32) as u32)
            .with_status(value._rsvdz)
            .with_remain(value.remain)
            .with_control(value.control)
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
            9 => EnableSlotCommand,
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
            EnableSlotCommand => 9,
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
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroed)]
    #[endian = "little"]
    pub struct SetupStage {
        parameter0: u32 => {
            #[bits(8)]
            bm_request_type: u8,
            #[bits(8)]
            b_ruquest: u8,
            #[bits(16)]
            w_value: u16,
        },
        parameter1: u32 => {
            #[bits(16)]
            w_index: u16,
            #[bits(16)]
            w_length: u16,
        },
        status: u32 => {
            #[bits(17)]
            trb_transfer_length: u32,
            #[bits(5)]
            _rsvdz: u8,
            #[bits(10)]
            interrupter_target: u16,
        },
        remain: u16 => {
            #[bits(1)]
            cycle_bit: bool,
            #[bits(4)]
            _rsvdz1: u16,
            #[bits(1)]
            interrupt_on_completion: bool,
            #[bits(1)]
            immediate_data: bool,
            #[bits(3)]
            _rsvdz2: u16,
            #[bits(6)]
            trb_type: TrbType,
        },
        control: u16 => {
            #[bits(2)]
            transfer_type: u8,
            #[bits(14)]
            _rsvdz: u16,
        }
    }
}

impl SetupStage {
    pub const TYPE: TrbType = TrbType::SetupStage;
}

impl Type for SetupStage {
    fn get_type(self) -> TrbType {
        Self::TYPE
    }
}

impl TryFrom<TrbRaw> for SetupStage {
    type Error = ();

    fn try_from(value: TrbRaw) -> Result<Self, Self::Error> {
        if matches!(value.get_remain_trb_type(), Self::TYPE) {
            Ok(Self {
                parameter0: value.parameter0,
                parameter1: value.parameter1,
                status: value.status,
                remain: value.remain,
                control: value.control,
            })
        } else {
            Err(())
        }
    }
}

bitfield_struct! {
    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroed)]
    #[endian = "little"]
    pub struct DataStage {
        buf_ptr_lo: u32,
        buf_ptr_hi: u32,
        status: u32 => {
            #[bits(17)]
            trb_transfer_length: u32,
            #[bits(5)]
            td_size: u8,
            #[bits(10)]
            interrupter_target: u16,
        },
        remain: u16 => {
            #[bits(1)]
            cycle_bit: bool,
            #[bits(1)]
            evaluate_next_trb: bool,
            #[bits(1)]
            interrupt_on_short_packet: bool,
            #[bits(1)]
            no_snoop: bool,
            #[bits(1)]
            chain_bit: bool,
            #[bits(1)]
            interrupt_on_completion: bool,
            #[bits(1)]
            immediate_data: bool,
            #[bits(3)]
            _rsvdz: u8,
            #[bits(6)]
            trb_type: TrbType,
        },
        control: u16 => {
            #[bits(1)]
            dir: bool,
            #[bits(15)]
            _rsvdz: u16,
        }
    }
}

impl DataStage {
    pub const TYPE: TrbType = TrbType::DataStage;
}

impl Type for DataStage {
    fn get_type(self) -> TrbType {
        Self::TYPE
    }
}

impl TryFrom<TrbRaw> for DataStage {
    type Error = ();

    fn try_from(value: TrbRaw) -> Result<Self, Self::Error> {
        if matches!(value.get_remain_trb_type(), Self::TYPE) {
            Ok(Self {
                buf_ptr_lo: value.parameter0,
                buf_ptr_hi: value.parameter1,
                status: value.status,
                remain: value.remain,
                control: value.control,
            })
        } else {
            Err(())
        }
    }
}

bitfield_struct! {
    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroed)]
    #[endian = "little"]
    pub struct StatusStage {
        _rsvdz1: u32,
        _rsvdz2: u32,
        status: u32 => {
            #[bits(22)]
            _rsvdz: u32,
            #[bits(10)]
            interrupt_target: u16,
        },
        remain: u16 => {
            #[bits(1)]
            cycle_bit: bool,
            #[bits(1)]
            evaluate_next_trb: bool,
            #[bits(2)]
            _rsvdz1: bool,
            #[bits(1)]
            chain_bit: bool,
            #[bits(1)]
            interrupt_on_completion: bool,
            #[bits(4)]
            _rsvdz2: u8,
            #[bits(6)]
            trb_type: TrbType,
        },
        control: u16 => {
            #[bits(1)]
            direction: bool,
            #[bits(15)]
            _rsvdz: u16,
        }
    }
}

impl StatusStage {
    pub const TYPE: TrbType = TrbType::StatusStage;
}

impl Type for StatusStage {
    fn get_type(self) -> TrbType {
        Self::TYPE
    }
}

impl TryFrom<TrbRaw> for StatusStage {
    type Error = ();

    fn try_from(value: TrbRaw) -> Result<Self, Self::Error> {
        if matches!(value.get_remain_trb_type(), Self::TYPE) {
            Ok(Self {
                _rsvdz1: value.parameter0,
                _rsvdz2: value.parameter1,
                status: value.status,
                remain: value.remain,
                control: value.control,
            })
        } else {
            Err(())
        }
    }
}

bitfield_struct! {
    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroed)]
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

        Self::zeroed()
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

impl TryFrom<TrbRaw> for Link {
    type Error = ();

    fn try_from(value: TrbRaw) -> Result<Self, Self::Error> {
        if matches!(value.get_remain_trb_type(), Self::TYPE) {
            Ok(Self {
                ring_segment_pointer_lo: value.parameter0,
                ring_segment_pointer_hi: value.parameter1,
                status: value.status,
                remain: value.remain,
                _rsvdz: value.control,
            })
        } else {
            Err(())
        }
    }
}

bitfield_struct! {
    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroed)]
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

impl Default for EnableSlotCommand {
    fn default() -> Self {
        Self {
            _rsvdz1: Default::default(),
            _rsvdz2: Default::default(),
            _rsvdz3: Default::default(),
            remain: Default::default(),
            control: Default::default(),
        }
        .with_remain_trb_type(Self::TYPE)
    }
}

impl TryFrom<TrbRaw> for EnableSlotCommand {
    type Error = ();

    fn try_from(value: TrbRaw) -> Result<Self, Self::Error> {
        if matches!(value.get_remain_trb_type(), TrbType::EnableSlotCommand) {
            Ok(Self {
                _rsvdz1: value.parameter0,
                _rsvdz2: value.parameter1,
                _rsvdz3: value.status,
                remain: value.remain,
                control: value.control,
            })
        } else {
            Err(())
        }
    }
}

bitfield_struct! {
    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroed)]
    #[endian = "little"]
    pub struct AddressDeviceCommand {
        params: u64 => {
            #[bits(4)]
            _rsvdz: u8,
            #[bits(60)]
            input_context_ptr: u64,
        },
        _rsvdz: u32,
        remain: u16 => {
            #[bits(1)]
            cycle_bit: bool,
            #[bits(8)]
            _rsvdz: u16,
            #[bits(1)]
            block_set_address_request: bool,
            #[bits(6)]
            trb_type: TrbType,
        },
        control: u16 => {
            #[bits(8)]
            _rsvdz: u8,
            #[bits(8)]
            slot_id: u8,
        }
    }
}

impl AddressDeviceCommand {
    pub const TYPE: TrbType = TrbType::AddressDeviceCommand;

    pub fn new(input_cx_ptr: *mut u8, slot_id: u8) -> Self {
        Self::zeroed()
            .with_remain_trb_type(Self::TYPE)
            .with_control_slot_id(slot_id)
            .with_params_input_context_ptr((input_cx_ptr as u64) >> 4)
    }
}

impl Type for AddressDeviceCommand {
    fn get_type(self) -> TrbType {
        Self::TYPE
    }
}

impl TryFrom<TrbRaw> for AddressDeviceCommand {
    type Error = ();

    fn try_from(value: TrbRaw) -> Result<Self, Self::Error> {
        if matches!(value.get_remain_trb_type(), Self::TYPE) {
            Ok(Self {
                params: ((value.parameter1 as u64) << 32) | value.parameter0 as u64,
                _rsvdz: value.status,
                remain: value.remain,
                control: value.control,
            })
        } else {
            Err(())
        }
    }
}

bitfield_struct! {
    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroed)]
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
            completion_code: CommandConpletionCode,
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
        let ptr = (self.get_params_ptr() << 4) as *const TrbRaw;
        Trb::from(unsafe { *ptr })
    }

    pub fn is_success(self) -> bool {
        self.get_status_completion_code().is_success()
    }
}

impl Type for CommandCompletionEvent {
    fn get_type(self) -> TrbType {
        Self::TYPE
    }
}

impl TryFrom<TrbRaw> for CommandCompletionEvent {
    type Error = ();

    fn try_from(value: TrbRaw) -> Result<Self, Self::Error> {
        if matches!(value.get_remain_trb_type(), Self::TYPE) {
            Ok(Self {
                params: ((value.parameter1 as u64) << 32) | value.parameter0 as u64,
                status: value.status,
                remain: value.remain,
                control: value.control,
            })
        } else {
            Err(())
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommandConpletionCode(u8);

impl CommandConpletionCode {
    pub const INVALID: u8 = 0;
    pub const SUCCESS: u8 = 1;

    pub fn is_success(self) -> bool {
        self.0 == Self::SUCCESS
    }
}

impl EndianFrom<u32> for CommandConpletionCode {
    fn from_le(v: u32) -> Self {
        Self::from_ne(u32::from_le(v))
    }

    fn from_be(v: u32) -> Self {
        Self::from_ne(u32::from_be(v))
    }

    fn from_ne(v: u32) -> Self {
        Self(v as u8)
    }
}

impl EndianInto<u32> for CommandConpletionCode {
    fn to_le(self) -> u32 {
        self.to_ne().to_le()
    }

    fn to_be(self) -> u32 {
        self.to_ne().to_be()
    }

    fn to_ne(self) -> u32 {
        self.0 as u32
    }
}

bitfield_struct! {
    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroed)]
    #[endian = "little"]
    pub struct PortStatusChangeEvent {
        parameter0: u32 => {
            #[bits(24)]
            _rsvdz: u32,
            #[bits(8)]
            port_id: u8,
        },
        parameter1: u32,
        status: u32 => {
            #[bits(24)]
            _rsvdz: u32,
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
        control: u16,
    }
}

impl PortStatusChangeEvent {
    pub const TYPE: TrbType = TrbType::PortStatusChangeEvent;
}

impl Type for PortStatusChangeEvent {
    fn get_type(self) -> TrbType {
        Self::TYPE
    }
}

impl TryFrom<TrbRaw> for PortStatusChangeEvent {
    type Error = ();

    fn try_from(value: TrbRaw) -> Result<Self, Self::Error> {
        if matches!(value.get_remain_trb_type(), Self::TYPE) {
            Ok(Self {
                parameter0: value.parameter0,
                parameter1: value.parameter1,
                status: value.status,
                remain: value.remain,
                control: value.control,
            })
        } else {
            Err(())
        }
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
    EnableSlotCommand(EnableSlotCommand),
    AddressDeviceCommand(AddressDeviceCommand),
    ConfigureEndpoint,
    NoOpCommand,
    TransferEvent,
    CommandCompletionEvent(CommandCompletionEvent),
    PortStatusChangeEvent(PortStatusChangeEvent),
    Unknown(u8),
}

impl From<TrbRaw> for Trb {
    fn from(value: TrbRaw) -> Self {
        unsafe {
            match value.get_remain_trb_type() {
                TrbType::Normal => todo!(),
                TrbType::SetupStage => todo!(),
                TrbType::DataStage => todo!(),
                TrbType::StatusStage => todo!(),
                TrbType::Link => Self::Link(Link::try_from(value).unwrap_unchecked()),
                TrbType::NoOp => todo!(),
                TrbType::EnableSlotCommand => {
                    Self::EnableSlotCommand(EnableSlotCommand::try_from(value).unwrap_unchecked())
                }
                TrbType::AddressDeviceCommand => Self::AddressDeviceCommand(
                    AddressDeviceCommand::try_from(value).unwrap_unchecked(),
                ),
                TrbType::ConfigureEndpoint => todo!(),
                TrbType::NoOpCommand => todo!(),
                TrbType::TransferEvent => todo!(),
                TrbType::CommandConpletionEvent => Self::CommandCompletionEvent(
                    CommandCompletionEvent::try_from(value).unwrap_unchecked(),
                ),
                TrbType::PortStatusChangeEvent => Self::PortStatusChangeEvent(
                    PortStatusChangeEvent::try_from(value).unwrap_unchecked(),
                ),
                TrbType::Unknown(x) => Trb::Unknown(x),
            }
        }
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
            Trb::EnableSlotCommand(_) => EnableSlotCommand::TYPE,
            Trb::AddressDeviceCommand(_) => AddressDeviceCommand::TYPE,
            Trb::ConfigureEndpoint => todo!(),
            Trb::NoOpCommand => todo!(),
            Trb::TransferEvent => todo!(),
            Trb::CommandCompletionEvent(_) => CommandCompletionEvent::TYPE,
            Trb::PortStatusChangeEvent(_) => PortStatusChangeEvent::TYPE,
            Trb::Unknown(x) => TrbType::Unknown(x),
        }
    }
}

pub trait Type {
    fn get_type(self) -> TrbType;
}
