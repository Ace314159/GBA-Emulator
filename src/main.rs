use core::gba::GBA;

fn main() {
    std::env::set_current_dir("ROMs").unwrap();
    let mut gba = GBA::new(&"armwrestler.gba".to_string());

    loop {
        gba.emulate();
    }
}
