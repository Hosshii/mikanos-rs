use common::Zeroed;
use macros::bitfield_struct;

use crate::xhci::register_map::FromSegment;

/// packed byte size of type
pub trait PackedSize
where
    Self: Sized,
{
    const SIZE: usize = core::mem::size_of::<Self>();
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TryFromBytesError {
    InvalidLength,
    InvalidType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Descriptor {
    Device(DeviceDescriptor),
    Configuration(ConfigurationDescriptor),
    Interface(InterfaceDescriptor),
    Endpoint(EndpointDescriptor),
    HIDDescriptor(HIDDescriptor),
}

impl TryFrom<&[u8]> for Descriptor {
    type Error = TryFromBytesError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        fn packed_size(ty: Type) -> usize {
            use Type::*;
            match ty {
                Device => DeviceDescriptor::SIZE,
                Configuration => ConfigurationDescriptor::SIZE,
                String => todo!(),
                Interface => InterfaceDescriptor::SIZE,
                Endpoint => EndpointDescriptor::SIZE,
                HID => HIDDescriptor::SIZE,
            }
        }

        unsafe fn cast(ty: Type, value: &[u8]) -> Descriptor {
            use Type::*;
            unsafe {
                match ty {
                    Device => Descriptor::Device(*value.as_ptr().cast()),
                    Configuration => Descriptor::Configuration(*value.as_ptr().cast()),
                    String => todo!(),
                    Interface => Descriptor::Interface(*value.as_ptr().cast()),
                    Endpoint => Descriptor::Endpoint(*value.as_ptr().cast()),
                    HID => Descriptor::HIDDescriptor(*value.as_ptr().cast()),
                }
            }
        }

        let [len, ty, ..] = value else {
            return Err(TryFromBytesError::InvalidLength);
        };

        let ty = Type::try_from(*ty).map_err(|_| TryFromBytesError::InvalidType)?;

        if (*len as usize) < packed_size(ty) {
            return Err(TryFromBytesError::InvalidLength);
        }

        Ok(unsafe { cast(ty, value) })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    Device = 1,
    Configuration = 2,
    String = 3,
    Interface = 4,
    Endpoint = 5,
    HID = 33,
}

impl TryFrom<u8> for Type {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use Type::*;
        match value {
            1 => Ok(Device),
            2 => Ok(Configuration),
            3 => Ok(String),
            4 => Ok(Interface),
            5 => Ok(Endpoint),
            33 => Ok(HID),
            _ => Err(()),
        }
    }
}

impl From<Type> for u8 {
    fn from(value: Type) -> Self {
        value as u8
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroed)]
pub struct DeviceDescriptor {
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

impl DeviceDescriptor {
    pub const TYPE: Type = Type::Device;
}

impl FromSegment<18> for DeviceDescriptor {
    type Element = u8;

    fn from_segment(v: [Self::Element; 18]) -> Self {
        unsafe { *v.as_ptr().cast() }
    }
}

impl PackedSize for DeviceDescriptor {}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroed)]
pub struct ConfigurationDescriptor {
    pub len: u8,
    pub descriptor_type: u8,
    pub total_length: u16,
    pub num_interfaces: u8,
    pub configuration_value: u8,
    pub configuration_id: u8,
    pub attributes: u8,
    pub max_power: u8,
}

impl ConfigurationDescriptor {
    pub const TYPE: Type = Type::Configuration;
}

impl PackedSize for ConfigurationDescriptor {}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroed)]
pub struct InterfaceDescriptor {
    pub len: u8,
    pub descriptor_type: u8,
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub num_endpoint: u8,
    pub interface_class: u8,
    pub interface_sub_class: u8,
    pub interface_protocol: u8,
    pub interfacte_id: u8,
}

impl InterfaceDescriptor {
    pub const TYPE: Type = Type::Configuration;
}

impl PackedSize for InterfaceDescriptor {}

bitfield_struct! {
    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroed)]
    pub struct EndpointDescriptor {
        pub len: u8,
        pub descriptor_type: u8,
        pub endpoint_address: u8 => {
            #[bits(4)]
            number: u8,
            #[bits(3)]
            _: u8,
            #[bits(1)]
            dir_in: bool,
        },
        pub attributes: u8 => {
            #[bits(2)]
            transfer_type: u8,
            #[bits(2)]
            sync_type: u8,
            #[bits(2)]
            usage_type: u8,
            #[bits(2)]
            _: u8,
        },
        pub max_packet_size: u16,
        pub interval: u8,
    }
}

impl EndpointDescriptor {
    pub const TYPE: Type = Type::Endpoint;
}

impl PackedSize for EndpointDescriptor {}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroed)]
pub struct HIDDescriptor {
    length: u8,
    descriptor_type: u8,
    hid_version: u16,
    country_code: u8,
    num_descriptors: u8,
    report_descriptor_type: u8,
    report_descriptor_length: u16,
}

impl HIDDescriptor {
    pub const TYPE: Type = Type::HID;
}

impl PackedSize for HIDDescriptor {}
