pub fn u8_to_bool(v: u8) -> bool {
    match v {
        0 => false,
        1 => true,
        _ => panic!("Invalid bool in u8 {}", v),
    }
}
