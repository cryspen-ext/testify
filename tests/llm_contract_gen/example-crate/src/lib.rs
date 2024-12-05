fn increment(x: u8) -> u8 {
    x + 1
}

fn double_increment(x: u8) -> u8 {
    increment(increment(x))
}
