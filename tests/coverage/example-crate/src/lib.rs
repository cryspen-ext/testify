fn add_or_zero(x: u8, y: u8) -> u8 {
    // Some irrelevant comment line
    if (x as u16) + (y as u16) < 256 {
        x + y
    } else {
        // Another comment
        0
    }
}
