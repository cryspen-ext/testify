fn add_or_zero(x: u8, y: u8) -> u8 {
    // AA
    // AA
    // AA
    // AA
    if (x as u16) + (y as u16) < 256 {
        x + y
    } else {
        // AA
        // AA
        // AA
        0
        // AA
        // AA
        // AA
    }
}
