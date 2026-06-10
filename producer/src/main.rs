use ipc_shared::*;
use rts_alloc::Allocator;
use shaq::spsc::Producer;
use std::{fs::File, ptr::NonNull};

fn main() {
    println!("Producer");

    let path = "/tmp/ipc_test";

    let file = File::options()
        .create(true)
        .read(true)
        .write(true)
        .open(path)
        .unwrap();

    let alloc = unsafe { Allocator::create(&file, 1024 * 1024, 2, 64 * 1024).unwrap() };

    file.sync_all().unwrap();

    let mut producer: Producer<Message> = unsafe { Producer::create(&file, 1024 * 1024) }.unwrap();

    file.sync_all().unwrap();

    producer.sync();

    let result_ptr = unsafe {
        let mem = alloc
            .allocate(size_of::<ResultSlot>().try_into().unwrap())
            .unwrap();
        mem.as_ptr() as *mut ResultSlot
    };

    let result_offset = unsafe { alloc.offset(NonNull::new(result_ptr as *mut u8).unwrap()) };

    let tx = Transaction {
        recent_blockhash: [1; HASH_SIZE],

        signer_count: 1,
        _pad0: 0,
        signers: [[2; PUBKEY_SIZE]; MAX_TX_ACCOUNTS],

        signature_count: 1,
        _pad1: 0,
        signatures: [[3; SIGNATURE_SIZE]; MAX_TX_ACCOUNTS],

        instruction: Instruction {
            program_id: [4; PUBKEY_SIZE],

            account_count: 1,

            _pad0: 0,
            accounts: [AccountMeta {
                pubkey: [5; PUBKEY_SIZE],
                is_signer: 1,
                is_writable: 1,
                _pad: [0; 6],
            }; MAX_ACCOUNTS],

            data_len: 4,
            _pad1: 0,
            data: [9; MAX_DATA],
        },
    };

    let msg = Message {
        header: MessageHeader {
            msg_type: 1,
            version: 1,
            length: std::mem::size_of::<Message>() as u32,
            request_id: 42,
            result_offset: result_offset as u64,
        },
        payload: MessagePayload { tx },
    };

    unsafe {
        let mut slot = producer.reserve().unwrap();
        *slot.as_mut() = msg;
    }

    producer.commit();

    println!("Transaction sent, waiting for result...");

    loop {
        let ready = unsafe { (*result_ptr).ready };
        if ready == 1 {
            let result = unsafe { *result_ptr };
            println!(
                "Got result: status={}, compute_units={}, request_id={}",
                result.status, result.compute_units, result.request_id
            );
            break;
        }
    }
}
