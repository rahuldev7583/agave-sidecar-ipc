use shaq::Producer;
use std::fs::File;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct ShareState {
    data: [u8; 512],
}

fn main() {
    println!("Producer");

    let file = File::options()
        .create(true)
        .read(true)
        .write(true)
        .truncate(true)
        .open("./../test")
        .unwrap();

    println!("file created");

    let mut producer: Producer<ShareState> =
        unsafe { Producer::create(&file, 1024 * 1024) }.unwrap();

    producer.sync();
    unsafe {
        let mut slot = producer.reserve().unwrap();

        slot.as_mut().data.fill(24);
    }

    producer.commit();
    println!("producer committed");
}
