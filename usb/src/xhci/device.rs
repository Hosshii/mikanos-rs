use core::{
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};

use common::{info, Zeroed};

use super::{
    context::{DeviceContext, InputContext},
    descripter::DeviceDescripter,
    doorbell::DCDoorbell,
    error::{Error, Result},
    ring::TCRing,
    trb::{DataStage, SetupStage, StatusStage},
};

const DEFAULT_TRANSFER_RING_SIZE: usize = 32;
const DEFAULT_TRANSFER_RING_NUM: usize = 16;

#[derive(Debug)]
pub struct Device<
    const RING_SIZE: usize = DEFAULT_TRANSFER_RING_SIZE,
    const RING_NUM: usize = DEFAULT_TRANSFER_RING_NUM,
> {
    slot_id: SlotId,
    context: DeviceContext,
    input_context: InputContext,
    transfer_rings: [TCRing<RING_SIZE>; RING_NUM],
    descrpiter: MaybeUninit<DeviceDescripter>,
}

impl<const RING_SIZE: usize, const RING_NUM: usize> Device<RING_SIZE, RING_NUM> {
    fn new(id: SlotId) -> Self {
        Self {
            slot_id: id,
            context: DeviceContext::zeroed(),
            input_context: InputContext::zeroed(),
            transfer_rings: [(); RING_NUM].map(|_| TCRing::zeroed()),
            descrpiter: MaybeUninit::zeroed(),
        }
    }

    /// Caller must guarentee that this struct does not move.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut DeviceContext {
        &mut self.context as *mut DeviceContext
    }

    pub fn device_context(&self) -> &DeviceContext {
        &self.context
    }

    pub unsafe fn read_descripter(&self) -> &DeviceDescripter {
        unsafe { self.descrpiter.assume_init_ref() }
    }

    pub fn input_context_mut(&mut self) -> &mut InputContext {
        &mut self.input_context
    }

    pub fn rings_mut(&mut self) -> &mut [TCRing<RING_SIZE>; RING_NUM] {
        &mut self.transfer_rings
    }

    pub fn ring_mut(&mut self, dci: u8) -> &mut TCRing<RING_SIZE> {
        self.transfer_rings.index_mut(dci as usize - 1)
    }

    pub fn port_num(&self) -> u8 {
        self.context.slot_context.get_data_1_root_hub_port_number()
    }

    /// default controls pipe's transfer ring
    pub fn dcp_ring_mut(&mut self) -> &mut TCRing<RING_SIZE> {
        self.transfer_rings.index_mut(0)
    }

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

        let buf_ptr = self.descrpiter.as_mut_ptr();
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

        let transfer_ring = self.ring_mut(endponint_id.dci());
        info!(
            "ring_ptr in request: {}",
            transfer_ring.as_mut_ptr() as usize
        );
        transfer_ring.push(setup);
        doorbell.notify_endpoint(HCP_ENDPOINT_ID);
        transfer_ring.push(data);
        doorbell.notify_endpoint(HCP_ENDPOINT_ID);
        transfer_ring.push(status);
        doorbell.notify_endpoint(HCP_ENDPOINT_ID);
    }
}

pub type SlotId = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    Out = 0,
    In = 1,
}

impl From<bool> for Direction {
    fn from(value: bool) -> Self {
        if value {
            Direction::In
        } else {
            Direction::Out
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EndpointID(u8, Direction);
impl EndpointID {
    pub fn new(id: u8, dir: impl Into<Direction>) -> Self {
        Self(id, dir.into())
    }

    pub fn dci(self) -> u8 {
        self.0 * 2 + self.1 as u8
    }
}
pub const HCP_ENDPOINT_ID: EndpointID = EndpointID(0, Direction::In);

pub struct DeviceManager<
    const N: usize,
    const RING_SIZE: usize = DEFAULT_TRANSFER_RING_SIZE,
    const RING_NUM: usize = DEFAULT_TRANSFER_RING_NUM,
> {
    devices: [Option<Device<RING_SIZE, RING_NUM>>; N],
}

impl<const N: usize, const RING_SIZE: usize, const RING_NUM: usize>
    DeviceManager<N, RING_SIZE, RING_NUM>
{
    pub fn new() -> Self {
        Self {
            devices: [(); N].map(|_| None),
        }
    }

    pub fn alloc_device(&mut self, slot_id: SlotId) -> Result<&mut Device<RING_SIZE, RING_NUM>> {
        if N <= (slot_id as usize) {
            return Err(Error::device_mnager_out_of_range());
        }
        let device = Device::new(slot_id);

        self.devices[slot_id as usize] = Some(device);

        Ok(self.devices[slot_id as usize].as_mut().unwrap())
    }

    pub fn device(&self, slot_id: SlotId) -> Option<&Device<RING_SIZE, RING_NUM>> {
        self.devices.index(slot_id as usize).as_ref()
    }

    pub fn device_mut(&mut self, slot_id: SlotId) -> Option<&mut Device<RING_SIZE, RING_NUM>> {
        self.devices.index_mut(slot_id as usize).as_mut()
    }
}
