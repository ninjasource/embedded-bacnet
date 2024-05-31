#[allow(dead_code)]
fn main() {
    // dummy main, this example is not meant to be run but used for common code
}

pub struct DummyBacnetServer {
    pub addr: String,
}

pub fn run_dummy_server() -> DummyBacnetServer {
    DummyBacnetServer {
        addr: "127.0.0.0:BAC0".into(),
    }
}
