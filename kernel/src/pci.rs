/// # PCI Configuration Space Register Layout
///
/// PCIデバイスの設定と状態を管理するための256バイトのメモリ領域。
///
/// ## Register Map (Type 00 Header)
///
/// ```plaintext
/// Offset (hex)   |31                                         16|15                                          0|
/// ---------------+---------------------------------------------+---------------------------------------------|
/// 00h            |     Device ID                               |     Vendor ID                               |
/// ---------------+---------------------------------------------+---------------------------------------------|
/// 04h            |        Status                               |      Command                                |
/// ---------------+-------------------------------------------------------------------------------------------|
/// 08h            |    Class Code                                                      |    Revision ID       |
/// ---------------+-------------------------------------------------------------------------------------------|
/// 0Ch            |         BIST         |     Header  Type     |    Latency  Timer    |    Cacheline Size    |
/// ---------------+-------------------------------------------------------------------------------------------|
/// 10h            |                               Base Address Registers (BARs)                               |
/// ---------------+-------------------------------------------------------------------------------------------|
/// 28h            |                                    Cardbus CIS Pointer                                    |
/// ---------------+-------------------------------------------------------------------------------------------|
/// 2Ch            |                 Subsystem ID                |             Subsystem Vendor ID             |
/// ---------------+-------------------------------------------------------------------------------------------|
/// 30h            |                                 Expansion ROM Base Address                                |
/// ---------------+-------------------------------------------------------------------------------------------|
/// 34h            |                               Reserved                             | Capabilities Pointer |
/// ---------------+-------------------------------------------------------------------------------------------|
/// 38h            |                                          Reserved                                         |
/// ---------------+-------------------------------------------------------------------------------------------|
/// 3Ch            |        Max_Lat       |        Min_Gnt       |      Interrupt Pin   |    Interrupt Line    |
/// ---------------+-------------------------------------------------------------------------------------------|
/// 40h            |                                  Device Specific Region                                   |
/// ---------------+-------------------------------------------------------------------------------------------|
/// ```
use crate::error::{Error, Result};

#[cfg(target_arch = "x86_64")]
mod arch {
    use core::arch::asm;

    pub const CONFIG_ADDRESS: u16 = 0x0cf8;
    pub const CONFIG_DATA: u16 = 0x0cfc;

    pub fn io_out32(addr: u16, data: u32) {
        unsafe { asm!("out dx, eax",in("dx") addr, in("eax") data) }
    }

    pub fn io_in32(addr: u16) -> u32 {
        let mut result: u32;
        unsafe { asm!("in eax, dx", out("eax") result, in("dx") addr) }
        result
    }
}

use arch::*;
use common::debug;

fn write_address(address: u32) {
    io_out32(CONFIG_ADDRESS, address);
}

fn write_data(v: u32) {
    io_out32(CONFIG_DATA, v.to_le());
}

fn read_data() -> u32 {
    u32::from_le(io_in32(CONFIG_DATA))
}

fn make_address(bus: u8, device: u8, function: u8, reg_addr: u8) -> Result<u32> {
    const DEVICE_MAX: u8 = 31;
    const FUNCTION_MAX: u8 = 7;

    if device > DEVICE_MAX || function > FUNCTION_MAX {
        return Err(Error::invalid_addr());
    }

    fn shl(x: u8, bit: u32) -> u32 {
        (x as u32) << bit
    }

    Ok(shl(1, 31) | shl(bus, 16) | shl(device, 11) | shl(function, 8) | (reg_addr as u32 & 0xfc))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct VenderID(u16);
impl VenderID {
    const INTEL: u16 = 0x8086;
    const INVALID: u16 = 0xffff;

    pub fn new(v: u16) -> Self {
        Self(v)
    }

    pub fn is_invalid(self) -> bool {
        self.0 == Self::INVALID
    }

    pub fn intel() -> Self {
        Self::new(Self::INTEL)
    }

    pub fn is_intel(&self) -> bool {
        self.0 == Self::INTEL
    }
}

fn read_vender_id(bus: u8, device: u8, function: u8) -> Result<VenderID> {
    let addr = make_address(bus, device, function, 0x00)?;
    write_address(addr);

    Ok(VenderID::new(read_data() as u16))
}

fn read_device_id(bus: u8, device: u8, function: u8) -> Result<u16> {
    let addr = make_address(bus, device, function, 0x00)?;
    write_address(addr);

    Ok((read_data() >> 16) as u16)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
struct HeaderType(u8);
impl HeaderType {
    pub fn new(v: u8) -> Self {
        Self(v)
    }

    pub fn is_single_fn_divice(self) -> bool {
        self.0 & 0x80 == 0
    }
}

fn read_header_type(bus: u8, device: u8, function: u8) -> Result<HeaderType> {
    let addr = make_address(bus, device, function, 0x0c)?;
    write_address(addr);

    Ok(HeaderType::new((read_data() >> 16) as u8))
}

fn read_class_code(bus: u8, device: u8, function: u8) -> Result<ClassCode> {
    let addr = make_address(bus, device, function, 0x08)?;
    write_address(addr);
    let reg = read_data();
    let cc = ClassCode::new((reg >> 24) as u8, (reg >> 16) as u8, (reg >> 8) as u8);

    Ok(cc)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeviceType {
    PciPciBridge,
    UsbController,
    Unknown,
}

impl From<ClassCode> for DeviceType {
    fn from(value: ClassCode) -> Self {
        match (value.base, value.sub) {
            (0x06, 0x04) => DeviceType::PciPciBridge,
            (0x0c, 0x03) => DeviceType::UsbController,
            _ => DeviceType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Interface(u8);

impl Interface {
    pub fn new(v: u8) -> Self {
        Self(v)
    }
}

pub enum UsbType {
    XHci,
}

impl TryFrom<Interface> for UsbType {
    type Error = ();

    fn try_from(value: Interface) -> core::result::Result<Self, Self::Error> {
        match value.0 {
            0x30 => Ok(UsbType::XHci),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ClassCode {
    base: u8,
    sub: u8,
    interface: Interface,
}

impl ClassCode {
    pub fn new(base: u8, sub: u8, interface: u8) -> Self {
        Self {
            base,
            sub,
            interface: Interface::new(interface),
        }
    }

    pub fn interface(&self) -> Interface {
        self.interface
    }
}

fn read_bus_number(bus: u8, device: u8, function: u8) -> Result<u32> {
    let addr = make_address(bus, device, function, 0x18)?;
    write_address(addr);

    Ok(read_data())
}

const MAX_DEVICES: u8 = 32;
const MAX_FUNCTION_NUM: u8 = 8;
pub struct Pci {
    devices_buffer: [Device; MAX_DEVICES as usize],
    device_num: u8,
}

impl Pci {
    pub fn new() -> Self {
        let d = Device::default();
        Self {
            devices_buffer: [d; MAX_DEVICES as usize],
            device_num: 0,
        }
    }

    pub fn scan_all_bus(&mut self) -> Result<()> {
        // host bridge
        let header_type = read_header_type(0, 0, 0)?;
        if header_type.is_single_fn_divice() {
            return self.scan_bus(0);
        }

        for function in 1..MAX_FUNCTION_NUM {
            if read_vender_id(0, 0, function)?.is_invalid() {
                continue;
            }

            self.scan_bus(function)?;
        }

        Ok(())
    }

    pub fn devices(&self) -> &[Device] {
        &self.devices_buffer[0..self.device_num as usize]
    }

    fn scan_bus(&mut self, bus: u8) -> Result<()> {
        for device in 0..MAX_DEVICES {
            if read_vender_id(bus, device, 0)?.is_invalid() {
                continue;
            }

            self.scan_device(bus, device)?;
        }

        Ok(())
    }

    fn scan_device(&mut self, bus: u8, device: u8) -> Result<()> {
        self.scan_function(bus, device, 0)?;

        if read_header_type(bus, device, 0)?.is_single_fn_divice() {
            return Ok(());
        }

        for function in 0..MAX_FUNCTION_NUM {
            if read_vender_id(bus, device, function)?.is_invalid() {
                continue;
            }

            self.scan_function(bus, device, function)?;
        }

        Ok(())
    }

    fn scan_function(&mut self, bus: u8, device: u8, function: u8) -> Result<()> {
        let class_code = read_class_code(bus, device, function)?;
        let hedaer_type = read_header_type(bus, device, function)?;
        let dev = Device::new(bus, device, function, hedaer_type, class_code);
        self.add_device(dev)?;

        let device_type = DeviceType::from(class_code);

        if matches!(device_type, DeviceType::PciPciBridge) {
            let bus_number = read_bus_number(bus, device, function)?;
            let secondary_bus = (bus_number >> 8) as u8;
            return self.scan_bus(secondary_bus);
        }

        Ok(())
    }

    fn add_device(&mut self, dev: Device) -> Result<()> {
        if self.device_num as usize == self.devices_buffer.len() {
            return Err(Error::too_many_devices());
        }

        self.devices_buffer[self.device_num as usize] = dev;
        self.device_num += 1;
        Ok(())
    }
}

impl Default for Pci {
    fn default() -> Self {
        Self::new()
    }
}

impl PciExtUsb for Pci {
    fn find_usb(&self) -> Option<&Device> {
        self.devices()
            .iter()
            .fold(None, |acc: Option<&Device>, cur| {
                match (
                    DeviceType::from(cur.class_code()),
                    UsbType::try_from(cur.class_code().interface()),
                ) {
                    (DeviceType::UsbController, Ok(UsbType::XHci)) => match acc {
                        Some(device) => {
                            if device.read_vender_id().ok()?.is_intel() {
                                Some(device)
                            } else {
                                Some(cur)
                            }
                        }
                        None => Some(cur),
                    },
                    _ => None,
                }
            })
    }

    fn switch_ehci2xhci(&self, usb_dev: &Device) -> Result<()> {
        let intel_ehc_exits = self.devices().iter().any(|dev| {
            matches!(
                (
                    DeviceType::from(dev.class_code()),
                    UsbType::try_from(dev.class_code().interface())
                ),
                (DeviceType::UsbController, Ok(UsbType::XHci))
            ) && dev.read_vender_id().map(|v| v.is_intel()).unwrap_or(false)
        });

        if !intel_ehc_exits {
            return Ok(());
        }

        let super_speed_port = usb_dev.read_reg(0xdc)?; // USB3PRM
        usb_dev.write_reg(0xd8, super_speed_port)?;
        let ehci2xhci_ports = usb_dev.read_reg(0xd4)?;
        usb_dev.write_reg(0xd0, ehci2xhci_ports)?;
        debug!(
            "switch ehci to xhci: ss= {}, xhci= {}",
            super_speed_port, ehci2xhci_ports
        );

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Device {
    bus: u8,
    device: u8,
    function: u8,
    header_type: HeaderType,
    class_code: ClassCode,
}

impl Device {
    fn new(
        bus: u8,
        device: u8,
        function: u8,
        header_type: HeaderType,
        class_code: ClassCode,
    ) -> Self {
        Self {
            bus,
            device,
            function,
            header_type,
            class_code,
        }
    }

    pub fn read_vender_id(&self) -> Result<VenderID> {
        read_vender_id(self.bus, self.device, self.function)
    }

    pub fn read_bar(&self, bar_index: u8) -> Result<u32> {
        const BAR_OFFSET: u8 = 0x10;
        if 5 < bar_index {
            return Err(Error::out_of_range_bar());
        }

        let addr = make_address(
            self.bus,
            self.device,
            self.function,
            BAR_OFFSET + 4 * bar_index,
        )?;
        write_address(addr);
        let bar = read_data();

        Ok(bar)
    }

    fn read_reg(&self, reg: u8) -> Result<u32> {
        let addr = make_address(self.bus, self.device, self.function, reg)?;
        write_address(addr);
        Ok(read_data())
    }

    fn write_reg(&self, reg: u8, v: u32) -> Result<()> {
        let addr = make_address(self.bus, self.device, self.function, reg)?;
        write_address(addr);
        write_data(v);
        Ok(())
    }

    pub fn class_code(&self) -> ClassCode {
        self.class_code
    }
}

pub trait PciExtUsb {
    fn find_usb(&self) -> Option<&Device>;
    fn switch_ehci2xhci(&self, usb_dev: &Device) -> Result<()>;
}
