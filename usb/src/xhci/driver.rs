use crate::xhci::{
    doorbell::HCDoorbell,
    register_map::Doorbell,
    trb::{AddressDeviceCommand, EnableSlotCommand},
};

use super::{
    context::DeviceContext,
    device::{Device, DeviceManager},
    error::{Error, Result},
    port::{PortConfigPhase, PortWrapper, PortsConfigPhase},
    register_map::{
        CapabilityRegisters, DoorbellRegisters, OperationalRegisters, RuntimeRegisters,
    },
    ring::{EventRing, EventRingSegmentTableEntry, TCRing},
    trb::{CommandCompletionEvent, PortStatusChangeEvent, Trb, TrbRaw, Type},
};
use common::{debug, info, Zeroed};
use core::{
    marker::{PhantomData, PhantomPinned},
    ops::IndexMut,
    pin::Pin,
    ptr,
};

pub struct Uninitialized;
pub struct Initialized;
pub struct Running;

const DEFAULT_NUM_DEVICE_CONTEXT: usize = 64;
const DEFAULT_COMMAND_RING_BUF_SIZE: usize = 16;
const DEFAULT_EVENT_RING_SEGMENT_SIZE: usize = 16;
const DEFAULT_EVENT_RING_SEGMENTS_NUM: usize = 1;
const DEFAULT_EVENT_RING_SEGMENT_TABLE_SIZE: usize = DEFAULT_EVENT_RING_SEGMENTS_NUM;
const DEFAULT_DEVICE_MANAGER_SIZE: usize = 16;

pub struct Context<
    const DEV: usize = DEFAULT_NUM_DEVICE_CONTEXT,
    const CMD: usize = DEFAULT_COMMAND_RING_BUF_SIZE,
    const SEG_SIZE: usize = DEFAULT_EVENT_RING_SEGMENT_SIZE,
    const SEG_NUM: usize = DEFAULT_EVENT_RING_SEGMENTS_NUM,
    const TAB_SIZE: usize = DEFAULT_EVENT_RING_SEGMENT_TABLE_SIZE,
    const DEV_MNGR_SIZE: usize = DEFAULT_DEVICE_MANAGER_SIZE,
> {
    device_context_ptrs: [*mut DeviceContext; DEV],
    command_ring: TCRing<CMD>,
    event_ring_segments: [EventRing<SEG_SIZE>; SEG_NUM],
    event_ring_segment_table: [EventRingSegmentTableEntry; TAB_SIZE],
    device_manager: DeviceManager<DEV_MNGR_SIZE>,
    _phantom_pinned: PhantomPinned,
}

impl<
        const DEV: usize,
        const CMD: usize,
        const SEG_SIZE: usize,
        const SEG_NUM: usize,
        const TAB_SIZE: usize,
        const DEV_MNGR_SIZE: usize,
    > Context<DEV, CMD, SEG_SIZE, SEG_NUM, TAB_SIZE, DEV_MNGR_SIZE>
{
    pub fn primary_ring_mut(&mut self) -> &mut EventRing<SEG_SIZE> {
        self.event_ring_segments.index_mut(0)
    }

    pub fn issue_command(&mut self, cmd: impl Into<TrbRaw> + Type + Copy) {
        self.command_ring.push(cmd)
    }
}

impl<
        const DEV: usize,
        const CMD: usize,
        const SEG_SIZE: usize,
        const SEG_NUM: usize,
        const TAB_SIZE: usize,
        const DEV_MNGR_SIZE: usize,
    > Zeroed for Context<DEV, CMD, SEG_SIZE, SEG_NUM, TAB_SIZE, DEV_MNGR_SIZE>
{
    fn zeroed() -> Self {
        let device_context_ptrs = [ptr::null_mut(); DEV];
        let command_ring = TCRing::new();
        let event_ring_segments = [(); SEG_NUM].map(|_| EventRing::new());
        let event_ring_segment_table = [EventRingSegmentTableEntry::zeroed(); TAB_SIZE];
        let device_manager = DeviceManager::new();

        Self {
            device_context_ptrs,
            command_ring,
            event_ring_segments,
            event_ring_segment_table,
            device_manager,
            _phantom_pinned: PhantomPinned,
        }
    }
}

pub struct Controller<
    'a,
    State,
    const DEV: usize = DEFAULT_NUM_DEVICE_CONTEXT,
    const CMD: usize = DEFAULT_COMMAND_RING_BUF_SIZE,
    const SEG_SIZE: usize = DEFAULT_EVENT_RING_SEGMENT_SIZE,
    const SEG_NUM: usize = DEFAULT_EVENT_RING_SEGMENTS_NUM,
    const TAB_SIZE: usize = DEFAULT_EVENT_RING_SEGMENT_TABLE_SIZE,
    const DEVICE_MANAGER_SIZE: usize = DEFAULT_DEVICE_MANAGER_SIZE,
> {
    _phantomdata: PhantomData<State>,
    capability_registers: CapabilityRegisters<'static>,
    operational_registers: OperationalRegisters<'static>,
    runtime_registers: RuntimeRegisters<'static>,
    doorbell_registers: DoorbellRegisters<'static>,
    ports_config_phase: PortsConfigPhase,
    // 配列のポインタが動かないようにしたい
    // 今の所move以外は大丈夫
    cx: Pin<&'a mut Context<DEV, CMD, SEG_SIZE, SEG_NUM, TAB_SIZE, DEVICE_MANAGER_SIZE>>,
}

impl<
        'a,
        State,
        const DEV: usize,
        const CMD: usize,
        const SEG_SIZE: usize,
        const SEG_NUM: usize,
        const TAB_SIZE: usize,
    > Controller<'a, State, DEV, CMD, SEG_SIZE, SEG_NUM, TAB_SIZE>
{
}

impl<
        'a,
        const DEV: usize,
        const CMD: usize,
        const SEG_SIZE: usize,
        const SEG_NUM: usize,
        const TAB_SIZE: usize,
        const DEV_MNGR_SIZE: usize,
    > Controller<'a, Uninitialized, DEV, CMD, SEG_SIZE, SEG_NUM, TAB_SIZE, DEV_MNGR_SIZE>
{
    /// # Safety
    /// bar must be correct address.
    /// And cust call at cost once.
    pub unsafe fn new(
        bar: u64,
        cx: Pin<&'a mut Context<DEV, CMD, SEG_SIZE, SEG_NUM, TAB_SIZE, DEV_MNGR_SIZE>>,
    ) -> Self {
        debug!("bar: {:x?}", bar);
        let capability_registers = unsafe { CapabilityRegisters::new(bar as *const u8) };

        let op_off = capability_registers.cap_length().read().get_data();
        let op_base = bar + op_off as u64;
        let max_ports = capability_registers
            .hcs_paracm1()
            .read()
            .get_data_max_ports();
        let operational_registers =
            unsafe { OperationalRegisters::new(op_base as *mut u8, max_ports) };
        debug!("op_off: {:x?}, op_base: {:x?}", op_off, op_base);

        let rts_off = capability_registers.rts_offset().read().get_data_offset() << 5;
        let rts_base = bar + rts_off as u64;
        let runtime_registers = unsafe { RuntimeRegisters::new(rts_base as *mut u8) };
        debug!("rts_off: {:x?}, rts_base: {:x?}", rts_off, rts_base);

        let db_off = capability_registers.db_offset().read().get_data_offset() << 2;
        let db_base = bar + db_off as u64;
        let db_len = capability_registers
            .hcs_paracm1()
            .read()
            .get_data_max_device_slots();
        let doorbell_registers =
            unsafe { DoorbellRegisters::new(db_base as *mut Doorbell, db_len as usize) };
        debug!(
            "db_off: {:x?}, db_base: {:x?}, db_len: {:x?}",
            db_off, db_base, db_len
        );

        Self {
            _phantomdata: PhantomData,
            capability_registers,
            operational_registers,
            runtime_registers,
            doorbell_registers,
            ports_config_phase: PortsConfigPhase::default(),
            cx,
        }
    }

    pub fn initialize(
        mut self,
    ) -> Result<Controller<'a, Initialized, DEV, CMD, SEG_SIZE, SEG_NUM, TAB_SIZE, DEV_MNGR_SIZE>>
    {
        self.reset();
        self.set_device_context()?;
        self.register_command_ring();
        self.register_event_ring();
        self.config_interrupte();

        Ok(Controller {
            _phantomdata: PhantomData,
            capability_registers: self.capability_registers,
            operational_registers: self.operational_registers,
            runtime_registers: self.runtime_registers,
            doorbell_registers: self.doorbell_registers,
            ports_config_phase: self.ports_config_phase,
            cx: self.cx,
        })
    }

    fn reset(&mut self) {
        debug!("start xhci reset");
        debug!("wait host controller halted");
        while !self
            .operational_registers
            .usb_status()
            .read()
            .get_data_host_controller_halted()
        {
            debug!("wait hch");
        }

        // reset host controller
        debug!("reset host controller");
        let mut cmd = self.operational_registers.usb_command().read();
        cmd.set_data_host_controller_reset(true);
        self.operational_registers.usb_command_mut().write(cmd);
        while self
            .operational_registers
            .usb_command()
            .read()
            .get_data_host_controller_reset()
        {
            debug!("wait hcr");
        }

        debug!("wait controller not ready");
        while self
            .operational_registers
            .usb_status()
            .read()
            .get_data_controller_not_ready()
        {
            debug!("wait cnr");
        }
    }

    fn set_device_context(&mut self) -> Result<()> {
        let max_device_slots = self
            .capability_registers
            .hcs_paracm1()
            .read()
            .get_data_max_device_slots();
        debug!("max slots: {}", max_device_slots);

        if DEV < max_device_slots as usize {
            return Err(Error::lack_of_device_contexts());
        }

        let mut config = self.operational_registers.configure().read();
        config.set_data_max_device_slots_enabled(max_device_slots);
        self.operational_registers.configure_mut().write(config);

        let ptr = unsafe {
            self.cx
                .as_mut()
                .get_unchecked_mut()
                .device_context_ptrs
                .as_mut_ptr() as usize
        };
        debug!("dcbaap: {:0x}", ptr);

        let mut dcbaap = self
            .operational_registers
            .device_context_base_address_array_pointer()
            .read();
        dcbaap.set_ptr_lo_ptr_lo((ptr as u32) >> 6);
        dcbaap.set_ptr_hi((ptr >> 32) as u32);
        self.operational_registers
            .device_context_base_address_array_pointer_mut()
            .write(dcbaap);

        Ok(())
    }

    fn register_command_ring(&mut self) {
        debug!("start register commmand ring");
        let (cr_buf, cr_pcs) = unsafe {
            let cx = self.cx.as_mut().get_unchecked_mut();
            let cr_buf = cx.command_ring.as_mut_ptr();
            let cr_pcs = cx.command_ring.cycle_bit();
            (cr_buf, cr_pcs)
        };

        debug!(
            "command ring ptr {:p}, command ring cycle bit {}",
            cr_buf, cr_pcs
        );
        let ptr = cr_buf as usize;
        let ptr_lo = (ptr as u32) >> 6;
        let ptr_hi = (ptr >> 32) as u32;

        let mut crcr = self.operational_registers.command_ring_control_mut().read();

        crcr.set_command_ring_ptr_hi(ptr_hi);
        crcr.set_command_ring_ptr_lo_data(ptr_lo);
        crcr.set_command_ring_ptr_lo_ring_cycle_state(cr_pcs);
        self.operational_registers
            .command_ring_control_mut()
            .write(crcr);
    }

    fn register_event_ring(&mut self) {
        debug!("start register event ring");
        let mut size = 0;
        unsafe {
            let cx = self.cx.as_mut().get_unchecked_mut();

            for (entry, segment) in cx
                .event_ring_segment_table
                .iter_mut()
                .zip(cx.event_ring_segments.iter_mut())
            {
                let addr = segment.as_mut_ptr();
                entry.set_ring_segment_base_address_data((addr as u64) >> 6);

                let buf_size = SEG_SIZE;
                entry.set_ring_segment_size_data(buf_size as u16);

                size += 1;
            }
        };

        let primary = self.runtime_registers.get_primary_interrupter_mut();

        debug!("write erstsz: {}", size);
        let mut erstsz = primary.event_ring_segment_table_size().read();
        erstsz.set_data_event_ring_segment_table_size(size);
        primary.event_ring_segment_table_size_mut().write(erstsz);
        debug!(
            "read erstsz: {}",
            primary
                .event_ring_segment_table_size()
                .read()
                .get_data_event_ring_segment_table_size()
        );

        let mut erdp = primary.event_ring_dequeue_pointer().read();
        let seg0ptr = self.cx.event_ring_segments[0].as_ptr();
        debug!("write erdp: {}", (seg0ptr as u64) >> 4);
        erdp.set_data_ptr((seg0ptr as u64) >> 4);
        primary.event_ring_dequeue_pointer_mut().write(erdp);
        debug!(
            "read erdp: {}",
            primary.event_ring_dequeue_pointer().read().get_data_ptr()
        );

        let mut erstba = primary.event_ring_segment_table_base_address().read();
        let tb0addr = self.cx.event_ring_segment_table.as_ptr();
        debug!("write erstba: {:p}", tb0addr);
        erstba.set_data_ptr((tb0addr as u64) >> 6);
        primary
            .event_ring_segment_table_base_address_mut()
            .write(erstba);
    }

    fn config_interrupte(&mut self) {
        let primary = self.runtime_registers.get_primary_interrupter_mut();

        let mut imod = primary.interrupt_moderation().read();
        imod.set_data_interrupt_modification_interval(4000);
        primary.interrupt_moderation_mut().write(imod);

        let mut iman = primary.interrupt_management().read();
        iman.set_data_interrupt_pending(true);
        iman.set_data_interrupt_enable(true);
        debug!("iman: {:?}", iman);
        primary.interrupt_management_mut().write(iman);

        let mut cmd = self.operational_registers.usb_command().read();
        cmd.set_data_interrupter_enable(true);
        self.operational_registers.usb_command_mut().write(cmd);
    }
}

impl<
        'a,
        const DEV: usize,
        const CMD: usize,
        const SEG_SIZE: usize,
        const SEG_NUM: usize,
        const TAB_SIZE: usize,
        const DEV_MNGR_SIZE: usize,
    > Controller<'a, Initialized, DEV, CMD, SEG_SIZE, SEG_NUM, TAB_SIZE, DEV_MNGR_SIZE>
{
    pub fn run(
        mut self,
    ) -> Controller<'a, Running, DEV, CMD, SEG_SIZE, SEG_NUM, TAB_SIZE, DEV_MNGR_SIZE> {
        info!("run xhci");
        let mut cmd = self.operational_registers.usb_command().read();
        cmd.set_data_run_stop(true);
        self.operational_registers.usb_command_mut().write(cmd);

        while self
            .operational_registers
            .usb_status()
            .read()
            .get_data_host_controller_halted()
        {
            debug!("waiting")
        }

        Controller {
            _phantomdata: PhantomData,
            capability_registers: self.capability_registers,
            operational_registers: self.operational_registers,
            runtime_registers: self.runtime_registers,
            doorbell_registers: self.doorbell_registers,
            ports_config_phase: self.ports_config_phase,
            cx: self.cx,
        }
    }
}

impl<
        'a,
        const DEV: usize,
        const CMD: usize,
        const SEG_SIZE: usize,
        const SEG_NUM: usize,
        const TAB_SIZE: usize,
        const DEV_MNGR_SIZE: usize,
    > Controller<'a, Running, DEV, CMD, SEG_SIZE, SEG_NUM, TAB_SIZE, DEV_MNGR_SIZE>
{
    pub fn ports_mut(&mut self) -> impl Iterator<Item = PortWrapper<'_, 'static, '_>> {
        ports_mut(
            &mut self.operational_registers,
            &mut self.ports_config_phase,
        )
    }

    pub fn processing_port_num(&self) -> Option<u8> {
        self.ports_config_phase.processing_port()
    }

    pub fn devices_mut(&mut self) -> impl Iterator<Item = Pin<&mut Device>> {
        unsafe {
            let devices = self
                .cx
                .as_mut()
                .get_unchecked_mut()
                .device_manager
                .devices_mut();
            devices.map(|v| Pin::new_unchecked(v))
        }
    }

    pub fn process_primary_event(&mut self) -> Result<()> {
        let primary = self.runtime_registers.get_primary_interrupter_mut();

        let Some(event) = unsafe { self.cx.as_mut().get_unchecked_mut() }
            .primary_ring_mut()
            .pop::<Trb>(primary)
        else {
            if !self.ports_config_phase.is_resetting_port_exist() {
                if let Some(port_num) = self.ports_config_phase.waiting_reset_port() {
                    self.ports_config_phase.set_processing_port(port_num)?;
                    let phase = self.ports_config_phase.phase_mut(port_num);
                    let mut port = port(&mut self.operational_registers, port_num, phase);
                    port.configure()?;
                }
            }
            return Ok(());
        };

        info!("process event");
        match event {
            Trb::Normal => todo!(),
            Trb::SetupStage => todo!(),
            Trb::DataStage => todo!(),
            Trb::StatusStage => todo!(),
            Trb::Link(_) => todo!(),
            Trb::NoOp => todo!(),
            Trb::EnableSlotCommand(_) => todo!(),
            Trb::AddressDeviceCommand(_) => todo!(),
            Trb::ConfigureEndpoint => todo!(),
            Trb::NoOpCommand => todo!(),
            Trb::TransferEvent => todo!(),
            Trb::CommandCompletionEvent(e) => self.process_command_completion_event(e),
            Trb::PortStatusChangeEvent(e) => self.process_port_status_change_event(e),
            Trb::Unknown(_) => {
                debug!("process event. unknown trb: {:?}", event);
                Ok(())
            }
        }
    }

    fn process_command_completion_event(&mut self, event: CommandCompletionEvent) -> Result<()> {
        debug!("process command completion event");
        debug!(
            "completion e: {:?}, {:?}",
            event.get_status_completion_code(),
            event
        );
        let issuer = unsafe { event.issuer() };
        let slot_id = event.get_control_slot_id();
        debug!("slot_id: {}, issuer: {:?}", slot_id, issuer);

        if !event.is_success() {
            return Err(Error::command_not_success(
                event.get_status_completion_code(),
                issuer,
            ));
        }

        match issuer {
            Trb::EnableSlotCommand(_) => {
                fn determin_max_packet_size(speed: u8) -> u16 {
                    match speed {
                        4 => 512,
                        3 => 64,
                        2 | 1 => 8,
                        _ => panic!("unknown speed {}", speed),
                    }
                }
                let processing_port_num = self
                    .processing_port_num()
                    .ok_or(Error::empty_processing_port())?;

                // may not move cx
                unsafe {
                    // alloc device context.
                    let cx = self.cx.as_mut().get_unchecked_mut();
                    let device = cx.device_manager.alloc_device(slot_id)?;
                    cx.device_context_ptrs[slot_id as usize] = device.as_mut_ptr();

                    let transfer_ring = device.dcp_ring_mut();
                    let ring_ptr = transfer_ring.as_mut_ptr();
                    info!("ring_ptr: {}", ring_ptr as u64);
                    let cycle_bit = transfer_ring.cycle_bit();

                    let input_context = device.input_context_mut();
                    input_context.enable_endpoint(1);
                    input_context.enable_slot_context();

                    let phase = self.ports_config_phase.phase_mut(processing_port_num);
                    let mut port =
                        port(&mut self.operational_registers, processing_port_num, phase);

                    input_context.init_slot_cx(&port);

                    input_context.init_ep0_endpoint(
                        ring_ptr.cast(),
                        cycle_bit,
                        determin_max_packet_size(port.speed()),
                    );
                    debug!("port speed: {}", port.speed());

                    port.set_phase(PortConfigPhase::AddressingDevice);
                    let cmd =
                        AddressDeviceCommand::new(input_context as *mut _ as *mut u8, slot_id);
                    cx.command_ring.push(cmd);
                    self.doorbell_registers
                        .host_controller_mut()
                        .notify_host_controller();
                    debug!("address device command");
                }
            }
            Trb::AddressDeviceCommand(_cmd) => {
                let processing_port_num = self
                    .processing_port_num()
                    .ok_or(Error::empty_processing_port())?;

                unsafe {
                    let cx = self.cx.as_mut().get_unchecked_mut();

                    let dev = cx
                        .device_manager
                        .device_mut(slot_id)
                        .ok_or(Error::invalid_slot_id())?;
                    let port_id = dev.port_num();
                    if processing_port_num != port_id {
                        return Err(Error::invalid_port_id());
                    }

                    info!(
                        "device address: {}",
                        dev.device_context()
                            .slot_context
                            .get_data_3_usb_device_address()
                    );

                    self.ports_config_phase.clear_processing_port()?;

                    let phase = self.ports_config_phase.phase(port_id);
                    if phase != PortConfigPhase::AddressingDevice {
                        return Err(Error::invalid_phase(
                            PortConfigPhase::AddressingDevice,
                            phase,
                        ));
                    }
                    self.ports_config_phase
                        .set_phase(port_id, PortConfigPhase::InitializingDevice);
                }
            }
            x => debug!("issuer {:?}", x),
        }
        Ok(())
    }

    fn process_port_status_change_event(&mut self, event: PortStatusChangeEvent) -> Result<()> {
        debug!("process psce");
        let port_num = event.get_parameter0_port_id();
        let phase = *self
            .ports_config_phase
            .phases_mut()
            .index_mut(port_num as usize);

        match phase {
            PortConfigPhase::ResettingPort => {
                fn enable_slot<const N: usize>(
                    port: &mut PortWrapper,
                    cmd_ring: &mut TCRing<N>,
                    hc_doorbell: &mut HCDoorbell,
                ) -> Result<()> {
                    debug!(
                        "enable slot: is enabled: {}. is_reset_changed: {}",
                        port.is_enabled(),
                        port.is_port_reset_changed()
                    );
                    if !port.is_enabled() {
                        return Err(Error::port_disabled());
                    }
                    if !port.is_connected_status_changed() {
                        return Err(Error::port_reset_not_finished());
                    }

                    port.set_phase(PortConfigPhase::EnablingSlot);
                    let cmd = EnableSlotCommand::default();
                    cmd_ring.push(cmd);
                    hc_doorbell.notify_host_controller();

                    Ok(())
                }

                // self.enable_slot(port);
                let mut port = port(
                    &mut self.operational_registers,
                    port_num,
                    self.ports_config_phase
                        .phases_mut()
                        .index_mut(port_num as usize),
                );
                unsafe {
                    let cmd_ring = &mut self.cx.as_mut().get_unchecked_mut().command_ring;
                    let mut hc_doorbell = self.doorbell_registers.host_controller_mut();
                    enable_slot(&mut port, cmd_ring, &mut hc_doorbell)
                }
            }
            x => {
                debug!("process psce. port phase: {:?}", x);
                Ok(())
            }
        }
    }
}

fn port<'a, 'b, 'c>(
    op: &'a mut OperationalRegisters<'b>,
    idx: u8,
    phase: &'c mut PortConfigPhase,
) -> PortWrapper<'a, 'b, 'c> {
    PortWrapper::new(
        op.port_registers_mut().index_mut((idx - 1) as usize),
        idx,
        phase,
    )
}

fn ports_mut<'a, 'b, 'c>(
    op: &'a mut OperationalRegisters<'b>,
    phases: &'c mut PortsConfigPhase,
) -> impl Iterator<Item = PortWrapper<'a, 'b, 'c>> {
    op.port_registers_mut()
        .iter_mut()
        .enumerate()
        .zip(phases.phases_mut().iter_mut().skip(1))
        .map(|((idx, v), phase)| PortWrapper::new(v, idx as u8 + 1, phase))
}
