use std::mem;

// CQC Protocol Version
pub const CQC_VERSION: u8 = 2;

// CQC Message Types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CQCType {
    Hello = 0,
    Command = 1,
    Factory = 2,
    Expire = 3,
    Done = 4,
    Recv = 5,
    EprOk = 6,
    MeasOut = 7,
    GetTime = 8,
    InfTime = 9,
    NewOk = 10,
    Mix = 11,
    If = 12,

    // Errors
    ErrGeneral = 20,
    ErrNoQubit = 21,
    ErrUnsupp = 22,
    ErrTimeout = 23,
    ErrInUse = 24,
    ErrUnknown = 25,
}

// CQC Commands
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CQCCmd {
    I = 0,
    New = 1,
    Measure = 2,
    MeasureInplace = 3,
    Reset = 4,
    Send = 5,
    Recv = 6,
    Epr = 7,
    EprRecv = 8,

    X = 10,
    Z = 11,
    Y = 12,
    T = 13,
    RotX = 14,
    RotY = 15,
    RotZ = 16,
    H = 17,
    K = 18,

    Cnot = 20,
    Cphase = 21,
    Allocate = 22,
    Release = 23,
}

// CQC Options
pub const CQC_OPT_NOTIFY: u8 = 0x01;
pub const CQC_OPT_ACTION: u8 = 0x02;
pub const CQC_OPT_BLOCK: u8 = 0x04;
pub const CQC_OPT_IFTHEN: u8 = 0x08;

// CQC Headers (packed, C-compatible)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CQCHeader {
    pub version: u8,
    pub msg_type: u8,
    pub app_id: u16,
    pub length: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CQCCmdHeader {
    pub qubit_id: u16,
    pub instr: u8,
    pub options: u8,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CQCRotationHeader {
    pub step: u8, // Angle in units of pi/256
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CQCXtraQubitHeader {
    pub qubit_id: u16,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CQCCommunicationHeader {
    pub remote_app_id: u16,
    pub remote_port: u16,
    pub remote_node: u32, // IPv4 address
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CQCFactoryHeader {
    pub num_iter: u8,
    pub options: u8,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CQCMeasOutHeader {
    pub meas_out: u8,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CQCTimeinfoHeader {
    pub datetime: u64,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct EntanglementInfoHeader {
    pub node_a: u32,
    pub port_a: u16,
    pub app_id_a: u16,
    pub node_b: u32,
    pub port_b: u16,
    pub app_id_b: u16,
    pub id_ab: u32,
    pub timestamp: u64,
    pub tog: u64, // Time of goodness
    pub goodness: u16,
    pub df: u8, // Directionality flag
    pub unused: u8,
}

// Serialization helpers
impl CQCHeader {
    pub fn new(msg_type: CQCType, app_id: u16, length: u32) -> Self {
        CQCHeader {
            version: CQC_VERSION,
            msg_type: msg_type as u8,
            app_id,
            length,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let ptr = self as *const Self as *const u8;
            let slice = std::slice::from_raw_parts(ptr, mem::size_of::<Self>());
            slice.to_vec()
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if bytes.len() < mem::size_of::<Self>() {
            return Err("Insufficient bytes for CQCHeader".into());
        }
        unsafe { Ok(*(bytes.as_ptr() as *const Self)) }
    }
}

// Add these trait implementations for all the packed structs

impl CQCCmdHeader {
    pub fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let ptr = self as *const Self as *const u8;
            let slice = std::slice::from_raw_parts(ptr, mem::size_of::<Self>());
            slice.to_vec()
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if bytes.len() < mem::size_of::<Self>() {
            return Err("Insufficient bytes for CQCCmdHeader".into());
        }
        unsafe { Ok(*(bytes.as_ptr() as *const Self)) }
    }
}

impl CQCRotationHeader {
    pub fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let ptr = self as *const Self as *const u8;
            let slice = std::slice::from_raw_parts(ptr, mem::size_of::<Self>());
            slice.to_vec()
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if bytes.len() < mem::size_of::<Self>() {
            return Err("Insufficient bytes for CQCRotationHeader".into());
        }
        unsafe { Ok(*(bytes.as_ptr() as *const Self)) }
    }
}

impl CQCXtraQubitHeader {
    pub fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let ptr = self as *const Self as *const u8;
            let slice = std::slice::from_raw_parts(ptr, mem::size_of::<Self>());
            slice.to_vec()
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if bytes.len() < mem::size_of::<Self>() {
            return Err("Insufficient bytes for CQCXtraQubitHeader".into());
        }
        unsafe { Ok(*(bytes.as_ptr() as *const Self)) }
    }
}

impl CQCCommunicationHeader {
    pub fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let ptr = self as *const Self as *const u8;
            let slice = std::slice::from_raw_parts(ptr, mem::size_of::<Self>());
            slice.to_vec()
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if bytes.len() < mem::size_of::<Self>() {
            return Err("Insufficient bytes for CQCCommunicationHeader".into());
        }
        unsafe { Ok(*(bytes.as_ptr() as *const Self)) }
    }
}

impl CQCFactoryHeader {
    pub fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let ptr = self as *const Self as *const u8;
            let slice = std::slice::from_raw_parts(ptr, mem::size_of::<Self>());
            slice.to_vec()
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if bytes.len() < mem::size_of::<Self>() {
            return Err("Insufficient bytes for CQCFactoryHeader".into());
        }
        unsafe { Ok(*(bytes.as_ptr() as *const Self)) }
    }
}

impl CQCMeasOutHeader {
    pub fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let ptr = self as *const Self as *const u8;
            let slice = std::slice::from_raw_parts(ptr, mem::size_of::<Self>());
            slice.to_vec()
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if bytes.len() < mem::size_of::<Self>() {
            return Err("Insufficient bytes for CQCMeasOutHeader".into());
        }
        unsafe { Ok(*(bytes.as_ptr() as *const Self)) }
    }
}

impl CQCTimeinfoHeader {
    pub fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let ptr = self as *const Self as *const u8;
            let slice = std::slice::from_raw_parts(ptr, mem::size_of::<Self>());
            slice.to_vec()
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if bytes.len() < mem::size_of::<Self>() {
            return Err("Insufficient bytes for CQCTimeinfoHeader".into());
        }
        unsafe { Ok(*(bytes.as_ptr() as *const Self)) }
    }
}

impl EntanglementInfoHeader {
    pub fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let ptr = self as *const Self as *const u8;
            let slice = std::slice::from_raw_parts(ptr, mem::size_of::<Self>());
            slice.to_vec()
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if bytes.len() < mem::size_of::<Self>() {
            return Err("Insufficient bytes for EntanglementInfoHeader".into());
        }
        unsafe { Ok(*(bytes.as_ptr() as *const Self)) }
    }
}

// Also add these convenience methods for CQC Types
impl CQCType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(CQCType::Hello),
            1 => Some(CQCType::Command),
            2 => Some(CQCType::Factory),
            3 => Some(CQCType::Expire),
            4 => Some(CQCType::Done),
            5 => Some(CQCType::Recv),
            6 => Some(CQCType::EprOk),
            7 => Some(CQCType::MeasOut),
            8 => Some(CQCType::GetTime),
            9 => Some(CQCType::InfTime),
            10 => Some(CQCType::NewOk),
            11 => Some(CQCType::Mix),
            12 => Some(CQCType::If),
            20 => Some(CQCType::ErrGeneral),
            21 => Some(CQCType::ErrNoQubit),
            22 => Some(CQCType::ErrUnsupp),
            23 => Some(CQCType::ErrTimeout),
            24 => Some(CQCType::ErrInUse),
            25 => Some(CQCType::ErrUnknown),
            _ => None,
        }
    }
}

impl CQCCmd {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(CQCCmd::I),
            1 => Some(CQCCmd::New),
            2 => Some(CQCCmd::Measure),
            3 => Some(CQCCmd::MeasureInplace),
            4 => Some(CQCCmd::Reset),
            5 => Some(CQCCmd::Send),
            6 => Some(CQCCmd::Recv),
            7 => Some(CQCCmd::Epr),
            8 => Some(CQCCmd::EprRecv),
            10 => Some(CQCCmd::X),
            11 => Some(CQCCmd::Z),
            12 => Some(CQCCmd::Y),
            13 => Some(CQCCmd::T),
            14 => Some(CQCCmd::RotX),
            15 => Some(CQCCmd::RotY),
            16 => Some(CQCCmd::RotZ),
            17 => Some(CQCCmd::H),
            18 => Some(CQCCmd::K),
            20 => Some(CQCCmd::Cnot),
            21 => Some(CQCCmd::Cphase),
            22 => Some(CQCCmd::Allocate),
            23 => Some(CQCCmd::Release),
            _ => None,
        }
    }
}
