mod application;
mod desktop;
// Sprint 01 defines these primitives before the first application use case consumes them.
// Keep their visibility crate-local while still linting their implementations and tests.
#[allow(dead_code, unused_imports)]
mod domain;
mod infrastructure;

pub fn run() {
    desktop::run();
}
