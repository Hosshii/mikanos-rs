use super::{
    context::DeviceContext,
    error::{Error, Result},
    register_map::{
        CapabilityRegisters, Doorbell, DoorbellRegisters, OperationalRegisters, RuntimeRegisters,
    },
    ring::{EventRing, EventRingSegmentTableEntry, TCRing},
    trb::{CommandCompletionEvent, Trb, TrbRaw},
};
use common::{debug, info};
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

pub struct Context<
    const DEV: usize = DEFAULT_NUM_DEVICE_CONTEXT,
    const CMD: usize = DEFAULT_COMMAND_RING_BUF_SIZE,
    const SEG_SIZE: usize = DEFAULT_EVENT_RING_SEGMENT_SIZE,
    const SEG_NUM: usize = DEFAULT_EVENT_RING_SEGMENTS_NUM,
    const TAB_SIZE: usize = DEFAULT_EVENT_RING_SEGMENT_TABLE_SIZE,
> {
    device_context_ptrs: [*mut DeviceContext; DEV],
    command_ring: TCRing<CMD>,
    event_ring_segments: [EventRing<SEG_SIZE>; SEG_NUM],
    event_ring_segment_table: [EventRingSegmentTableEntry; TAB_SIZE],
    _phantom_pinned: PhantomPinned,
}

impl<
        const DEV: usize,
        const CMD: usize,
        const SEG_SIZE: usize,
        const SEG_NUM: usize,
        const TAB_SIZE: usize,
    > Context<DEV, CMD, SEG_SIZE, SEG_NUM, TAB_SIZE>
{
    pub fn zeroed() -> Self {
        let device_context_ptrs = [ptr::null_mut(); DEV];
        let command_ring = TCRing::new();
        let event_ring_segments = [(); SEG_NUM].map(|_| EventRing::new());
        let event_ring_segment_table = [EventRingSegmentTableEntry::zeroed(); TAB_SIZE];

        Self {
            device_context_ptrs,
            command_ring,
            event_ring_segments,
            event_ring_segment_table,
            _phantom_pinned: PhantomPinned,
        }
    }

    pub fn primary_ring_mut(&mut self) -> &mut EventRing<SEG_SIZE> {
        self.event_ring_segments.index_mut(0)
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
> {
    _phantomdata: PhantomData<State>,
    capability_registers: CapabilityRegisters<'static>,
    operational_registers: OperationalRegisters<'static>,
    runtime_registers: RuntimeRegisters<'static>,
    doorbell_registers: DoorbellRegisters<'static>,
    // 配列のポインタが動かないようにしたい
    // 今の所move以外は大丈夫
    cx: Pin<&'a mut Context<DEV, CMD, SEG_SIZE, SEG_NUM, TAB_SIZE>>,
}

impl<
        'a,
        const DEV: usize,
        const CMD: usize,
        const SEG_SIZE: usize,
        const SEG_NUM: usize,
        const TAB_SIZE: usize,
    > Controller<'a, Uninitialized, DEV, CMD, SEG_SIZE, SEG_NUM, TAB_SIZE>
{
    /// # Safety
    /// bar must be correct address.
    /// And cust call at cost once.
    pub unsafe fn new(
        bar: u64,
        cx: Pin<&'a mut Context<DEV, CMD, SEG_SIZE, SEG_NUM, TAB_SIZE>>,
    ) -> Self {
        let capability_registers = unsafe { CapabilityRegisters::new(bar as *const u8) };

        let op_off = capability_registers.cap_length().read().get_data();
        let op_base = bar + op_off as u64;
        let operational_registers = unsafe { OperationalRegisters::new(op_base as *mut u8) };

        let rts_off = capability_registers.rts_offset().read().get_data_offset();
        let rts_base = bar + rts_off as u64;
        let runtime_registers = unsafe { RuntimeRegisters::new(rts_base as *mut u8) };

        let dboff = capability_registers.db_offset().read().get_data_offset();
        let db_base = bar + dboff as u64;
        let db_len = capability_registers
            .hcs_paracm1()
            .read()
            .get_data_max_device_slots();
        let doorbell_registers =
            unsafe { DoorbellRegisters::new(db_base as *mut u8, db_len as usize) };

        Self {
            _phantomdata: PhantomData,
            capability_registers,
            operational_registers,
            runtime_registers,
            doorbell_registers,
            cx,
        }
    }

    pub fn initialize(
        mut self,
    ) -> Result<Controller<'a, Initialized, DEV, CMD, SEG_SIZE, SEG_NUM, TAB_SIZE>> {
        self.reset();
        self.set_device_context()?;
        self.register_command_ring();
        self.register_event_ring();

        Ok(Controller {
            _phantomdata: PhantomData,
            capability_registers: self.capability_registers,
            operational_registers: self.operational_registers,
            runtime_registers: self.runtime_registers,
            doorbell_registers: self.doorbell_registers,
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
            return Err(Error::lack_of_max_slots());
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

        debug!("command ring ptr {:p}", cr_buf);
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
        let size = unsafe {
            let cx = self.cx.as_mut().get_unchecked_mut();

            let mut count = 0;
            for (entry, segment) in cx
                .event_ring_segment_table
                .iter_mut()
                .zip(cx.event_ring_segments.iter_mut())
            {
                let addr = segment.as_mut_ptr();
                entry.set_ring_segment_base_address_data((addr as u64) >> 6);
                count += 1;
            }

            count
        };

        let primary = self
            .runtime_registers
            .get_interrupter_register_sets_mut()
            .index_mut(0);

        debug!("write erstsz: {}", size);
        let mut erstsz = primary.event_ring_segment_table_size().read();
        erstsz.set_data_event_ring_segment_table_size(size);
        primary.event_ring_segment_table_size_mut().write(erstsz);

        let mut erdp = primary.event_ring_dequeue_pointer().read();
        let seg0ptr = self.cx.event_ring_segments[0].as_ptr();
        debug!("write erdp: {:p}", seg0ptr);
        erdp.set_data_ptr((seg0ptr as u64) >> 4);
        primary.event_ring_dequeue_pointer_mut().write(erdp);

        let mut erstba = primary.event_ring_segment_table_base_address().read();
        let tb0addr = self.cx.event_ring_segment_table.as_ptr();
        debug!("write erstba: {:p}", tb0addr);
        erstba.set_data_ptr((tb0addr as u64) >> 6);
        primary
            .event_ring_segment_table_base_address_mut()
            .write(erstba);
    }
}

impl<
        'a,
        const DEV: usize,
        const CMD: usize,
        const SEG_SIZE: usize,
        const SEG_NUM: usize,
        const TAB_SIZE: usize,
    > Controller<'a, Initialized, DEV, CMD, SEG_SIZE, SEG_NUM, TAB_SIZE>
{
    pub fn run(mut self) -> Controller<'a, Running, DEV, CMD, SEG_SIZE, SEG_NUM, TAB_SIZE> {
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
    > Controller<'a, Running, DEV, CMD, SEG_SIZE, SEG_NUM, TAB_SIZE>
{
    pub fn issue_command(&mut self, cmd: impl Into<TrbRaw>) {
        unsafe { self.cx.as_mut().get_unchecked_mut().command_ring.push(cmd) }
    }

    pub fn notify_command(&mut self) {
        let cmd = Doorbell::default();
        self.doorbell_registers[0].write(cmd);
    }

    pub fn process_primary_event(&mut self) {
        let Some(event) = unsafe { self.cx.as_mut().get_unchecked_mut() }
            .primary_ring_mut()
            .pop::<Trb>()
        else {
            return;
        };

        match event {
            Trb::Normal => todo!(),
            Trb::SetupStage => todo!(),
            Trb::DataStage => todo!(),
            Trb::StatusStage => todo!(),
            Trb::Link(_) => todo!(),
            Trb::NoOp => todo!(),
            Trb::EnableSlotCommand => todo!(),
            Trb::AddressDeviceCommand => todo!(),
            Trb::ConfigureEndpoint => todo!(),
            Trb::NoOpCommand => todo!(),
            Trb::TransferEvent => todo!(),
            Trb::CommandCompletionEvent(e) => self.process_command_completion_event(e),
            Trb::PortStatusChangeEvent => todo!(),
            Trb::Unknown(_) => todo!(),
        }
    }

    fn process_command_completion_event(&mut self, event: CommandCompletionEvent) {
        let issuer = unsafe { event.issuer() };
        let slot_id = event.get_control_slot_id();
        debug!("slot_id: {}, issuer: {:?}", slot_id, issuer)
    }
}
