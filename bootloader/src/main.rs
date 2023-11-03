#![no_std]
#![no_main]

use bootloader::{
    elf::{Elf, PT_LOAD},
    error::{Error, Result, ToRusult},
    logger,
};
use common::info;
use core::{arch::asm, fmt::Write, mem, panic::PanicInfo, ptr, slice};
use kernel::{KernelArg, KernelMain};
use macros::cstr16;
use uefi::{
    protocol::{
        console::{GraphicsOutputProtocol, GRAPHICS_OUTPUT_PROTOCOL_GUID},
        image::{LoadedImageProtocol, LOADED_IMAGE_PROTOCOL_GUID},
        media::{
            FileInfo, FileProtocol, SimpleFileSystemProtocol, FILE_INFO_GUID, FILE_MODE_READ,
            SIMPLE_FILE_SYSTEM_PROTOCOL_GUID,
        },
    },
    table::{
        boot_services::{
            AllocateType, BootServices, LocateSearchType, MemoryDescriptor, MemoryType,
            OPEN_PROTOCOL_BY_HANDLE_PROTOCOL,
        },
        system_table::SystemTable,
    },
    types::{Char16, Handle, Status, Uint32, Uintn, Void},
};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    halt()
}

#[no_mangle]
pub extern "efiapi" fn efi_main(image_handle: Handle, mut system_table: SystemTable) -> Status {
    unsafe { logger::init_logger(&mut system_table) }

    match main_impl(image_handle, &mut system_table) {
        Ok(_) => Status::SUCCRSS,
        Err(e) => {
            writeln!(system_table.stdout(), "kernel error: {}", e).unwrap();
            halt();
        }
    }
}

fn main_impl(image_handle: Handle, system_table: &mut SystemTable) -> Result<()> {
    system_table
        .clear_screen()
        .to_result()
        .map_err(|_| Error::Custom("failed to clear screen"))?;
    let stdout = system_table.stdout();
    writeln!(stdout, "hello world")?;

    let kernel_first_addr = load_kernel_file(image_handle, system_table)?;
    let kernel_arg = calc_kernel_arg(image_handle, system_table)?;

    let stack_base = alloc_stack(system_table)?;

    exit_boot_service(image_handle, system_table)?;

    let entry_addr = unsafe { *((kernel_first_addr + 24) as *const u64) } as *const ();
    let kernel_main: KernelMain = unsafe { mem::transmute(entry_addr) };

    init(kernel_main, kernel_arg, stack_base)
}

use arch::*;
#[cfg(target_arch = "x86_64")]
mod arch {
    use super::*;
    use core::mem::MaybeUninit;

    pub const STACK_SIZE: usize = 4 * 1024 * 1024;

    pub fn init(kernel_fn: KernelMain, arg: KernelArg, stack_base: u64) -> ! {
        unsafe {
            ARG.write(arg);
            FN.write(kernel_fn);
        }

        unsafe {
            asm! {
                "mov rsp, {x}",
                x = in(reg) stack_base
            }
        }

        unsafe { FN.assume_init()(ARG.assume_init_ref()) }
    }

    static mut ARG: MaybeUninit<KernelArg> = MaybeUninit::uninit();
    static mut FN: MaybeUninit<KernelMain> = MaybeUninit::uninit();
}

fn alloc_stack(system_table: &SystemTable) -> Result<u64> {
    let num_pages = (STACK_SIZE + 0xfff) / 0x1000;
    let mut stack_base = 0;
    (system_table.boot_services().allocate_pages)(
        AllocateType::AllocateAnyPages,
        MemoryType::EfiLoaderData,
        num_pages,
        &mut stack_base,
    )
    .to_result()
    .map_err(|_| Error::Custom("failed to allocate pages"))?;

    Ok(stack_base)
}

fn calc_kernel_arg(image_handle: Handle, system_table: &SystemTable) -> Result<KernelArg> {
    let gop = open_gop(system_table.boot_services(), image_handle)?;
    let mode = gop.mode();
    let info = mode.info();

    let frame_buffer_base = mode.frame_buffer_base;
    let frame_buffer_size = mode.frame_buffer_size;

    let frame_buffer = mode.frame_buffer_base as *mut u8;
    for offset in 0..mode.frame_buffer_size {
        unsafe {
            frame_buffer.add(offset).write_volatile(255);
        }
    }

    info!(
        "resolutoin: {}x{}, pixel format: {}, {}",
        info.horizontal_resolution,
        info.vertical_resolution,
        info.pixel_format,
        info.pixel_per_scan_line
    );
    info!(
        "frame buffer: {:p} - {:p}, size: {:x}",
        frame_buffer_base as *const u8,
        (frame_buffer_base + frame_buffer_size as u64) as *const u8,
        frame_buffer_size
    );

    let pixel_format = match &info.pixel_format {
        uefi::protocol::console::PixelFormat::PixelBlueGreenRedReserved8BitPerColor => {
            kernel::PixelFormat::PixelBGRResv8BitPerColor
        }
        uefi::protocol::console::PixelFormat::PixelRedGreenBlueReserved8BitPerColor => {
            kernel::PixelFormat::PixelRGBResv8BitPerColor
        }
        f => {
            info!("unsupported pixel format: {}", f);
            return Err(Error::Custom("unsupported pixel format"));
        }
    };
    let kernel_arg = unsafe {
        KernelArg::new(
            frame_buffer_base as *mut u8,
            mode.frame_buffer_size,
            info.pixel_per_scan_line,
            info.horizontal_resolution,
            info.vertical_resolution,
            pixel_format,
        )
    };

    Ok(kernel_arg)
}

fn load_kernel_file(image_handle: Handle, system_table: &mut SystemTable) -> Result<u64> {
    info!("open root dir");
    let root_dir = open_root_dir(image_handle, system_table.boot_services())?;

    // カーネルファイルを開く
    info!("open kernel file");
    let mut kernel_file = ptr::null_mut();
    unsafe {
        ((*root_dir).open)(
            &*root_dir,
            &mut kernel_file,
            cstr16!("\\kernel.elf").as_ptr(),
            FILE_MODE_READ,
            0,
        )
        .to_result()
        .map_err(|_| Error::Custom("cannot open \\kernel.elf"))?
    };

    info!("get file info");
    const FILE_INFO_SIZE: Uintn = mem::size_of::<FileInfo>() + mem::size_of::<Char16>() * 12;
    let mut file_info_size = FILE_INFO_SIZE;
    let mut file_info_buffer: [Uintn; FILE_INFO_SIZE] = [0; FILE_INFO_SIZE];
    unsafe {
        ((*kernel_file).get_info)(
            &*kernel_file,
            &FILE_INFO_GUID,
            &mut file_info_size,
            file_info_buffer.as_mut_ptr() as *mut Void,
        )
    }
    .to_result()
    .map_err(|_| Error::Custom("cannot get info"))?;

    let file_info = unsafe { &*(file_info_buffer.as_ptr() as *const FileInfo) };

    let mut kernel_file_size = file_info.file_size as Uintn;
    let mut kernel_buffer = ptr::null_mut();
    info!("allocate pool. kernel_file_size: {kernel_file_size}");
    (system_table.boot_services().allocate_pool)(
        MemoryType::EfiLoaderData,
        kernel_file_size,
        &mut kernel_buffer,
    )
    .to_result()
    .map_err(|_| Error::Custom("cannot allocate pool"))?;

    info!("read kernel file");
    unsafe { ((*kernel_file).read)(&*kernel_file, &mut kernel_file_size, kernel_buffer) }
        .to_result()
        .map_err(|_| Error::Custom("cannot read kernel file"))?;

    let kernel_buffer = kernel_buffer as *const u8;

    let elf = unsafe { Elf::from_raw_parts(kernel_buffer, kernel_file_size) }?;

    let (kernel_first_addr, kernel_last_addr) = elf.calc_loader_addr_range();
    info!("kernel_first_addr: {kernel_first_addr}, kernel_last_addr: {kernel_last_addr}");

    info!("allocate pages");
    let num_pages = (kernel_last_addr - kernel_first_addr + 0xfff) / 0x1000;
    let mut kernel_first_addr = kernel_first_addr as u64;
    (system_table.boot_services().allocate_pages)(
        AllocateType::AllocateAddress,
        MemoryType::EfiLoaderData,
        num_pages,
        &mut kernel_first_addr,
    )
    .to_result()
    .map_err(|_| Error::Custom("failed to allocate pages"))?;

    info!("copy segments");
    unsafe { copy_load_segments(&elf, kernel_buffer) }
    writeln!(
        system_table.stdout(),
        "kernel: 0x{:x} - 0x{:x}",
        kernel_first_addr,
        kernel_last_addr
    )?;

    info!("free pool");
    (system_table.boot_services().free_pool)(kernel_buffer as *mut Void)
        .to_result()
        .map_err(|_| Error::Custom("failed to free pages"))?;

    Ok(kernel_first_addr)
}

fn exit_boot_service(image_handle: Handle, system_table: &SystemTable) -> Result<()> {
    const MEMORY_MAP_SIZE: usize = 4096 * 4;
    let mut memorymap_buf = [0u8; MEMORY_MAP_SIZE];
    let mut memory_map = MemoryMap {
        buffer_size: MEMORY_MAP_SIZE,
        buffer: &mut memorymap_buf as *mut [u8; MEMORY_MAP_SIZE] as *mut Void,
        map_size: 0,
        map_key: 0,
        descriptor_size: 0,
        descriptor_version: 0,
    };
    info!("get memory map");
    get_memory_map(system_table, &mut memory_map)?;

    info!("exit boot services :{}", memory_map.map_key);
    let status =
        (system_table.boot_services().exit_boot_services)(image_handle, memory_map.map_key);
    // .map_err(|_| Error::Custom("failed to exit boot services"))?;
    if status.is_err() {
        info!("get memory map");
        info!("exit boot services 2: {}", memory_map.map_key);
        get_memory_map(system_table, &mut memory_map)?;

        (system_table.boot_services().exit_boot_services)(image_handle, memory_map.map_key)
            .to_result()
            .map_err(|_| Error::Custom("failed to exit boot services"))?;
    }

    Ok(())
}

fn open_root_dir(image_handle: Handle, boot_services: &BootServices) -> Result<*mut FileProtocol> {
    info!("load image");
    let mut loaded_image = ptr::null_mut();
    (boot_services.open_protocol)(
        image_handle,
        &LOADED_IMAGE_PROTOCOL_GUID,
        &mut loaded_image,
        image_handle,
        ptr::null_mut(),
        OPEN_PROTOCOL_BY_HANDLE_PROTOCOL,
    )
    .to_result()
    .map_err(|_| Error::Custom("cannot open load image protocol"))?;

    info!("open fs");
    let loaded_image = loaded_image as *mut LoadedImageProtocol;
    let mut fs = ptr::null_mut();

    (boot_services.open_protocol)(
        unsafe { (*loaded_image).device_handle },
        &SIMPLE_FILE_SYSTEM_PROTOCOL_GUID,
        &mut fs,
        image_handle,
        ptr::null_mut(),
        OPEN_PROTOCOL_BY_HANDLE_PROTOCOL,
    )
    .to_result()
    .map_err(|_| Error::Custom("cannot open fs protocol"))?;

    let fs = fs as *mut SimpleFileSystemProtocol;
    let mut root = ptr::null_mut();

    info!("open volume");
    unsafe { ((*fs).open_volume)(&*fs, &mut root) }
        .to_result()
        .map_err(|_| Error::Custom("cannot open root volume"))?;

    Ok(root)
}

unsafe fn copy_load_segments(elf: &Elf, buf: *const u8) {
    for segment in elf.program_header() {
        if segment.type_() != PT_LOAD {
            continue;
        }
        let offset = segment.offset() as usize;
        let src = buf.add(offset);
        let dst = segment.virtual_addr() as *mut u8;
        let count = segment.file_size() as usize;

        ptr::copy(src, dst, count);

        let dst = dst.add(count);
        let remain = segment.mem_size() - segment.file_size();
        ptr::write_bytes(dst, 0, remain as usize)
    }
}

#[cfg(target_arch = "x86_64")]
#[inline]
fn halt() -> ! {
    loop {
        unsafe {
            asm! {"hlt"}
        }
    }
}

#[cfg(target_arch = "aarch64")]
#[inline]
fn halt() -> ! {
    loop {
        unsafe {
            asm! {"wfi"}
        }
    }
}

struct MemoryMap {
    buffer_size: Uintn,
    buffer: *mut Void,
    map_size: Uintn,
    map_key: Uintn,
    descriptor_size: Uintn,
    descriptor_version: Uint32,
}

fn get_memory_map(system_table: &SystemTable, map: &mut MemoryMap) -> Result<()> {
    if map.buffer.is_null() {
        return Err(Error::Custom("buffer is too small"));
    }

    map.map_size = map.buffer_size;
    (system_table.boot_services().get_memory_map)(
        &mut map.map_size,
        map.buffer as *mut MemoryDescriptor,
        &mut map.map_key,
        &mut map.descriptor_size,
        &mut map.descriptor_version,
    )
    .to_result()
    .map_err(|_| Error::Custom("failed to get memory map"))
}

fn open_gop(boot_services: &BootServices, image_handle: Handle) -> Result<&GraphicsOutputProtocol> {
    let mut num_handle = 0;
    let mut gop_handles = ptr::null_mut();
    (boot_services.locate_handle_buffer)(
        LocateSearchType::ByProtocol,
        &GRAPHICS_OUTPUT_PROTOCOL_GUID,
        ptr::null(),
        &mut num_handle,
        &mut gop_handles,
    )
    .to_result()
    .map_err(|_| Error::Custom("cannot locate handle buffer"))?;
    let gop_handles = unsafe { slice::from_raw_parts_mut(gop_handles, num_handle) };

    let mut gop = ptr::null_mut();
    (boot_services.open_protocol)(
        gop_handles[0],
        &GRAPHICS_OUTPUT_PROTOCOL_GUID,
        &mut gop,
        image_handle,
        ptr::null_mut(),
        OPEN_PROTOCOL_BY_HANDLE_PROTOCOL,
    )
    .to_result()
    .map_err(|_| Error::Custom("cannot open gop"))?;
    (boot_services.free_pool)(gop_handles.as_mut_ptr() as *mut Void)
        .to_result()
        .map_err(|_| Error::Custom("cannot free gop handles"))?;

    Ok(unsafe { &*(gop as *const GraphicsOutputProtocol) })
}
