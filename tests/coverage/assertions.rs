fn main() {
    assert!(example_crate::add_or_zero(99u8, 99u8) == 198u8);
    assert!(example_crate::add_or_zero(12u8, 12u8) == 24u8);
    assert!(example_crate::add_or_zero(74u8, 74u8) == 148u8);
    assert!(example_crate::add_or_zero(14u8, 14u8) == 28u8);
    assert!(example_crate::add_or_zero(3u8, 3u8) == 6u8);
}
