extern crate alloc;
use core::{mem::size_of, ops::Add};

use crate::{executor::spawn_global, info, mmio, pci::{BarMem64, BusDeviceFunction, VendorDeviceId}, println};
use crate::pci::Pci;
use crate::result::Result;
use alloc::vec::Vec;

pub struct NicDriver {
    nic: Nic,
}
impl NicDriver {
    pub fn supports(vp: VendorDeviceId) -> bool {
        const VDI_LIST: [VendorDeviceId; 1] = [
            // NIC Intel e1000e
            VendorDeviceId {
                vendor: 0x8086,
                device: 0x10D3, 
            },
        ];
        VDI_LIST.contains(&vp)
    }

    pub fn attach(pci: &Pci, bdf: BusDeviceFunction) -> Result<()> {

        info!("Nic found at: {bdf:?}");
        pci.disable_interrupt(bdf)?;
        pci.enable_bus_master(bdf)?;

        //bar
        let bar0 = pci.try_bar0_mem64(bdf)?;
        info!("[BAR0_NIC]{:?}", bar0);
        bar0.disable_cache();

        info!("[NIC] bar0.addr():{:?}", bar0.addr());
        let mmio_base = bar0.addr() as u64;
        info!("[NIC] mmio_base:{}", mmio_base);

        //mmio
        let mut nic = Nic::new(mmio_base);

        nic.initialize(true);
        // let regs: = nic::write_register;

        // spawn_global(Self::run());
        Ok(())
    }
}


#[derive(Debug)]
pub struct Nic {
    mmio_base: u64,
    t_descriptor: *mut TDescriptor,
    r_descriptor: *mut RDescriptor,
    packet_buffer: u64,
    t_tail: u64,
    r_tail: u64,
}

static mut T_DESC_RING_BUFFER: [TDescriptor; T_DESC_NUM] = [TDescriptor::new(); T_DESC_NUM];
static mut R_DESC_RING_BUFFER: [RDescriptor; R_DESC_NUM] = [RDescriptor::new(); R_DESC_NUM];

const PACKET_SIZE: usize  = 2048;
const T_DESC_CMD_RS: u8   = 0b00001000;

const CTRL_OFFSET :u16    = 0x00000;
const CTRL_FD: u16        = 0x00000001;
const CTRL_ASDE: u16      = 0x000000020;
const CTRL_SLU: u16       = 0x000000040;

const TCTL_OFFSET:u16    = 0x00400;
const TIPG_OFFSET:u16    = 0x00410;
const TDBAL_OFFSET:u16   = 0x03800;
const TDBAH_OFFSET:u16   = 0x03804;
const TDLEN_OFFSET:u16   = 0x03808;
const TDH_OFFSET:u16     = 0x03810;
const TDT_OFFSET:u16     = 0x03818;
const TCTL_EN:u32        = 0x00000002;
const TCTL_PSP:u32       = 0x00000008;
const TCTL_CT:u32        = 0x00000100;
const TCTL_COLD:u32      = 0x00400000;
const TIPG_IPGT:u16      = 8;
const TIPG_IPGR1:u16     = 8;
const TIPG_IPGR2:u16     = 6;

impl Nic {

    pub fn new(mmio_base: u64) -> Self {
        let t_desc: *mut TDescriptor;
        let r_desc: *mut RDescriptor;
        unsafe {
            t_desc = T_DESC_RING_BUFFER.as_mut_ptr();
            r_desc = R_DESC_RING_BUFFER.as_mut_ptr();
        };
        info!("[NIC] t_desc_ring_buffer:{:?}", t_desc);
        info!("[NIC] r_desc_ring_buffer:{:?}", r_desc);

        Nic { mmio_base, t_descriptor: t_desc, r_descriptor: r_desc, packet_buffer: 0, t_tail: 0, r_tail: 0 }
    }

    pub fn initialize(&mut self, accept_all: bool) {
        
        unsafe {
            self.t_descriptor = T_DESC_RING_BUFFER.as_mut_ptr();
            self.r_descriptor = R_DESC_RING_BUFFER.as_mut_ptr();

            info!("[NIC] t_desc = {:?}", self.t_descriptor);
            info!("[NIC] r_desc = {:?}", self.r_descriptor);

            for i in 0..T_DESC_NUM {

                T_DESC_RING_BUFFER[i].buffer_address = 0;
                T_DESC_RING_BUFFER[i].length = 0;
                T_DESC_RING_BUFFER[i].checksum_offset = 0;
                T_DESC_RING_BUFFER[i].command = T_DESC_CMD_RS;
                T_DESC_RING_BUFFER[i].status = 0;
                T_DESC_RING_BUFFER[i].reserved = 0;
                T_DESC_RING_BUFFER[i].checksum_start_field = 0;
                T_DESC_RING_BUFFER[i].special = 0;

            }

            self.packet_buffer = [[0; PACKET_SIZE]; R_DESC_NUM].as_ptr() as u64;

            for i in 0..R_DESC_NUM {
                info!("[NIC] RDESC index = {i}");
                R_DESC_RING_BUFFER[i].buffer_address = (self.packet_buffer as *const u64).add(i * PACKET_SIZE) as u64;
                R_DESC_RING_BUFFER[i].status = 0;
                R_DESC_RING_BUFFER[i].errors = 0;
                info!("[NIC] R_DESC_RING_BUFFER = {:?}", R_DESC_RING_BUFFER[i]);
            }

        }

        self.write_register(CTRL_OFFSET, (CTRL_FD | CTRL_ASDE | CTRL_SLU) as u32);

        self.t_tail = 0;
        self.r_tail = R_DESC_NUM as u64  - 1;

        self.write_register(TCTL_OFFSET, (TCTL_EN | TCTL_PSP | TCTL_CT | TCTL_COLD) as u32);
        self.write_register(TIPG_OFFSET, (TIPG_IPGT | TIPG_IPGR1 | TIPG_IPGR2) as u32);
        self.write_register(TDBAL_OFFSET, self.t_descriptor as u32);
        self.write_register(TDBAH_OFFSET, (self.t_descriptor as u64 >> 32) as u32);
        self.write_register(TDLEN_OFFSET, (size_of::<TDescriptor>() * T_DESC_NUM) as u32);
        self.write_register(TDH_OFFSET, self.t_tail as u32);
        self.write_register(TDT_OFFSET, self.t_tail as u32);

    }

    fn read_register(&self, offset: u16) -> u32 {
        let reg_addr = offset + (self.mmio_base as u16);
        reg_addr as u32
    }

    fn write_register(&self, offset: u16, value: u32) {
        info!("{:?}", self.mmio_base);
        let reg_addr = (offset as u64 + (self.mmio_base)) as *mut u32;
        info!("[NIC] {offset}, {:04X}, {:?}", value, reg_addr);
        unsafe {
            *reg_addr = value;
        };
        info!("[NIC] *reg_addr={:?}", &reg_addr);
    }

}

const T_DESC_NUM:usize = 8;
const R_DESC_NUM:usize = 16;

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug)]
pub struct TDescriptor {
    buffer_address: u64,
    length: u16,
    checksum_offset: u8,
    command: u8,
    status: u8, //4
    reserved: u8, //4
    checksum_start_field: u8,
    special: u16,
}

impl TDescriptor {
    pub const fn new() -> Self {
        TDescriptor { 
            buffer_address: 0, 
            length: 0, 
            checksum_offset: 0, 
            command: 0, 
            status: 4, 
            reserved: 4, 
            checksum_start_field: 0, 
            special: 0, 
        }
    }
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug)]
struct RDescriptor {
    buffer_address: u64,
    length: u16,
    reserved: u16,
    status: u8,
    errors: u8,
    special: u16,
}
impl RDescriptor {
    pub const fn new() -> Self {
        RDescriptor { 
            buffer_address: 0, 
            length: 0, 
            reserved: 4,  
            status: 4, 
            errors: 0,
            special: 0, 
        }
    }
}

