mod imported;

fn main() {
    testify::driver::run(imported::contracts(), "regressions.rs");
}
