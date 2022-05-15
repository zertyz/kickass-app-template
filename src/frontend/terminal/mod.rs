mod demo;

pub fn run() {
    std::thread::sleep(std::time::Duration::from_secs(5));
    demo::run_demo(demo::Config {
        enhanced_graphics: false,
        ..Default::default()
    }).expect("Error running Terminal UI");
}
