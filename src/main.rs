#![forbid(unsafe_code)]
mod parser;
use  parser::DiskInfo;
fn main() {
    if let Some(disk) = DiskInfo::new(true){
        print!("{:#?}",disk);
        disk.mount();
    }
}
