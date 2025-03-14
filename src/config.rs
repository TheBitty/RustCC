#[derive(Default)]
pub struct Config {
    pub optimization: OptimizationConfig,
    pub obfuscation: ObfuscationConfig,
    pub output: OutputConfig,
    pub verbose: bool,
} 