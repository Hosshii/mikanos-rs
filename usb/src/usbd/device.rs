use core::{marker::PhantomPinned, pin::Pin};

use super::{descriptor::DeviceDescriptor, endpoint::EndpointID};
use crate::{
    usbd::endpoint::HCP_ENDPOINT_ID,
    xhci::{
        device::{Device as XHCIDevice, SlotId, DEFAULT_TRANSFER_RING_SIZE},
        doorbell::DCDoorbell,
        ring::TCRing,
        trb::{DataStage, SetupStage, StatusStage},
    },
};
use common::{info, Zeroed};

pub const DEFAULT_BUF_SIZE: usize = 256;
pub struct Device<'a, 'b, const BUF: usize = DEFAULT_BUF_SIZE> {
    buf: &'b mut [u8; BUF],
    device: Pin<&'a mut XHCIDevice>,
    _phatmot_pinned: PhantomPinned,
}

impl<'a, 'b, const BUF: usize> Device<'a, 'b, BUF> {
    pub fn new(buf: &'b mut [u8; BUF], device: Pin<&'a mut XHCIDevice>) -> Self {
        Self {
            buf,
            device,
            _phatmot_pinned: PhantomPinned,
        }
    }

    unsafe fn ring_mut(&mut self, id: u8) -> &mut TCRing<DEFAULT_TRANSFER_RING_SIZE> {
        unsafe { self.device.as_mut().get_unchecked_mut().ring_mut(id) }
    }

    pub fn buf(&self) -> &[u8; BUF] {
        &self.buf
    }

    pub fn slot_id(&self) -> SlotId {
        self.device.slot_id()
    }

    pub fn request_device_descripter(
        &mut self,
        endponint_id: EndpointID,
        mut doorbell: DCDoorbell,
        interface_num: u16,
    ) {
        let buf_len = BUF;
        let setup = SetupStage::zeroed()
            .with_parameter0_bm_request_type(0b10000000)
            .with_parameter0_b_request(6)
            .with_parameter0_w_value(0x0100)
            .with_parameter1_w_index(interface_num)
            .with_parameter1_w_length(buf_len as u16)
            .with_status_trb_transfer_length(8)
            .with_control_transfer_type(3)
            // .with_remain_interrupt_on_completion(true)
            .with_remain_immediate_data(true);

        let buf_ptr = self.buf.as_mut_ptr();
        let data = DataStage::zeroed()
            .with_buf_ptr_lo(buf_ptr as u32)
            .with_buf_ptr_hi(((buf_ptr as usize) >> 32) as u32)
            .with_status_trb_transfer_length(buf_len as u32)
            .with_status_td_size(0)
            .with_control_dir(true)
            .with_remain_interrupt_on_completion(true);

        let status = StatusStage::zeroed().with_control_direction(false);
        // .with_remain_interrupt_on_completion(true);

        let transfer_ring = unsafe { self.ring_mut(endponint_id.dci()) };
        info!(
            "ring_ptr in request: {}",
            transfer_ring.as_mut_ptr() as usize
        );
        transfer_ring.push(setup);
        transfer_ring.push(data);
        transfer_ring.push(status);
        doorbell.notify_endpoint(HCP_ENDPOINT_ID.dci());
    }

    pub fn request_configuration_descriptor(
        &mut self,
        endponint_id: EndpointID,
        mut doorbell: DCDoorbell,
        interface_num: u16,
    ) {
        let buf_len = BUF;
        let setup = SetupStage::zeroed()
            .with_parameter0_bm_request_type(0b10000000)
            .with_parameter0_b_request(6)
            .with_parameter0_w_value(0x0200)
            .with_parameter1_w_index(interface_num)
            .with_parameter1_w_length(buf_len as u16)
            .with_status_trb_transfer_length(8)
            .with_control_transfer_type(3)
            // .with_remain_interrupt_on_completion(true)
            .with_remain_immediate_data(true);

        let buf_ptr = self.buf.as_mut_ptr();
        let data = DataStage::zeroed()
            .with_buf_ptr_lo(buf_ptr as u32)
            .with_buf_ptr_hi(((buf_ptr as usize) >> 32) as u32)
            .with_status_trb_transfer_length(buf_len as u32)
            .with_status_td_size(0)
            .with_control_dir(true)
            .with_remain_interrupt_on_completion(true);

        let status = StatusStage::zeroed().with_control_direction(false);
        // .with_remain_interrupt_on_completion(true);

        let transfer_ring = unsafe { self.ring_mut(endponint_id.dci()) };
        info!(
            "ring_ptr in request: {}",
            transfer_ring.as_mut_ptr() as usize
        );
        transfer_ring.push(setup);
        transfer_ring.push(data);
        transfer_ring.push(status);
        doorbell.notify_endpoint(HCP_ENDPOINT_ID.dci());
    }

    pub fn request_boot_protocol_descriptor(
        &mut self,
        endponint_id: EndpointID,
        mut doorbell: DCDoorbell,
    ) {
        let setup = SetupStage::zeroed()
            .with_parameter0_bm_request_type(0b00100001)
            .with_parameter0_b_request(11)
            .with_parameter0_w_value(0)
            .with_parameter1_w_index(0)
            .with_parameter1_w_length(0)
            .with_status_trb_transfer_length(8)
            .with_control_transfer_type(0)
            // .with_remain_interrupt_on_completion(true)
            .with_remain_immediate_data(true);

        let transfer_ring = unsafe { self.ring_mut(endponint_id.dci()) };
        info!(
            "ring_ptr in request: {}",
            transfer_ring.as_mut_ptr() as usize
        );
        transfer_ring.push(setup);
        doorbell.notify_endpoint(HCP_ENDPOINT_ID.dci());
    }

    pub fn request_mouse(
        &mut self,
        endponint_id: EndpointID,
        mut doorbell: DCDoorbell,
        interface_num: u16,
    ) {
        let buf_len = BUF;
        let setup = SetupStage::zeroed()
            .with_parameter0_bm_request_type(0b1010001)
            .with_parameter0_b_request(1)
            .with_parameter0_w_value(0x0100)
            .with_parameter1_w_index(interface_num)
            .with_parameter1_w_length(buf_len as u16)
            .with_status_trb_transfer_length(8)
            .with_control_transfer_type(3)
            // .with_remain_interrupt_on_completion(true)
            .with_remain_immediate_data(true);

        let buf_ptr = self.buf.as_mut_ptr();
        let data = DataStage::zeroed()
            .with_buf_ptr_lo(buf_ptr as u32)
            .with_buf_ptr_hi(((buf_ptr as usize) >> 32) as u32)
            .with_status_trb_transfer_length(buf_len as u32)
            .with_status_td_size(0)
            .with_control_dir(true)
            .with_remain_interrupt_on_completion(true);

        let status = StatusStage::zeroed().with_control_direction(false);

        let transfer_ring = unsafe { self.ring_mut(endponint_id.dci()) };
        info!(
            "ring_ptr in request: {}",
            transfer_ring.as_mut_ptr() as usize
        );
        transfer_ring.push(setup);
        transfer_ring.push(data);
        transfer_ring.push(status);
        doorbell.notify_endpoint(HCP_ENDPOINT_ID.dci());
    }
}
