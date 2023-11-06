use core::mem::MaybeUninit;

use super::{descriptor::DeviceDescripter, endpoint::EndpointID};
use crate::{
    usbd::endpoint::HCP_ENDPOINT_ID,
    xhci::{
        device::Device as XHCIDevice,
        doorbell::DCDoorbell,
        trb::{DataStage, SetupStage, StatusStage},
    },
};
use common::{info, Zeroed};

struct Device {
    descriptor: MaybeUninit<DeviceDescripter>,
    device: XHCIDevice,
}

impl Device {
    pub fn request_device_descripter(
        &mut self,
        endponint_id: EndpointID,
        mut doorbell: DCDoorbell,
    ) {
        let buf_len = core::mem::size_of::<DeviceDescripter>();
        let setup = SetupStage::zeroed()
            .with_parameter0_bm_request_type(0b10000000)
            .with_parameter0_b_ruquest(6)
            .with_parameter0_w_value(0x0100)
            .with_parameter1_w_index(0)
            .with_parameter1_w_length(buf_len as u16)
            .with_status_trb_transfer_length(8)
            .with_control_transfer_type(3)
            .with_remain_interrupt_on_completion(true)
            .with_remain_immediate_data(true);

        let buf_ptr = self.descriptor.as_mut_ptr();
        let data = DataStage::zeroed()
            .with_buf_ptr_lo(buf_ptr as u32)
            .with_buf_ptr_hi(((buf_ptr as usize) >> 32) as u32)
            .with_status_trb_transfer_length(buf_len as u32)
            .with_status_td_size(0)
            .with_control_dir(true)
            .with_remain_interrupt_on_completion(true);

        let status = StatusStage::zeroed()
            .with_control_direction(false)
            .with_remain_interrupt_on_completion(true);

        let transfer_ring = self.device.ring_mut(endponint_id.dci());
        info!(
            "ring_ptr in request: {}",
            transfer_ring.as_mut_ptr() as usize
        );
        transfer_ring.push(setup);
        doorbell.notify_endpoint(HCP_ENDPOINT_ID.dci());
        transfer_ring.push(data);
        doorbell.notify_endpoint(HCP_ENDPOINT_ID.dci());
        transfer_ring.push(status);
        doorbell.notify_endpoint(HCP_ENDPOINT_ID.dci());
    }
}
