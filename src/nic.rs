use crate::info;


#[derive(Debug)]
pub struct Nic {
    mmio_base: u64,
    t_descriptor: *mut TDescriptor,
    r_descriptor: *mut RDescriptor,
}

static mut T_DESC_RING_BUFFER: [TDescriptor; T_DESC_NUM] = [TDescriptor::new(); T_DESC_NUM];
static mut R_DESC_RING_BUFFER: [RDescriptor; R_DESC_NUM] = [RDescriptor::new(); R_DESC_NUM];

impl Nic {

    pub fn new(mmio_base: u64) -> Self {

    }

    pub fn initialize(&mut self, accept_all: bool) {
        
        unsafe {
            self.t_descriptor = T_DESC_RING_BUFFER.as_mut_ptr();
            self.r_descriptor = R_DESC_RING_BUFFER.as_mut_ptr();
        }

        info!("[NIC]{:?}", self.t_descriptor);
        info!("[NIC]{:?}", self.r_descriptor);
    }

    fn get_nic_reg(reg_offset: u16) -> u32 {
        0
    }

    fn set_nic_reg(reg_offset: u16, value: u32) {

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
    pub fn new() -> Self {
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
    pub fn new() -> Self {
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

