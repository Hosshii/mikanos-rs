#![no_std]
#![no_main]

use bootloader::{
    elf::{Elf, PT_LOAD},
    error::{Error, Result, ToRusult},
    info, log,
};

use core::{arch::asm, fmt::Write, mem, panic::PanicInfo, ptr};
use macros::cstr16;
use uefi::{
    protocol::{
        image::{LoadedImageProtocol, LOADED_IMAGE_PROTOCOL_GUID},
        media::{
            FileInfo, FileProtocol, SimpleFileSystemProtocol, FILE_INFO_GUID, FILE_MODE_READ,
            SIMPLE_FILE_SYSTEM_PROTOCOL_GUID,
        },
    },
    table::{
        boot_services::{
            AllocateType, BootServices, MemoryDescriptor, MemoryType,
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
    unsafe { log::init_logger(&mut system_table) }

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

    const MEMORY_MAP_SIZE: usize = 4096 * 4;
    let mut memorymap_buf = [0u8; MEMORY_MAP_SIZE];
    let mut memeory_map = MemoryMap {
        buffer_size: MEMORY_MAP_SIZE,
        buffer: &mut memorymap_buf as *mut [u8; MEMORY_MAP_SIZE] as *mut Void,
        map_size: 0,
        map_key: 0,
        descriptor_size: 0,
        descriptor_version: 0,
    };
    info!("get memory map");
    get_memory_map(system_table, &mut memeory_map)?;

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

    info!("allocate pool");
    let file_info = file_info_buffer.as_ptr() as *const FileInfo;
    let mut kernel_file_size = unsafe { (*file_info).file_size } as Uintn;
    let mut kernel_buffer = ptr::null_mut();
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

    info!("allocate pages");
    let num_pages = (kernel_last_addr - kernel_first_addr + 0xffff) / 0x10000;
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
        "kernel: 0x{} - 0x{}",
        kernel_first_addr,
        kernel_last_addr
    )?;

    info!("free pool");
    (system_table.boot_services().free_pool)(kernel_buffer as *mut Void)
        .to_result()
        .map_err(|_| Error::Custom("failed to free pages"))?;

    info!("exit boot services :{}", memeory_map.map_key);
    let status =
        (system_table.boot_services().exit_boot_services)(image_handle, memeory_map.map_key);
    // .map_err(|_| Error::Custom("failed to exit boot services"))?;
    if status.is_err() {
        info!("get memory map");
        info!("exit boot services 2: {}", memeory_map.map_key);
        get_memory_map(system_table, &mut memeory_map)?;

        (system_table.boot_services().exit_boot_services)(image_handle, memeory_map.map_key)
            .to_result()
            .map_err(|_| Error::Custom("failed to exit boot services"))?;
    }

    let entry_addr = (kernel_first_addr + 24) as *const ();
    let kernel_main: extern "C" fn() -> ! = unsafe { mem::transmute(entry_addr) };

    kernel_main();
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
fn halt() -> ! {
    loop {
        unsafe {
            asm! {"hlt"}
        }
    }
}

#[cfg(target_arch = "aarch64")]
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
