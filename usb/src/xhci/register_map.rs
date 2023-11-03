use super::{
    device::SlotId,
    doorbell::DoorbellWrapper,
    endian::{Endian, EndianInto},
};
use common::Zeroed;
use core::{
    array::IntoIter,
    iter::{Map, Take},
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Index, IndexMut},
    slice,
};
use macros::{bitfield_struct, FromSegment, IntoSegment};

mod sealed {
    pub trait AccessMode {
        type PtrType<T>: RawPtrBase<T>;
    }

    pub trait RawPtrBase<T>: Copy {
        unsafe fn add(self, count: usize) -> Self;
        unsafe fn read_volatile(self) -> T;
    }

    impl<T> RawPtrBase<T> for *const T {
        unsafe fn add(self, count: usize) -> Self {
            self.add(count)
        }

        unsafe fn read_volatile(self) -> T {
            self.read_volatile()
        }
    }

    impl<T> RawPtrBase<T> for *mut T {
        unsafe fn add(self, count: usize) -> Self {
            self.add(count)
        }

        unsafe fn read_volatile(self) -> T {
            self.read_volatile()
        }
    }
}

use sealed::{AccessMode, RawPtrBase};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadOnly;
impl AccessMode for ReadOnly {
    type PtrType<T> = *const T;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadWrite;
impl AccessMode for ReadWrite {
    type PtrType<T> = *mut T;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterMap<'a, const N: usize, U, Mode>
where
    U: Segment<N>,
    Mode: AccessMode,
{
    ptr: Mode::PtrType<<U as Segment<N>>::Element>,
    _phantomdata: PhantomData<&'a ()>,
}

impl<'a, const N: usize, U> RegisterMap<'a, N, U, ReadWrite>
where
    U: Segment<N>,
    <U as Segment<N>>::Element: Endian + Copy,
{
    pub fn write(&mut self, val: U) {
        for (idx, v) in val.into_segment().into_iter().enumerate() {
            unsafe { self.ptr.add(idx).write_volatile(v.to_le()) };
        }
    }

    pub fn new_writeable(ptr: &'a mut [<U as Segment<N>>::Element; N]) -> Self {
        Self {
            ptr: ptr.as_mut_ptr(),
            _phantomdata: PhantomData,
        }
    }

    /// # Safety
    /// ptr must be valid.
    /// sizeof::<U> == sizeof::<T> * N;
    pub unsafe fn from_raw_mut(ptr: *mut U) -> Self {
        Self {
            ptr: ptr.cast::<<U as Segment<N>>::Element>(),
            _phantomdata: PhantomData,
        }
    }
}

impl<'a, const N: usize, U> RegisterMap<'a, N, U, ReadOnly>
where
    U: Segment<N>,
    <U as Segment<N>>::Element: Endian + Copy,
{
    /// # Safety
    /// ptr must be valid.
    /// sizeof::<U> == sizeof::<T> * N;
    pub unsafe fn from_raw(ptr: *const U) -> Self {
        Self {
            ptr: ptr.cast::<<U as Segment<N>>::Element>(),
            _phantomdata: PhantomData,
        }
    }
}

impl<'a, const N: usize, U, Mode> RegisterMap<'a, N, U, Mode>
where
    U: Segment<N>,
    <U as Segment<N>>::Element: Endian + Copy,
    Mode: AccessMode,
{
    pub fn read(&self) -> U {
        let mut arr: [MaybeUninit<<U as Segment<N>>::Element>; N] =
            unsafe { MaybeUninit::uninit().assume_init() };
        for (i, arr) in arr.iter_mut().enumerate() {
            arr.write(unsafe { self.ptr.add(i).read_volatile() });
        }

        // could not compile
        // https://github.com/rust-lang/rust/issues/61956
        // let arr: [T; N] = unsafe { core::mem::transmute::<[MaybeUninit<T>; N], [T; N]>(arr) };

        let arr = unsafe {
            let slice = slice::from_raw_parts(arr.as_ptr().cast::<<U as Segment<N>>::Element>(), N);
            <[<U as Segment<N>>::Element; N]>::try_from(slice).unwrap_unchecked()
        };

        U::from_segment(arr)
    }
}

// associate cosnt is preferred
// https://github.com/rust-lang/rust/issues/60551
pub trait Segment<const N: usize>:
    IntoSegment<N, Element = <Self as Segment<N>>::Element>
    + FromSegment<N, Element = <Self as Segment<N>>::Element>
{
    type Element;
}

impl<const N: usize, T, U> Segment<N> for T
where
    T: IntoSegment<N, Element = U> + FromSegment<N, Element = U>,
{
    type Element = U;
}

pub trait IntoSegment<const N: usize> {
    type Element;

    fn into_segment(self) -> [Self::Element; N];
}

pub trait FromSegment<const N: usize> {
    type Element;
    fn from_segment(v: [Self::Element; N]) -> Self;
}

bitfield_struct! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    struct RsvdZU8 {
        data: u8,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    struct RsvdZU16 {
        data: u16,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    struct RsvdZU32 {
        data: u32,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    struct RsvdZU64 {
        data: u64,
    }
}

bitfield_struct! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct CapLength {
        data: u8,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct HciVersion {
        data: u16,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct HcsParams1 {
        data: u32 => {
            #[bits(8)]
            max_device_slots: u8,
            #[bits(11)]
            max_interrupters: u16,
            #[bits(5)]
            _rsvdz: u8,
            #[bits(8)]
            max_ports: u8,
        }
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct HcsParams2 {
        data: u32 => {
            #[bits(4)]
            isochronous_scheduling_threshold: u8,
            #[bits(4)]
            event_ring_segment_table_max: u8,
            #[bits(13)]
            _rsvdz: u16,
            #[bits(5)]
            max_scratchpad_buffers_high: u8,
            #[bits(1)]
            scratchpad_restore: bool,
            #[bits(5)]
            max_scratchpad_buffers_low: u8,
        }
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct HcsParams3 {
        data: u32 => {
            #[bits(8)]
            u1_device_exit_latency: u8,
            #[bits(8)]
            u2_device_exit_latency: u8,
            #[bits(16)]
            _rsvd: u16,
        }
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct DbOffset {
        data: u32 => {
            #[bits(2)]
            _rsvdz: u8,
            #[bits(30)]
            offset: u32,
        }
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct RtsOffset {
        data: u32 => {
            #[bits(5)]
            _rsvdz: u8,
            #[bits(27)]
            offset: u32,
        }
    }

}

#[derive(Debug)]
pub struct CapabilityRegisters<'a> {
    cap_length: RegisterMap<'a, 1, CapLength, ReadOnly>,
    hci_version: RegisterMap<'a, 1, HciVersion, ReadOnly>,
    hcs_paracm1: RegisterMap<'a, 1, HcsParams1, ReadOnly>,
    hcs_paracm2: RegisterMap<'a, 1, HcsParams2, ReadOnly>,
    hcs_params3: RegisterMap<'a, 1, HcsParams3, ReadOnly>,
    db_offset: RegisterMap<'a, 1, DbOffset, ReadOnly>,
    rts_offset: RegisterMap<'a, 1, RtsOffset, ReadOnly>,
}

impl<'a> CapabilityRegisters<'a> {
    pub const CAP_LENGTH_OFFSET: usize = 0x00;
    pub const HCI_VERSION_OFFSET: usize = 0x02;
    pub const HCS_PARAMS1_OFFSET: usize = 0x04;
    pub const HCS_PARAMS2_OFFSET: usize = 0x08;
    pub const HCS_PARAMS3_OFFSET: usize = 0x0C;

    pub const DB_OFFSET_OFFSET: usize = 0x14;
    pub const RTS_OFFSET_OFFSET: usize = 0x18;

    /// # Safety
    /// base is the beginnint of the host controller's MMIO address space.
    pub unsafe fn new(base: *const u8) -> Self {
        Self {
            cap_length: RegisterMap::from_raw(base.add(Self::CAP_LENGTH_OFFSET).cast()),
            hci_version: RegisterMap::from_raw(base.add(Self::HCI_VERSION_OFFSET).cast()),
            hcs_paracm1: RegisterMap::from_raw(base.add(Self::HCS_PARAMS1_OFFSET).cast()),
            hcs_paracm2: RegisterMap::from_raw(base.add(Self::HCS_PARAMS2_OFFSET).cast()),
            hcs_params3: RegisterMap::from_raw(base.add(Self::HCS_PARAMS3_OFFSET).cast()),

            db_offset: RegisterMap::from_raw(base.add(Self::DB_OFFSET_OFFSET).cast()),
            rts_offset: RegisterMap::from_raw(base.add(Self::RTS_OFFSET_OFFSET).cast()),
        }
    }

    pub fn cap_length(&self) -> &RegisterMap<'a, 1, CapLength, ReadOnly> {
        &self.cap_length
    }

    pub fn hci_version(&self) -> &RegisterMap<'a, 1, HciVersion, ReadOnly> {
        &self.hci_version
    }

    pub fn hcs_paracm1(&self) -> &RegisterMap<'a, 1, HcsParams1, ReadOnly> {
        &self.hcs_paracm1
    }

    pub fn hcs_paracm2(&self) -> &RegisterMap<'a, 1, HcsParams2, ReadOnly> {
        &self.hcs_paracm2
    }

    pub fn hcs_params3(&self) -> &RegisterMap<'a, 1, HcsParams3, ReadOnly> {
        &self.hcs_params3
    }

    pub fn rts_offset(&self) -> &RegisterMap<'a, 1, RtsOffset, ReadOnly> {
        &self.rts_offset
    }

    pub fn db_offset(&self) -> &RegisterMap<'a, 1, DbOffset, ReadOnly> {
        &self.db_offset
    }
}

bitfield_struct! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct PortSC {
        data: u32 => {
            #[bits(1)]
            current_connect_status: bool,
            #[bits(1)]
            port_enabled_disabled: bool,
            #[bits(1)]
            _rsvdz1: bool,
            #[bits(1)]
            over_current_active: bool,
            #[bits(1)]
            port_reset: bool,
            #[bits(4)]
            port_link_state: u8,
            #[bits(1)]
            port_power: bool,
            #[bits(4)]
            port_speed: u8,
            #[bits(2)]
            port_indicator_control: u8,
            #[bits(1)]
            port_link_state_write_strobe: bool,
            #[bits(1)]
            connect_status_change: bool,
            #[bits(1)]
            port_enabled_disabled_change: bool,
            #[bits(1)]
            warm_port_reset_change: bool,
            #[bits(1)]
            over_current_change: bool,
            #[bits(1)]
            port_reset_change: bool,
            #[bits(1)]
            port_link_state_change: bool,
            #[bits(1)]
            port_config_error_change: bool,
            #[bits(1)]
            cold_attach_status: bool,
            #[bits(1)]
            wake_on_connect_enable: bool,
            #[bits(1)]
            wake_on_disconnect_enable: bool,
            #[bits(1)]
            wake_on_over_current_enable: bool,
            #[bits(2)]
            _rsvdz2: u8,
            #[bits(1)]
            device_removable: bool,
            #[bits(1)]
            warm_port_reset: bool,
        }
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct PortPowerMSC3 {
        data: u32 => {
            #[bits(8)]
            u1_timeout: u8,
            #[bits(8)]
            u2_timeout: u8,
            #[bits(1)]
            force_link_accept: bool,
            #[bits(15)]
            _rsvdp: u16,
        }
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct PortLinkInfo3 {
        data: u32 => {
            #[bits(16)]
            link_error_count: u16,
            #[bits(4)]
            rx_lane_count: u8,
            #[bits(4)]
            tx_lane_count: u8,
            #[bits(8)]
            _rsvdp: u8,
        }
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct PortHardwareLPMControl3 {
        data: u32 => {
            #[bits(16)]
            link_soft_error_count: u16,
            #[bits(16)]
            _rsvdp: u16,
        }
    }
}

impl PortSC {
    pub fn clear_rw1s(self) -> Self {
        self.with_data_port_enabled_disabled(false)
            .with_data_port_reset(false)
            .with_data_connect_status_change(false)
            .with_data_port_enabled_disabled_change(false)
            .with_data_warm_port_reset_change(false)
            .with_data_over_current_change(false)
            .with_data_port_reset_change(false)
            .with_data_port_link_state_change(false)
            .with_data_port_config_error_change(false)
            .with_data_warm_port_reset(false)
    }
}

#[derive(Debug)]
pub struct PortRegisterSet<'a> {
    port_status_and_control: RegisterMap<'a, 1, PortSC, ReadWrite>,
    port_power_management_status_and_control: RegisterMap<'a, 1, PortPowerMSC3, ReadWrite>,
    port_link_info: RegisterMap<'a, 1, PortLinkInfo3, ReadWrite>,
    port_hardware_lpm_control: RegisterMap<'a, 1, PortHardwareLPMControl3, ReadWrite>,
}

impl<'a> PortRegisterSet<'a> {
    pub const PORT_STATUS_AND_CONTROL_OFFSET: usize = 0x00;
    pub const PORT_POWER_MANAGEMENT_STATUS_AND_CONTROL_OFFSET: usize = 0x04;
    pub const PORT_LINK_INFO: usize = 0x08;
    pub const PORT_HARDWARE_LPM_CONTROL: usize = 0x0c;

    /// # Safety
    /// `base` = Operational Base + (0x400 + (0x10 * (n–1)))
    /// where `n` = 1, 2, 3, ... , MaxPorts
    pub unsafe fn new(base: *mut u8) -> Self {
        Self {
            port_status_and_control: RegisterMap::from_raw_mut(
                base.add(Self::PORT_STATUS_AND_CONTROL_OFFSET).cast(),
            ),
            port_power_management_status_and_control: RegisterMap::from_raw_mut(
                base.add(Self::PORT_POWER_MANAGEMENT_STATUS_AND_CONTROL_OFFSET)
                    .cast(),
            ),
            port_link_info: RegisterMap::from_raw_mut(base.add(Self::PORT_LINK_INFO).cast()),
            port_hardware_lpm_control: RegisterMap::from_raw_mut(
                base.add(Self::PORT_HARDWARE_LPM_CONTROL).cast(),
            ),
        }
    }

    pub fn port_status_and_control(&self) -> &RegisterMap<'a, 1, PortSC, ReadWrite> {
        &self.port_status_and_control
    }

    pub fn port_power_management_status_and_control(
        &self,
    ) -> &RegisterMap<'a, 1, PortPowerMSC3, ReadWrite> {
        &self.port_power_management_status_and_control
    }

    pub fn port_link_info(&self) -> &RegisterMap<'a, 1, PortLinkInfo3, ReadWrite> {
        &self.port_link_info
    }

    pub fn port_hardware_lpm_control(
        &self,
    ) -> &RegisterMap<'a, 1, PortHardwareLPMControl3, ReadWrite> {
        &self.port_hardware_lpm_control
    }

    pub fn port_status_and_control_mut(&mut self) -> &mut RegisterMap<'a, 1, PortSC, ReadWrite> {
        &mut self.port_status_and_control
    }

    pub fn port_power_management_status_and_control_mut(
        &mut self,
    ) -> &mut RegisterMap<'a, 1, PortPowerMSC3, ReadWrite> {
        &mut self.port_power_management_status_and_control
    }

    pub fn port_link_info_mut(&mut self) -> &mut RegisterMap<'a, 1, PortLinkInfo3, ReadWrite> {
        &mut self.port_link_info
    }

    pub fn port_hardware_lpm_control_mut(
        &mut self,
    ) -> &mut RegisterMap<'a, 1, PortHardwareLPMControl3, ReadWrite> {
        &mut self.port_hardware_lpm_control
    }
}

bitfield_struct! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct UsbCommand {
        data: u32 => {
            #[bits(1)]
            run_stop: bool,
            #[bits(1)]
            host_controller_reset: bool,
            #[bits(1)]
            interrupter_enable: bool,
            #[bits(1)]
            host_system_error_enable: bool,
            #[bits(3)]
            _rsvdz1: u8,
            #[bits(1)]
            light_host_controller_reset: bool,
            #[bits(1)]
            controller_save_state: bool,
            #[bits(1)]
            controller_restore_state: bool,
            #[bits(1)]
            enable_wrap_event: bool,
            #[bits(1)]
            enable_u3_mfindex_stop: bool,
            #[bits(1)]
            _rsvdz2: bool,
            #[bits(1)]
            cem_enable: bool,
            #[bits(1)]
            extended_tbc_enable: bool,
            #[bits(1)]
            extended_tbc_trb_enable: bool,
            #[bits(1)]
            vtio_enable: bool,
            #[bits(15)]
            _rsvdz3: u32,
        }
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct UsbStatus {
        data: u32 => {
            #[bits(1)]
            host_controller_halted: bool,
            #[bits(1)]
            _rsvdz1: bool,
            #[bits(1)]
            host_system_error: bool,
            #[bits(1)]
            event_interrupt: bool,
            #[bits(1)]
            port_change_detect: bool,
            #[bits(3)]
            _rsvdz2: u8,
            #[bits(1)]
            save_state_status: bool,
            #[bits(1)]
            restore_state_status: bool,
            #[bits(1)]
            save_restore_error: bool,
            #[bits(1)]
            controller_not_ready: bool,
            #[bits(1)]
            host_controller_error: bool,
            #[bits(19)]
            _rsvdz3: u32,
        }
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct CommandRingControl {
        command_ring_ptr_lo: u32 => {
            #[bits(1)]
            ring_cycle_state: bool,
            #[bits(1)]
            command_stop: bool,
            #[bits(1)]
            command_abort: bool,
            #[bits(1)]
            command_ring_running: bool,
            #[bits(2)]
            _rsvdp: u8,
            #[bits(26)]
            data: u32,
        },
        command_ring_ptr_hi: u32,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct Dcbaap {
        ptr_lo: u32 => {
            #[bits(6)]
            _rsvdz: u8,
            #[bits(26)]
            ptr_lo: u32,
        },
        ptr_hi: u32,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct Configure {
        data: u32 => {
            #[bits(8)]
            max_device_slots_enabled: u8,
            #[bits(1)]
            u3_entry_enable: bool,
            #[bits(1)]
            configuration_information_enable: bool,
            #[bits(22)]
            _rsvdp: u32,
        }
    }
}

pub const MAX_PORT_REGISTER_SET_NUM: usize = 256;
#[derive(Debug)]
pub struct PortRegisters<'a> {
    regs: [MaybeUninit<PortRegisterSet<'a>>; MAX_PORT_REGISTER_SET_NUM],
    max_ports: u8,
}

impl<'a> PortRegisters<'a> {
    /// # Safety
    /// base is valid ptr. op base + port off
    pub unsafe fn new(base: *mut u8, max_ports: u8) -> Self {
        let mut arr: [MaybeUninit<PortRegisterSet>; MAX_PORT_REGISTER_SET_NUM] =
            unsafe { MaybeUninit::zeroed().assume_init() };
        for (idx, elem) in arr.iter_mut().enumerate().take(max_ports as usize) {
            elem.write(unsafe { PortRegisterSet::new(base.add(0x10 * idx)) });
        }

        Self {
            regs: arr,
            max_ports,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &PortRegisterSet<'a>> {
        self.regs
            .iter()
            .take(self.max_ports as usize)
            .map(|v| unsafe { v.assume_init_ref() })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PortRegisterSet<'a>> {
        self.regs
            .iter_mut()
            .take(self.max_ports as usize)
            .map(|v| unsafe { v.assume_init_mut() })
    }
}

impl<'a> Index<usize> for PortRegisters<'a> {
    type Output = PortRegisterSet<'a>;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.max_ports as usize {
            unsafe { self.regs.index(index).assume_init_ref() }
        } else {
            panic!("index out of range: {}", index)
        }
    }
}

impl<'a> IndexMut<usize> for PortRegisters<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index < self.max_ports as usize {
            unsafe { self.regs.index_mut(index).assume_init_mut() }
        } else {
            panic!("index out of range: {}", index)
        }
    }
}

impl<'a> IntoIterator for PortRegisters<'a> {
    type Item = PortRegisterSet<'a>;

    type IntoIter = Map<
        Take<IntoIter<MaybeUninit<Self::Item>, MAX_PORT_REGISTER_SET_NUM>>,
        fn(MaybeUninit<PortRegisterSet<'a>>) -> PortRegisterSet<'a>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.regs
            .into_iter()
            .take(self.max_ports as usize)
            .map(|v| unsafe { v.assume_init() })
    }
}

#[derive(Debug)]
pub struct OperationalRegisters<'a> {
    usb_command: RegisterMap<'a, 1, UsbCommand, ReadWrite>,
    usb_status: RegisterMap<'a, 1, UsbStatus, ReadWrite>,
    _page_size: RegisterMap<'a, 1, RsvdZU32, ReadWrite>,
    _device_notification_control: RegisterMap<'a, 1, RsvdZU32, ReadWrite>,
    command_ring_control: RegisterMap<'a, 2, CommandRingControl, ReadWrite>,
    device_context_base_address_array_pointer: RegisterMap<'a, 2, Dcbaap, ReadWrite>,
    configure: RegisterMap<'a, 1, Configure, ReadWrite>,
    port_register_set: PortRegisters<'a>,
}

impl<'a> OperationalRegisters<'a> {
    pub const USB_COMMAND_OFFSET: usize = 0x00;
    pub const USB_STATUS_OFFSET: usize = 0x04;
    pub const PAGE_SIZE_OFFSET: usize = 0x08;
    pub const DEVICE_NOTIFICATION_CONTROL_OFFSET: usize = 0x14;
    pub const COMMAND_RING_CONTROL_OFFSET: usize = 0x18;
    pub const DEVICE_CONTEXT_BASE_ADDRESS_ARRAY_POINTER_OFFSET: usize = 0x30;
    pub const CONFIGUR_OFFSET: usize = 0x38;
    pub const PORT_REGISTER_SET_OFFSET: usize = 0x400;

    /// # Safety
    /// base is the beginning of the Operational Register space.
    pub unsafe fn new(base: *mut u8, max_ports: u8) -> Self {
        let port_register_set =
            PortRegisters::new(base.add(Self::PORT_REGISTER_SET_OFFSET), max_ports);

        Self {
            usb_command: RegisterMap::from_raw_mut(base.add(Self::USB_COMMAND_OFFSET).cast()),
            usb_status: RegisterMap::from_raw_mut(base.add(Self::USB_STATUS_OFFSET).cast()),
            _page_size: RegisterMap::from_raw_mut(base.add(Self::PAGE_SIZE_OFFSET).cast()),
            _device_notification_control: RegisterMap::from_raw_mut(
                base.add(Self::DEVICE_NOTIFICATION_CONTROL_OFFSET).cast(),
            ),
            command_ring_control: RegisterMap::from_raw_mut(
                base.add(Self::COMMAND_RING_CONTROL_OFFSET).cast(),
            ),
            device_context_base_address_array_pointer: RegisterMap::from_raw_mut(
                base.add(Self::DEVICE_CONTEXT_BASE_ADDRESS_ARRAY_POINTER_OFFSET)
                    .cast(),
            ),
            configure: RegisterMap::from_raw_mut(base.add(Self::CONFIGUR_OFFSET).cast()),
            port_register_set,
        }
    }

    pub fn usb_command(&self) -> &RegisterMap<'a, 1, UsbCommand, ReadWrite> {
        &self.usb_command
    }

    pub fn usb_command_mut(&mut self) -> &mut RegisterMap<'a, 1, UsbCommand, ReadWrite> {
        &mut self.usb_command
    }

    pub fn usb_status(&self) -> &RegisterMap<'a, 1, UsbStatus, ReadWrite> {
        &self.usb_status
    }

    pub fn usb_status_mut(&mut self) -> &mut RegisterMap<'a, 1, UsbStatus, ReadWrite> {
        &mut self.usb_status
    }

    pub fn configure(&self) -> &RegisterMap<'a, 1, Configure, ReadWrite> {
        &self.configure
    }

    pub fn configure_mut(&mut self) -> &mut RegisterMap<'a, 1, Configure, ReadWrite> {
        &mut self.configure
    }

    pub fn device_context_base_address_array_pointer(
        &self,
    ) -> &RegisterMap<'a, 2, Dcbaap, ReadWrite> {
        &self.device_context_base_address_array_pointer
    }

    pub fn device_context_base_address_array_pointer_mut(
        &mut self,
    ) -> &mut RegisterMap<'a, 2, Dcbaap, ReadWrite> {
        &mut self.device_context_base_address_array_pointer
    }

    pub fn command_ring_control(&self) -> &RegisterMap<'a, 2, CommandRingControl, ReadWrite> {
        &self.command_ring_control
    }

    pub fn command_ring_control_mut(
        &mut self,
    ) -> &mut RegisterMap<'a, 2, CommandRingControl, ReadWrite> {
        &mut self.command_ring_control
    }

    pub fn port_registers_mut(&mut self) -> &mut PortRegisters<'a> {
        &mut self.port_register_set
    }
}

bitfield_struct! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct Iman {
        data: u32 => {
            #[bits(1)]
            interrupt_pending: bool,
            #[bits(1)]
            interrupt_enable: bool,
            #[bits(30)]
            _rsvdp: u32,
        }
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct Imod {
        data: u32 => {
            #[bits(16)]
            interrupt_modification_interval: u16,
            #[bits(16)]
            interrupt_moderation_counter: u16,
        }
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct Erstsz {
        data: u32 => {
            #[bits(16)]
            event_ring_segment_table_size: u16,
            #[bits(16)]
            _rsvdp: u16,
        }
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct Erstba {
        data: u64 => {
            #[bits(6)]
            _rsvdp: u8,
            #[bits(58)]
            ptr: u64,
        }
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct Erdp {
        data: u64 => {
            #[bits(3)]
            dequeue_erst_segment_index: u8,
            #[bits(1)]
            even_handler_busy: bool,
            #[bits(60)]
            ptr: u64,
        }
    }
}

#[derive(Debug)]
pub struct InterrupterRegisterSet<'a> {
    interrupt_management: RegisterMap<'a, 1, Iman, ReadWrite>,
    interrupt_moderation: RegisterMap<'a, 1, Imod, ReadWrite>,
    event_ring_segment_table_size: RegisterMap<'a, 1, Erstsz, ReadWrite>,
    event_ring_segment_table_base_address: RegisterMap<'a, 1, Erstba, ReadWrite>,
    event_ring_dequeue_pointer: RegisterMap<'a, 1, Erdp, ReadWrite>,
}

impl<'a> InterrupterRegisterSet<'a> {
    pub const INTERRUPT_MANAGEMENT_OFFSET: usize = 0x00;
    pub const INTERRUPT_MODERATION_OFFSET: usize = 0x04;
    pub const EVENT_RING_SEGMENT_TABLE_SIZE_OFFSET: usize = 0x08;
    pub const EVENT_RING_SEGMENT_TABLE_BASE_ADDRESS_OFFSET: usize = 0x10;
    pub const EVENT_RING_DEQUEUE_POINTER_OFFSET: usize = 0x18;

    /// # Safety
    /// base is Runtime Base + 0x20 + (32 * idx)
    pub unsafe fn new(base: *mut u8) -> Self {
        Self {
            interrupt_management: RegisterMap::from_raw_mut(
                base.add(Self::INTERRUPT_MANAGEMENT_OFFSET).cast(),
            ),
            interrupt_moderation: RegisterMap::from_raw_mut(
                base.add(Self::INTERRUPT_MODERATION_OFFSET).cast(),
            ),
            event_ring_segment_table_size: RegisterMap::from_raw_mut(
                base.add(Self::EVENT_RING_SEGMENT_TABLE_SIZE_OFFSET).cast(),
            ),
            event_ring_segment_table_base_address: RegisterMap::from_raw_mut(
                base.add(Self::EVENT_RING_SEGMENT_TABLE_BASE_ADDRESS_OFFSET)
                    .cast(),
            ),
            event_ring_dequeue_pointer: RegisterMap::from_raw_mut(
                base.add(Self::EVENT_RING_DEQUEUE_POINTER_OFFSET).cast(),
            ),
        }
    }

    pub fn event_ring_segment_table_size(&self) -> &RegisterMap<'a, 1, Erstsz, ReadWrite> {
        &self.event_ring_segment_table_size
    }

    pub fn event_ring_segment_table_size_mut(
        &mut self,
    ) -> &mut RegisterMap<'a, 1, Erstsz, ReadWrite> {
        &mut self.event_ring_segment_table_size
    }

    pub fn event_ring_segment_table_base_address(&self) -> &RegisterMap<'a, 1, Erstba, ReadWrite> {
        &self.event_ring_segment_table_base_address
    }

    pub fn event_ring_segment_table_base_address_mut(
        &mut self,
    ) -> &mut RegisterMap<'a, 1, Erstba, ReadWrite> {
        &mut self.event_ring_segment_table_base_address
    }

    pub fn event_ring_dequeue_pointer(&self) -> &RegisterMap<'a, 1, Erdp, ReadWrite> {
        &self.event_ring_dequeue_pointer
    }

    pub fn event_ring_dequeue_pointer_mut(&mut self) -> &mut RegisterMap<'a, 1, Erdp, ReadWrite> {
        &mut self.event_ring_dequeue_pointer
    }

    pub fn interrupt_management(&self) -> &RegisterMap<'a, 1, Iman, ReadWrite> {
        &self.interrupt_management
    }

    pub fn interrupt_moderation(&self) -> &RegisterMap<'a, 1, Imod, ReadWrite> {
        &self.interrupt_moderation
    }

    pub fn interrupt_management_mut(&mut self) -> &mut RegisterMap<'a, 1, Iman, ReadWrite> {
        &mut self.interrupt_management
    }

    pub fn interrupt_moderation_mut(&mut self) -> &mut RegisterMap<'a, 1, Imod, ReadWrite> {
        &mut self.interrupt_moderation
    }
}

pub const INTERRUPTER_REGISTER_SET_NUM: usize = 1;
#[derive(Debug)]
pub struct RuntimeRegisters<'a> {
    _microframe_index: RegisterMap<'a, 1, RsvdZU32, ReadOnly>,
    interrupter_register_sets: [InterrupterRegisterSet<'a>; INTERRUPTER_REGISTER_SET_NUM],
}

impl<'a> RuntimeRegisters<'a> {
    pub const INTERRUPTER_REGISTER_SET_NUM: usize = INTERRUPTER_REGISTER_SET_NUM;
    pub const MICROFRAME_INDEX_OFFSET: usize = 0x00;
    pub const INTERRUPTER_REGISTER_OFFSET: usize = 0x20;

    /// # Safety
    /// base is the beginning of the Runtime Register space.
    pub unsafe fn new(base: *mut u8) -> Self {
        let mut arr: [MaybeUninit<InterrupterRegisterSet>; INTERRUPTER_REGISTER_SET_NUM] =
            unsafe { MaybeUninit::zeroed().assume_init() };

        for (idx, elem) in arr.iter_mut().enumerate() {
            let addr = base.add(Self::INTERRUPTER_REGISTER_OFFSET + (32 * idx));
            elem.write(InterrupterRegisterSet::new(addr));
        }

        let interrupter_register_sets = unsafe { core::mem::transmute(arr) };

        Self {
            _microframe_index: RegisterMap::from_raw(
                base.add(Self::MICROFRAME_INDEX_OFFSET).cast(),
            ),
            interrupter_register_sets,
        }
    }

    pub fn get_interrupter_register_sets(
        &self,
    ) -> &[InterrupterRegisterSet<'a>; INTERRUPTER_REGISTER_SET_NUM] {
        &self.interrupter_register_sets
    }

    pub fn get_interrupter_register_sets_mut(
        &mut self,
    ) -> &mut [InterrupterRegisterSet<'a>; INTERRUPTER_REGISTER_SET_NUM] {
        &mut self.interrupter_register_sets
    }

    pub fn get_primary_interrupter_mut(&mut self) -> &mut InterrupterRegisterSet<'a> {
        self.interrupter_register_sets.index_mut(0)
    }
}

bitfield_struct! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Zeroed, IntoSegment, FromSegment)]
    #[endian = "little"]
    pub struct Doorbell {
        data: u32 => {
            #[bits(8)]
            db_target: u8,
            #[bits(8)]
            _rsvdz: u8,
            #[bits(16)]
            db_task_id: u16,
        }
    }
}

pub const MAX_DOORBELL_NUM: usize = 256;
pub type DoorbellRegister<'a> = RegisterMap<'a, 1, Doorbell, ReadWrite>;
#[derive(Debug)]
pub struct DoorbellRegisters<'a> {
    regs: [MaybeUninit<DoorbellRegister<'a>>; MAX_DOORBELL_NUM],
    len: usize,
}

impl<'a> DoorbellRegisters<'a> {
    /// # Safety
    /// base is base of doorbell registers(cap base + dboff)
    pub unsafe fn new(base: *mut Doorbell, len: usize) -> Self {
        assert!(len <= MAX_DOORBELL_NUM);

        let mut regs: [MaybeUninit<DoorbellRegister<'a>>; MAX_DOORBELL_NUM] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for (i, elem) in regs.iter_mut().enumerate().take(len) {
            elem.write(RegisterMap::from_raw_mut(base.add(i)));
        }

        Self { regs, len }
    }

    pub fn host_controller_mut(&mut self) -> DoorbellWrapper<'_, 'a> {
        self.index_mut(0).into()
    }

    pub fn slot(&mut self, slot_id: SlotId) -> DoorbellWrapper<'_, 'a> {
        self.index_mut(slot_id as usize).into()
    }
}

impl<'a> Index<usize> for DoorbellRegisters<'a> {
    type Output = DoorbellRegister<'a>;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.len {
            unsafe { self.regs[index].assume_init_ref() }
        } else {
            panic!("index out of range: {}", index)
        }
    }
}

impl<'a> IndexMut<usize> for DoorbellRegisters<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index < self.len {
            unsafe { self.regs[index].assume_init_mut() }
        } else {
            panic!("index out of range: {}", index)
        }
    }
}
