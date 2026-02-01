use std::fs::File;

use memmap2::{Mmap, MmapMut};

fn main() {
    println!("hello, world!");

    // let file = File::open("readme.md").unwrap();
    // let mmap = unsafe { Mmap::map(&file).unwrap() };
    // println!("mmap: {:?}", mmap);
    //
    // println!("first 8 byte of file: {:?}", &mmap[0..8]);

    let mut file_mut = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open("file.txt")
        .unwrap();
    file_mut.set_len(14).unwrap();

    let mut mmap_mut = unsafe { MmapMut::map_mut(&file_mut).unwrap() };

    mmap_mut.copy_from_slice(b"Hello MmmapMut");

    println!("mmap_mut: {:?}", mmap_mut);
}
