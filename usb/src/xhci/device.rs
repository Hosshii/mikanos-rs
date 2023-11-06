use core::{
    marker::PhantomPinned,
    ops::{Index, IndexMut},
};

use common::Zeroed;

use super::{
    context::{DeviceContext, InputContext},
    error::{Error, Result},
    ring::TCRing,
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
    _phantom_pinned: PhantomPinned,
}

impl<const RING_SIZE: usize, const RING_NUM: usize> Device<RING_SIZE, RING_NUM> {
    fn new(id: SlotId) -> Self {
        Self {
            slot_id: id,
            context: DeviceContext::zeroed(),
            input_context: InputContext::zeroed(),
            transfer_rings: [(); RING_NUM].map(|_| TCRing::zeroed()),
            _phantom_pinned: PhantomPinned,
        }
    }

    /// Caller must guarentee that this struct does not move.
    pub(super) unsafe fn as_mut_ptr(&mut self) -> *mut DeviceContext {
        &mut self.context as *mut DeviceContext
    }

    pub(super) fn device_context(&self) -> &DeviceContext {
        &self.context
    }

    pub fn input_context_mut(&mut self) -> &mut InputContext {
        &mut self.input_context
    }

    pub(super) fn rings_mut(&mut self) -> &mut [TCRing<RING_SIZE>; RING_NUM] {
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
}

pub type SlotId = u8;

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

    pub fn devices_mut(&mut self) -> impl Iterator<Item = &mut Device<RING_SIZE, RING_NUM>> {
        self.devices.iter_mut().flatten()
    }
}
