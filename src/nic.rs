extern crate alloc;
use core::ops::Add;

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

        let mmio_base = bar0.addr() as u8;
        info!("[NIC] mmio_base:{}", mmio_base);

        //mmio
        Nic::new(mmio_base);
        // let regs: = nic::write_register;

        // spawn_global(Self::run());
        Ok(())
    }
}


#[derive(Debug)]
pub struct Nic {
    mmio_base: u8,
    t_descriptor: *mut TDescriptor,
    r_descriptor: *mut RDescriptor,
    packet_buffer: u64,
}

static mut T_DESC_RING_BUFFER: [TDescriptor; T_DESC_NUM] = [TDescriptor::new(); T_DESC_NUM];
static mut R_DESC_RING_BUFFER: [RDescriptor; R_DESC_NUM] = [RDescriptor::new(); R_DESC_NUM];

const PACKET_SIZE: usize = 2048;
const T_DESC_CMD_RS :u8 = 0b00001000;

impl Nic {

    pub fn new(mmio_base: u8) -> Self {
        let t_desc: *mut TDescriptor;
        let r_desc: *mut RDescriptor;
        unsafe {
            t_desc = T_DESC_RING_BUFFER.as_mut_ptr();
            r_desc = R_DESC_RING_BUFFER.as_mut_ptr();
        };
        info!("[NIC] t_desc_ring_buffer:{:?}", t_desc);
        info!("[NIC] r_desc_ring_buffer:{:?}", r_desc);
        Nic { mmio_base, t_descriptor: t_desc, r_descriptor: r_desc, packet_buffer: 0 }
    }

    pub fn initialize(&mut self, accept_all: bool) {
        
        unsafe {
            self.t_descriptor = T_DESC_RING_BUFFER.as_mut_ptr();
            self.r_descriptor = R_DESC_RING_BUFFER.as_mut_ptr();

            for i in 0..=T_DESC_NUM {
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
            for i in 0..=R_DESC_NUM {
                R_DESC_RING_BUFFER[i].buffer_address = (self.packet_buffer as *const u64).add(i * PACKET_SIZE) as u64;
                R_DESC_RING_BUFFER[i].status = 0;
                R_DESC_RING_BUFFER[i].errors = 0;
            }

        }


    }

    fn get_register(offset: u16) -> u32 {
        0
    }

    fn write_register(bar0: &BarMem64) -> Result<()> {
        Ok(())
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

