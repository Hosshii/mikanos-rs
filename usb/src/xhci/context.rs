use common::Zeroed;
use macros::bitfield_struct;

bitfield_struct! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default,Zeroed)]
    #[endian = "little"]
    pub struct SlotContext {
        data: [u32; 8] => [
            {
                #[bits(20)]
                route_string: u32,
                #[bits(4)]
                speed: u8,
                #[bits(1)]
                _rsvdz1: bool,
                #[bits(1)]
                mtt: bool,
                #[bits(1)]
                hub: bool,
                #[bits(5)]
                context_entries: u8,
            },
            {
                #[bits(16)]
                max_exit_latency: u16,
                #[bits(8)]
                root_hub_number: u8,
                #[bits(8)]
                number_of_ports: u8,
            },
            {
                #[bits(8)]
                tt_hub_slot_id: u8,
                #[bits(8)]
                tt_port_number: u8,
                #[bits(2)]
                ttt: u8,
                #[bits(4)]
                _rsvdz: u8,
                #[bits(10)]
                interrupt_target: u16
            },
            {
                #[bits(8)]
                usb_device_address: u8,
                #[bits(19)]
                _rsvdz: u32,
                #[bits(5)]
                slot_state: u8,
            },
            {
                #[bits(32)]
                _rsvdz: u32,
            },
            {
                #[bits(32)]
                _rsvdz: u32,
            },
            {
                #[bits(32)]
                _rsvdz: u32,
            },
            {
                #[bits(32)]
                _rsvdz: u32,
            },
        ]
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Zeroed)]
    #[endian = "little"]
    pub struct EndpointContxt {
        data: [u32; 8] => [
            {
                #[bits(3)]
                ep_state: u8,
                #[bits(5)]
                _rsvdz: u8,
                #[bits(2)]
                mult: u8,
                #[bits(5)]
                max_primary_streams: u8,
                #[bits(1)]
                linear_stram_array: bool,
                #[bits(8)]
                interval: u8,
                #[bits(8)]
                max_esit_payload_hi: u8,
            },
            {
                #[bits(1)]
                _rsvdz1: bool,
                #[bits(2)]
                error_count: u8,
                #[bits(3)]
                ep_type: u8,
                #[bits(1)]
                _rsvdz2: bool,
                #[bits(1)]
                host_initiate_disable: bool,
                #[bits(8)]
                max_burst_size: u8,
                #[bits(16)]
                max_packet_size: u16,
            },
            {
                #[bits(1)]
                dequeue_cycle_state: bool,
                #[bits(3)]
                _rsvdz: u8,
                #[bits(28)]
                tr_dequeue_pointer_lo: u32,
            },
            {
                #[bits(32)]
                tr_dequeue_pointer_hi: u32,
            },
            {
                #[bits(16)]
                average_trb_length: u16,
                #[bits(16)]
                max_esit_payload_lo: u16,
            },
            {
                #[bits(32)]
                _rsvdz: u32,
            },
            {
                #[bits(32)]
                _rsvdz: u32,
            },
            {
                #[bits(32)]
                _rsvdz: u32,
            },
        ]
    }
}

const MAX_DEVICE_CONTEXT: usize = 31;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Zeroed)]
pub struct DeviceContext {
    slot_context: SlotContext,
    device_contexts: [EndpointContxt; MAX_DEVICE_CONTEXT],
}

impl DeviceContext {
    pub fn new() -> Self {
        Self {
            slot_context: SlotContext::default(),
            device_contexts: [EndpointContxt::default(); MAX_DEVICE_CONTEXT],
        }
    }
}

impl Default for DeviceContext {
    fn default() -> Self {
        Self::new()
    }
}
