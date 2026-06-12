pub const PUBKEY_SIZE: usize = 32;
pub const SIGNATURE_SIZE: usize = 64;
pub const HASH_SIZE: usize = 32;

pub const MAX_ACCOUNTS: usize = 16;
pub const MAX_DATA: usize = 256;
pub const MAX_TX_ACCOUNTS: usize = 16;

pub const ALLOC_PATH: &str = "/tmp/ipc_alloc";
pub const ALLOC_SIZE: u64 = 512 * 1024;
pub const QUEUE_PATH: &str = "/tmp/ipc_queue";
pub const QUEUE_SIZE: u64 = 512 * 1024;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Account {
    pub pubkey: [u8; PUBKEY_SIZE],
    pub lamports: u64,
    pub owner: [u8; PUBKEY_SIZE],

    pub data_len: u32,
    pub _pad0: u32,
    pub data: [u8; MAX_DATA],

    pub executable: u8,
    pub _pad1: [u8; 7],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct AccountMeta {
    pub pubkey: [u8; PUBKEY_SIZE],
    pub is_signer: u8,
    pub is_writable: u8,
    pub _pad: [u8; 6],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Instruction {
    pub program_id: [u8; PUBKEY_SIZE],

    pub account_count: u32,
    pub _pad0: u32,
    pub accounts: [AccountMeta; MAX_ACCOUNTS],

    pub data_len: u32,
    pub _pad1: u32,
    pub data: [u8; MAX_DATA],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Transaction {
    pub recent_blockhash: [u8; HASH_SIZE],

    pub signer_count: u32,
    pub _pad0: u32,
    pub signers: [[u8; PUBKEY_SIZE]; MAX_TX_ACCOUNTS],

    pub signature_count: u32,
    pub _pad1: u32,
    pub signatures: [[u8; SIGNATURE_SIZE]; MAX_TX_ACCOUNTS],

    pub instruction: Instruction,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ExecutionResult {
    pub request_id: u64,

    pub status: u8,
    pub _pad0: [u8; 7],

    pub compute_units_used: u64,

    pub state_root: [u8; HASH_SIZE],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Bank {
    pub slot: u64,
    pub parent_slot: u64,

    pub state_root: [u8; HASH_SIZE],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MessageHeader {
    pub msg_type: u16,
    pub version: u16,
    pub length: u32,
    pub request_id: u64,
    pub result_offset: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union MessagePayload {
    pub tx: Transaction,
    pub result: ExecutionResult,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Message {
    pub header: MessageHeader,
    pub payload: MessagePayload,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ResultSlot {
    pub ready: u32,
    pub request_id: u64,
    pub status: u8,
    pub _pad0: [u8; 7],
    pub compute_units: u64,
    pub _pad_align: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
}
