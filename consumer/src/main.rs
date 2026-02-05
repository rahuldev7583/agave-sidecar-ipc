use shaq::Consumer;
use std::fs::File;

#[derive(Debug, Copy, Clone)]
struct ShareState {
    data: [u8; 512],
}

fn main() {
    println!("Consumer");

    let file = File::options()
        .read(true)
        .write(true)
        .open("./../test")
        .unwrap();

    let mut consumer: Consumer<ShareState> = unsafe { Consumer::join(&file).unwrap() };
    let value = consumer.try_read().unwrap();

    println!("the value is : {:?}", value);
}
