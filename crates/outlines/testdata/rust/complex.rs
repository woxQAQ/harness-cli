const VERSION: &str = "1.0";

trait Loader {
    fn load(&self, key: &str) -> String;
    fn label(&self) -> &'static str;
}

enum Mode {
    Fast,
    Safe,
}

struct Config {
    mode: Mode,
    retries: usize,
    timeout_ms: u64,
}

struct Engine<L> {
    loader: L,
    config: Config,
}

impl Config {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            retries: 3,
            timeout_ms: 500,
        }
    }

    fn with_retry(mut self, retries: usize) -> Self {
        self.retries = retries;
        self
    }
}

impl<L: Loader> Engine<L> {
    fn new(loader: L, config: Config) -> Self {
        Self { loader, config }
    }

    fn run(&self, key: &str) -> String {
        let prefix = self.loader.label();
        format!("{}:{}", prefix, self.loader.load(key))
    }

    fn retry_budget(&self) -> usize {
        self.config.retries
    }
}

fn bootstrap<L: Loader>(loader: L) -> Engine<L> {
    let config = Config::new(Mode::Safe).with_retry(5);
    Engine::new(loader, config)
}
