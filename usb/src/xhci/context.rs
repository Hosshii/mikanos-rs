use core::ops::IndexMut;

use common::Zeroed;
use macros::bitfield_struct;

use super::port::PortWrapper;

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
                root_hub_port_number: u8,
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

#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Zeroed)]
pub struct DeviceContext {
    pub slot_context: SlotContext,
    pub device_contexts: [EndpointContxt; MAX_DEVICE_CONTEXT],
}

bitfield_struct! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Zeroed)]
    #[endian = "little"]
    pub struct InputControlContext {
        drop_context_flags: u32,
        add_context_flags: u32,
        _rsvdz: [u32; 5],
        data: u32 => {
            #[bits(8)]
            configuration_value: u8,
            #[bits(8)]
            interface_number: u8,
            #[bits(8)]
            altermate_setting: u8,
            #[bits(8)]
            _rsvdz: u8,
        }
    }
}

const EP_CONTEXT_NUM: usize = 32;

#[repr(C, align(64))]
#[derive(Debug, Clone, PartialEq, Eq, Zeroed)]
pub struct InputContext {
    input_control_context: InputControlContext,
    slot: SlotContext,
    ep_contexts: [EndpointContxt; EP_CONTEXT_NUM],
}

impl InputContext {
    pub fn enable_slot_context(&mut self) {
        self.input_control_context.add_context_flags |= 1;
    }

    pub fn enable_endpoint(&mut self, idx: u8) {
        self.input_control_context.add_context_flags |= 1 << idx;
    }

    pub fn init_slot_cx(&mut self, port: &PortWrapper) {
        let slot = &mut self.slot;
        slot.set_data_0_route_string(0);
        slot.set_data_1_root_hub_port_number(port.number());
        slot.set_data_0_context_entries(1);
        slot.set_data_0_speed(port.speed());
    }

    pub fn init_ep0_endpoint(
        &mut self,
        ring_ptr: *mut u8,
        cycle_state: bool,
        max_packet_size: u16,
    ) {
        let ep0 = self.ep_contexts.index_mut(0);
        ep0.set_data_1_ep_type(4); // control type
        ep0.set_data_1_max_packet_size(max_packet_size);
        ep0.set_data_1_max_burst_size(0);

        ep0.set_data_2_tr_dequeue_pointer_lo((ring_ptr as u32) >> 4);
        ep0.set_data_3_tr_dequeue_pointer_hi((ring_ptr as usize >> 32) as u32);

        ep0.set_data_2_dequeue_cycle_state(cycle_state);

        ep0.set_data_0_interval(0);
        ep0.set_data_0_max_primary_streams(0);
        ep0.set_data_0_mult(0);
        ep0.set_data_1_error_count(3);
    }
}
