use std::{fs::File, thread, time::Duration};

use ipc_shared::{ALLOC_PATH, Message, QUEUE_PATH, ResultSlot};
use rts_alloc::{Allocator, error::Error};
use shaq::spsc::Consumer;

fn main() {
    println!("Consumer");

    let alloc_file = loop {
        match File::options().write(true).read(true).open(ALLOC_PATH) {
            Ok(f) => break f,
            Err(_) => thread::sleep(Duration::from_millis(10)),
        }
    };

    println!("alloc file got");

    let alloc = loop {
        match unsafe { Allocator::join(&alloc_file) } {
            Ok(a) => break a,
            Err(e) => {
                match e {
                    Error::InvalidMagic => {}
                    _ => panic!("real error: {:?}", e),
                }
                eprintln!("alloc join failed: {:?}, retrying...", e);
                thread::sleep(Duration::from_millis(10));
            }
        }
    };

    println!("alloc got");

    let queue_file = loop {
        match File::options().write(true).read(true).open(QUEUE_PATH) {
            Ok(f) => break f,
            Err(_) => thread::sleep(Duration::from_millis(10)),
        }
    };

    println!("queue file got");

    let mut consumer: Consumer<Message> = loop {
        match unsafe { Consumer::join(&queue_file) } {
            Ok(c) => break c,
            Err(_) => thread::sleep(Duration::from_millis(10)),
        }
    };

    println!("consumer got");

    loop {
        consumer.sync();

        if let Some(msg) = consumer.try_read() {
            match msg.header.msg_type {
                1 => {
                    println!("Received TX, request_id={}", msg.header.request_id);

                    let result_ptr = unsafe {
                        alloc
                            .ptr_from_offset(msg.header.result_offset as usize)
                            .as_ptr() as *mut ResultSlot
                    };

                    let result = ResultSlot {
                        ready: 1,
                        request_id: msg.header.request_id,
                        status: 1,
                        _pad0: [0; 7],
                        compute_units: 500,
                        _pad_align: 0,
                    };

                    unsafe {
                        *result_ptr = result;
                    }

                    println!("Wrote result back to shared memory");
                }
                _ => {}
            }

            consumer.finalize();
        }
    }
}
