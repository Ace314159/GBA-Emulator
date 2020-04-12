use core::gba::GBA;

fn main() {
    let mut gba = GBA::new();

    loop {
        gba.emulate();
    }
}
