use core::{
    ops::IndexMut,
    pin::{pin, Pin},
};

use common::{info, map::FixedMap, Zeroed};

use crate::{
    usbd::descriptor::{
        ConfigurationDescriptor, DeviceDescriptor, EndpointDescriptor, HIDDescriptor,
        InterfaceDescriptor, PackedSize,
    },
    xhci::{
        context::{EndpointContxt, InputContext},
        device::SlotId,
        driver::{Controller, Running, Uninitialized},
        port::PortConfigPhase,
        trb::{ConfigureEndpointCommand, Trb, TrbType, Type},
    },
};

use super::{
    descriptor::Descriptor,
    device::Device,
    endpoint::HCP_ENDPOINT_ID,
    error::{Error, Result},
};

const DEVICE_NUM: usize = 16;

pub struct Driver<'a> {
    xhcid: Controller<'a, Running>,
    devices: FixedMap<SlotId, (InputContext, bool)>,
}

impl<'a> Driver<'a> {
    pub fn new(xhcid: Controller<'a, Uninitialized>) -> Result<Self> {
        let mut xhcid = xhcid.initialize()?.run();

        for mut port in xhcid.ports_mut() {
            if port.is_connected() {
                port.set_phase(PortConfigPhase::WaitingAddressed)
            }
        }

        Ok(Self {
            xhcid,
            devices: FixedMap::new(),
        })
    }

    pub fn process(&mut self) -> Result<()> {
        let event = self.xhcid.process_primary_event()?;

        Ok(())
    }

    pub fn configure_device(&mut self) -> Result<Option<SlotId>> {
        let slot_id = self.xhcid.devices_mut().find(|d| {
            let slot_id = d.slot_id();
            self.devices.get(&slot_id).map(|v| !v.1).unwrap_or(true)
        });

        let Some(slot_id) = slot_id else {
            return Ok(None);
        };

        let slot_id = slot_id.slot_id();
        let d = self.get_device_descriptor(slot_id)?;
        let Some(device_descriptor) = d else {
            return Ok(None);
        };
        info!("{:?}", device_descriptor);

        let d = self.get_config_descriptor(slot_id)?;
        let Some(configuration_descriptor) = d else {
            return Ok(None);
        };
        info!("{:?}", configuration_descriptor);

        let descriptors = configuration_descriptor.iter().filter_map(|v| {
            v.iter().find_map(|v| match v {
                Descriptor::Endpoint(e) => Some(e),
                _ => None,
            })
        });

        self.configure_endpoint(descriptors)?;

        self.set_boot_mode()?;

        Ok(Some(slot_id))
    }

    fn size(d: &Descriptor) -> usize {
        match d {
            Descriptor::Device(_) => DeviceDescriptor::SIZE,
            Descriptor::Configuration(_) => ConfigurationDescriptor::SIZE,
            Descriptor::Interface(_) => InterfaceDescriptor::SIZE,
            Descriptor::Endpoint(_) => EndpointDescriptor::SIZE,
            Descriptor::HIDDescriptor(_) => HIDDescriptor::SIZE,
        }
    }

    fn get_device_descriptor(&mut self, slot_id: SlotId) -> Result<Option<DeviceDescriptor>> {
        let mut buf = [0; 32];

        let dev = unsafe { self.xhcid.cx.as_mut().get_unchecked_mut() }
            .devices_mut()
            .find(|d| d.slot_id() == slot_id)
            .unwrap();

        let mut dev = Device::<32>::new(&mut buf, dev);

        let slot_id = dev.slot_id();
        let doorbell = self.xhcid.doorbell_registers.slot(slot_id);
        dev.request_device_descripter(HCP_ENDPOINT_ID, doorbell, 0);

        let a = loop {
            if let Some(v) = self.xhcid.process_primary_event()? {
                break v;
            }
        };

        let Trb::TransferEvent(e) = a else {
            return Err(Error::unexpected_trb(TrbType::TransferEvent, a));
        };

        info!("{:?}", e.get_status_completion_code());
        if !e.get_status_completion_code().is_success() {
            // todo!("{:?}", e.get_status_completion_code())
        }

        let Descriptor::Device(v) = Descriptor::try_from(buf.as_slice())? else {
            return Err(Error::unexpected_descriptor());
        };

        Ok(Some(v))
    }

    fn get_config_descriptor(
        &mut self,
        slot_id: SlotId,
    ) -> Result<Option<[Option<Descriptor>; 10]>> {
        let mut buf = [0; 256];
        let dev = unsafe { self.xhcid.cx.as_mut().get_unchecked_mut() }
            .devices_mut()
            .find(|d| d.slot_id() == slot_id)
            .unwrap();

        let mut dev = Device::new(&mut buf, dev);
        let slot_id = dev.slot_id();
        let doorbell = self.xhcid.doorbell_registers.slot(slot_id);
        dev.request_configuration_descriptor(HCP_ENDPOINT_ID, doorbell, 0);

        let a = loop {
            if let Some(v) = self.xhcid.process_primary_event()? {
                break v;
            }
        };

        let Trb::TransferEvent(e) = a else {
            return Err(Error::unexpected_trb(TrbType::TransferEvent, a));
        };

        if !e.get_status_completion_code().is_success() {
            info!("{:?}", e.get_status_completion_code());
        }

        let len = buf.len() - e.get_status_trb_transfer_length() as usize;
        let mut tmp: [Option<Descriptor>; 10] = [(); 10].map(|_| None);

        let mut read_bytes = 0;
        let mut idx = 0;

        let mut buf = buf.as_slice();
        while read_bytes < len {
            let d = Descriptor::try_from(buf)?;
            let size = Self::size(&d);
            tmp[idx] = Some(d);
            idx += 1;
            read_bytes += size;
            buf = &buf[size..];
        }

        Ok(Some(tmp))
    }

    fn configure_endpoint<'b>(
        &mut self,
        descriptors: impl Iterator<Item = &'b EndpointDescriptor>,
    ) -> Result<()> {
        fn configute_ep_cx(cx: &mut EndpointContxt, desc: &EndpointDescriptor) {
            let ty = match (
                desc.get_endpoint_address_dir_in(),
                desc.get_attributes_transfer_type(),
            ) {
                (false, 1) => 1,
                (false, 2) => 2,
                (false, 3) => 3,
                (_, 0) => 4,
                (true, 1) => 5,
                (true, 2) => 6,
                (true, 3) => 7,
                _ => unimplemented!(),
            };

            cx.set_data_1_ep_type(ty);
            cx.set_data_1_max_packet_size(desc.get_max_packet_size());
            cx.set_data_1_max_burst_size(0);
            cx.set_data_2_dequeue_cycle_state(true);
            cx.set_data_0_interval(desc.get_interval());
            cx.set_data_0_max_primary_streams(0);
            cx.set_data_0_mult(0);
            cx.set_data_1_error_count(3);
        }

        let mut dev_iter = unsafe { self.xhcid.cx.as_mut().get_unchecked_mut() }.devices_mut();

        let Some(dev) = dev_iter.find(|v| !self.devices.contains_key(&v.slot_id())) else {
            return Ok(());
        };

        let slot_id = dev.slot_id();

        let mut input_context = InputContext::zeroed();
        input_context.slot = dev.context.slot_context;
        input_context.enable_slot_context();
        input_context.slot.set_data_0_context_entries(31);
        for desc in descriptors {
            let dci =
                desc.get_endpoint_address_number() * 2 + desc.get_endpoint_address_dir_in() as u8;
            input_context.enable_endpoint(dci);

            configute_ep_cx(input_context.ep_contexts.index_mut(dci as usize), desc);
        }
        self.devices
            .insert(slot_id, (input_context, false))
            .unwrap();
        drop(dev_iter);

        let ptr = &self.devices.get(&slot_id).unwrap().0 as *const InputContext as usize;
        let cycle_bit = self.xhcid.cx.command_ring().cycle_bit();
        let trb = ConfigureEndpointCommand::zeroed()
            .with_parameter0_input_context_ptr_lo((ptr as u32) >> 4)
            .with_input_context_ptr_hi((ptr >> 32) as u32)
            .with_control_slot_id(slot_id)
            .with_remain_cycle_bit(cycle_bit);

        unsafe { self.xhcid.cx.as_mut().get_unchecked_mut() }.issue_command(trb);
        self.xhcid
            .doorbell_registers
            .host_controller_mut()
            .notify_host_controller();

        let e = loop {
            if let Some(x) = self.xhcid.process_primary_event()? {
                if let Trb::CommandCompletionEvent(e) = x {
                    break e;
                } else {
                    info!("{:?}", x);
                }
            }
        };

        if !e.is_success() {
            info!("{:?}", e.get_status_completion_code());
        }

        info!("{:?}", e);

        Ok(())
    }

    fn set_boot_mode(&mut self) -> Result<()> {
        let mut dev_iter = unsafe { self.xhcid.cx.as_mut().get_unchecked_mut() }.devices_mut();

        let Some(dev) = dev_iter.find(|v| !self.devices.contains_key(&v.slot_id())) else {
            return Ok(());
        };

        let slot_id = dev.slot_id();
        let doorbell = self.xhcid.doorbell_registers.slot(slot_id);

        let mut dev = Device::new(&mut [0u8; 0], dev);
        dev.request_boot_protocol_descriptor(HCP_ENDPOINT_ID, doorbell);

        drop(dev_iter);

        let a = loop {
            if let Some(v) = self.xhcid.process_primary_event()? {
                break v;
            }
        };

        let Trb::TransferEvent(e) = a else {
            return Err(Error::unexpected_trb(TrbType::TransferEvent, a));
        };

        info!("{:?}", e);

        self.devices.get_mut(&slot_id).unwrap().1 = true;

        Ok(())
    }

    pub fn get_mouse(&mut self, slot_id: SlotId) -> Result<[u8; 3]> {
        let mut buf = [0; 3];

        let dev = unsafe { self.xhcid.cx.as_mut().get_unchecked_mut() }
            .devices_mut()
            .find(|d| d.slot_id() == slot_id)
            .unwrap();

        let mut dev = Device::new(&mut buf, dev);

        let slot_id = dev.slot_id();
        let doorbell = self.xhcid.doorbell_registers.slot(slot_id);
        dev.request_mouse(HCP_ENDPOINT_ID, doorbell, 0);

        let a = loop {
            if let Some(v) = self.xhcid.process_primary_event()? {
                break v;
            }
        };

        let Trb::TransferEvent(e) = a else {
            return Err(Error::unexpected_trb(TrbType::TransferEvent, a));
        };

        // info!("{:?}", e.get_status_completion_code());
        if !e.get_status_completion_code().is_success() {
            // todo!("{:?}", e.get_status_completion_code())
        }

        Ok(buf)
    }
}
