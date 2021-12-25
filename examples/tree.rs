extern crate cardinal;

fn main() {
    cardinal::init_sdk();
    loop {
        std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
        let events = cardinal::take_fs_events();
        if !events.is_empty() {
            println!("{:#?}", events);
        }
    }
}
