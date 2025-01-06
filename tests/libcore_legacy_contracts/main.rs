mod imported;

fn main() {
    testify::driver::setup_tracing();
    testify::driver::run(imported::contracts(), "regressions.rs");
}
