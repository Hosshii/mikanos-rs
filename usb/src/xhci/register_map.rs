use super::endian::Endian;
use core::{marker::PhantomData, mem::MaybeUninit, slice};
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
    struct RsvdZU8 {
        data: u8,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    struct RsvdZU16 {
        data: u16,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    struct RsvdZU32 {
        data: u32,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    struct RsvdZU64 {
        data: u64,
    }
}

bitfield_struct! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    pub struct CapLength {
        data: u8,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
    pub struct HciVersion {
        data: u16,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
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

}

#[derive(Debug)]
pub struct CapabilityRegisters<'a> {
    cap_length: RegisterMap<'a, 1, CapLength, ReadOnly>,
    hci_version: RegisterMap<'a, 1, HciVersion, ReadOnly>,
    hcs_paracm1: RegisterMap<'a, 1, HcsParams1, ReadOnly>,
    hcs_paracm2: RegisterMap<'a, 1, HcsParams2, ReadOnly>,
    hcs_params3: RegisterMap<'a, 1, HcsParams3, ReadOnly>,
}

impl<'a> CapabilityRegisters<'a> {
    pub const CAP_LENGTH_OFFSET: usize = 0x00;
    pub const HCI_VERSION_OFFSET: usize = 0x02;
    pub const HCS_PARAMS1_OFFSET: usize = 0x04;
    pub const HCS_PARAMS2_OFFSET: usize = 0x08;
    pub const HCS_PARAMS3_OFFSET: usize = 0x0C;

    /// # Safety
    /// base is the beginnint of the host controller's MMIO address space.
    pub unsafe fn new(base: *const u8) -> Self {
        Self {
            cap_length: RegisterMap::from_raw(base.add(Self::CAP_LENGTH_OFFSET).cast()),
            hci_version: RegisterMap::from_raw(base.add(Self::HCI_VERSION_OFFSET).cast()),
            hcs_paracm1: RegisterMap::from_raw(base.add(Self::HCS_PARAMS1_OFFSET).cast()),
            hcs_paracm2: RegisterMap::from_raw(base.add(Self::HCS_PARAMS2_OFFSET).cast()),
            hcs_params3: RegisterMap::from_raw(base.add(Self::HCS_PARAMS3_OFFSET).cast()),
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
}

bitfield_struct! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, IntoSegment, FromSegment)]
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
}

#[derive(Debug)]
pub struct OperationalRegisters<'a> {
    usb_command: RegisterMap<'a, 1, UsbCommand, ReadWrite>,
    usb_status: RegisterMap<'a, 1, UsbStatus, ReadWrite>,
    _page_size: RegisterMap<'a, 1, RsvdZU32, ReadWrite>,
    _device_notification_control: RegisterMap<'a, 1, RsvdZU32, ReadWrite>,
    _command_ring_control: RegisterMap<'a, 1, RsvdZU16, ReadWrite>,
    _device_context_base_address_array_pointer: RegisterMap<'a, 1, RsvdZU64, ReadWrite>,
    _configure: RegisterMap<'a, 1, RsvdZU32, ReadWrite>,
}

impl<'a> OperationalRegisters<'a> {
    pub const USB_COMMAND_OFFSET: usize = 0x00;
    pub const USB_STATUS_OFFSET: usize = 0x04;
    pub const PAGE_SIZE_OFFSET: usize = 0x08;
    pub const DEVICE_NOTIFICATION_CONTROL_OFFSET: usize = 0x14;
    pub const COMMAND_RING_CONTROL_OFFSET: usize = 0x18;
    pub const DEVICE_CONTEXT_BASE_ADDRESS_ARRAY_POINTER_OFFSET: usize = 0x30;
    pub const CONFIGUR_OFFSET: usize = 0x38;

    /// # Safety
    /// base is the beginnint of the host controller's MMIO address space.
    pub unsafe fn new(base: *mut u8) -> Self {
        Self {
            usb_command: RegisterMap::from_raw_mut(base.add(Self::USB_COMMAND_OFFSET).cast()),
            usb_status: RegisterMap::from_raw_mut(base.add(Self::USB_STATUS_OFFSET).cast()),
            _page_size: RegisterMap::from_raw_mut(base.add(Self::PAGE_SIZE_OFFSET).cast()),
            _device_notification_control: RegisterMap::from_raw_mut(
                base.add(Self::DEVICE_NOTIFICATION_CONTROL_OFFSET).cast(),
            ),
            _command_ring_control: RegisterMap::from_raw_mut(
                base.add(Self::COMMAND_RING_CONTROL_OFFSET).cast(),
            ),
            _device_context_base_address_array_pointer: RegisterMap::from_raw_mut(
                base.add(Self::DEVICE_CONTEXT_BASE_ADDRESS_ARRAY_POINTER_OFFSET)
                    .cast(),
            ),
            _configure: RegisterMap::from_raw_mut(base.add(Self::CONFIGUR_OFFSET).cast()),
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
}
