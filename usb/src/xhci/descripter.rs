use common::Zeroed;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    Device = 1,
    Configuration = 2,
    String = 3,
    Interface = 4,
    Endpoint = 5,
    HID = 33,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroed)]
pub struct DeviceDescripter {
    pub length: u8,
    pub descripter_type: u8,
    pub usb_release: u16,
    pub device_class: u8,
    pub device_sub_class: u8,
    pub device_protocol: u8,
    pub max_packet_size: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_release: u16,
    pub manufactuer: u8,
    pub product: u8,
    pub serial_number: u8,
    pub num_configurations: u8,
}
