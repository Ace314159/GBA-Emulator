pub(super) fn gen_condition_table() -> [bool; 256] {
    let mut lut = [false; 256];
    let (n_mask, z_mask, c_mask, v_mask) = (0x8, 0x4, 0x2, 0x1);
    for flags in 0 ..= 0xF {
        for condition in 0 ..= 0xF {
            let n = flags & n_mask != 0;
            let z = flags & z_mask != 0;
            let c = flags & c_mask != 0;
            let v = flags & v_mask != 0;
            lut[flags << 4 | condition] = match condition {
                0x0 => z,
                0x1 => !z,
                0x2 => c,
                0x3 => !c,
                0x4 => n,
                0x5 => !n,
                0x6 => v,
                0x7 => !v,
                0x8 => c && !z,
                0x9 => !c || z,
                0xA => n == v,
                0xB => n != v,
                0xC => !z && n == v,
                0xD => z || n != v,
                0xE => true,
                0xF => false, // TODO: Change
                _ => panic!("Invalid Condition"),
            };
        }
    }

    lut
}
