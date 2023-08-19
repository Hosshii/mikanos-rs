use crate::types::{Guid, Handle, Status, Uint32, Uint64, Uintn, UnusedPtr, Void};

use super::header::TableHeader;

pub type RaiseTPL = UnusedPtr;
pub type RestoreTPL = UnusedPtr;
pub type AllocatePages = extern "efiapi" fn(
    type_: AllocateType,
    memory_type: MemoryType,
    pages: Uintn,
    memory: *mut PhysicalAddress,
) -> Status;
pub type FreePages = UnusedPtr;
pub type GetMemoryMap = extern "efiapi" fn(
    memory_map_size: *mut Uintn,
    memory_map: *mut MemoryDescriptor,
    map_key: *mut Uintn,
    descriptor_size: *mut Uintn,
    descriptor_version: *mut Uint32,
) -> Status;
pub type AllocatePool =
    extern "efiapi" fn(pool_type: MemoryType, size: Uintn, &mut *mut Void) -> Status;
pub type FreePool = extern "efiapi" fn(buffer: *mut Void) -> Status;
pub type CreateEvent = UnusedPtr;
pub type SetTimer = UnusedPtr;
pub type WaitForEvent = UnusedPtr;
pub type SignalEvent = UnusedPtr;
pub type CloseEvent = UnusedPtr;
pub type CheckEvent = UnusedPtr;
pub type InstallProtocolIenterface = UnusedPtr;
pub type ReinstallProtocolIenterface = UnusedPtr;
pub type UninstallProtocolIenterface = UnusedPtr;
pub type HnadleProtocol = UnusedPtr;
pub type RegisterProtocolNotify = UnusedPtr;
pub type LocateHandle = UnusedPtr;
pub type LocateDevichPath = UnusedPtr;
pub type InstallConfigurationTable = UnusedPtr;
pub type ImageUnload = UnusedPtr;
pub type ImageStart = UnusedPtr;
pub type Exit = UnusedPtr;
pub type ExitBootServices = extern "efiapi" fn(image_handle: Handle, map_key: Uintn) -> Status;
pub type GetNextMonotonicCount = UnusedPtr;
pub type Stall = UnusedPtr;
pub type SetWatchdogTimer = UnusedPtr;
pub type ConnectController = UnusedPtr;
pub type DisconnectController = UnusedPtr;
pub type OpenProtocol = extern "efiapi" fn(
    handle: Handle,
    protocol: *const Guid,
    interface: *mut *mut Void,
    agnen_handle: Handle,
    controller_handle: Handle,
    attributes: Uint32,
) -> Status;
pub type CloseProtocol = UnusedPtr;
pub type OpenProtocolInformation = UnusedPtr;
pub type ProtocolsPerHandle = UnusedPtr;
pub type LocateHandleBuffer = UnusedPtr;
pub type LocateProtocol = UnusedPtr;
pub type InstallMultipleProtocolInterfaces = UnusedPtr;
pub type UninstallMultipleProtocolInterfaces = UnusedPtr;
pub type CalculateCrc32 = UnusedPtr;
pub type CopyMem = UnusedPtr;
pub type SetMem = UnusedPtr;
pub type CreateEventEx = UnusedPtr;

pub type PhysicalAddress = Uint64;
pub type VirtualAddress = Uint64;

pub const OPEN_PROTOCOL_BY_HANDLE_PROTOCOL: Uint32 = 0x00000001;
pub const OPEN_PROTOCOL_GET_PROTOCOL: Uint32 = 0x00000002;
pub const OPEN_PROTOCOL_TEST_PROTOCOL: Uint32 = 0x00000004;
pub const OPEN_PROTOCOL_BY_CHILD_CONTROLLER: Uint32 = 0x00000008;
pub const OPEN_PROTOCOL_BY_DRIVER: Uint32 = 0x00000010;
pub const OPEN_PROTOCOL_EXCLUSIVE: Uint32 = 0x00000020;

#[repr(C)]
pub struct BootServices {
    pub header: TableHeader,
    pub raise_tpl: RaiseTPL,
    pub restore_tpl: RestoreTPL,

    pub allocate_pages: AllocatePages,
    pub free_pages: FreePages,
    pub get_memory_map: GetMemoryMap,
    pub allocate_pool: AllocatePool,
    pub free_pool: FreePool,

    pub create_event: CreateEvent,
    pub set_timer: SetTimer,
    pub wait_for_event: WaitForEvent,
    pub signal_event: SignalEvent,
    pub close_event: CloseEvent,
    pub check_event: CheckEvent,

    pub install_protocol_interface: InstallProtocolIenterface,
    pub reinstall_protocol_interface: ReinstallProtocolIenterface,
    pub uninstall_protocol_interface: UninstallProtocolIenterface,
    pub handle_protocol: HnadleProtocol,
    pub register_protocol_notify: RegisterProtocolNotify,
    reserved: *mut Void,
    pub locate_handle: LocateHandle,
    pub locate_device_path: LocateDevichPath,
    pub install_configuration_table: InstallConfigurationTable,

    pub load_image: ImageUnload,
    pub start_image: ImageStart,
    pub exit: Exit,
    pub unload_image: ImageUnload,
    pub exit_boot_services: ExitBootServices,

    pub get_next_monotonic_count: GetNextMonotonicCount,
    pub stall: Stall,
    pub set_watchdog_timer: SetWatchdogTimer,

    pub connect_controller: ConnectController,
    pub disconnect_controller: DisconnectController,

    pub open_protocol: OpenProtocol,
    pub close_protocol: CloseProtocol,
    pub open_protocol_information: OpenProtocolInformation,

    pub protocols_per_handle: ProtocolsPerHandle,
    pub locate_handle_buffer: LocateHandleBuffer,
    pub locate_protocol: LocateProtocol,
    pub install_multiple_protocol_interfaces: InstallMultipleProtocolInterfaces,
    pub uninstall_multiple_protocol_interfaces: UninstallMultipleProtocolInterfaces,

    pub calculate_crc32: CalculateCrc32,

    pub copy_mem: CopyMem,
    pub set_mem: SetMem,
    pub create_event_ex: CreateEventEx,
}

#[repr(C)]
pub enum MemoryType {
    EfiReservedMemoryType,
    EfiLoaderCode,
    EfiLoaderData,
    EfiBootServicesCode,
    EfiBootServicesData,
    EfiRuntimeServicesCode,
    EfiRuntimeServicesData,
    EfiConventionalMemory,
    EfiUnusableMemory,
    EfiACPIReclaimMemory,
    EfiACPIMemoryNVS,
    EfiMemoryMappedIO,
    EfiMemoryMappedIOPortSpace,
    EfiPalCode,
    EfiPersistentMemory,
    EfiUnacceptedMemoryType,
    EfiMaxMemoryType,
}

#[repr(C)]
pub enum AllocateType {
    AllocateAnyPages,
    AllocateMaxAddress,
    AllocateAddress,
    MaxAllocateType,
}

#[repr(C)]
pub struct MemoryDescriptor {
    pub type_: Uint32,
    pub phsycal_start: PhysicalAddress,
    pub virtual_address: VirtualAddress,
    pub number_of_pages: Uint64,
    pub attribute: Uint64,
}

const MEMORY_UC: Uint64 = 0x0000000000000001;
const MEMORY_WC: Uint64 = 0x0000000000000002;
const MEMORY_WT: Uint64 = 0x0000000000000004;
const MEMORY_WB: Uint64 = 0x0000000000000008;
const MEMORY_UCE: Uint64 = 0x0000000000000010;
const MEMORY_WP: Uint64 = 0x0000000000001000;
const MEMORY_RP: Uint64 = 0x0000000000002000;
const MEMORY_XP: Uint64 = 0x0000000000004000;
const MEMORY_NV: Uint64 = 0x0000000000008000;
const MEMORY_MORE_RELIABLE: Uint64 = 0x0000000000010000;
const MEMORY_RO: Uint64 = 0x0000000000020000;
const MEMORY_SP: Uint64 = 0x0000000000040000;
const MEMORY_CPU_CRYPTO: Uint64 = 0x0000000000080000;
const MEMORY_RUNTIME: Uint64 = 0x8000000000000000;
const MEMORY_ISA_VALID: Uint64 = 0x4000000000000000;
const MEMORY_ISA_MASK: Uint64 = 0x0FFFF00000000000;
