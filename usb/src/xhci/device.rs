use common::Zeroed;

use super::{
    context::{DeviceContext, InputContext},
    error::{Error, Result},
    ring::TCRing,
};

const DEFAULT_TRANSFER_RING_SIZE: usize = 32;
const DEFAULT_TRANSFER_RING_NUM: usize = 16;

#[derive(Debug, PartialEq, Eq)]
pub struct Device<
    const RING_SIZE: usize = DEFAULT_TRANSFER_RING_SIZE,
    const RING_NUM: usize = DEFAULT_TRANSFER_RING_NUM,
> {
    slot_id: SlotId,
    context: DeviceContext,
    input_context: InputContext,
    transfer_rings: [TCRing<RING_SIZE>; RING_NUM],
}

impl<const N: usize, const RINGS: usize> Device<N, RINGS> {
    fn new(id: SlotId) -> Self {
        Self {
            slot_id: id,
            context: DeviceContext::zeroed(),
            input_context: InputContext::zeroed(),
            transfer_rings: [(); RINGS].map(|_| TCRing::zeroed()),
        }
    }

    /// Caller must guarentee that this struct does not move.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut DeviceContext {
        &mut self.context as *mut DeviceContext
    }

    pub fn input_context_mut(&mut self) -> &mut InputContext {
        &mut self.input_context
    }

    pub fn rings_mut(&mut self) -> &mut [TCRing<N>; RINGS] {
        &mut self.transfer_rings
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
}
