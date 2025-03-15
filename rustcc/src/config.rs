use crate::compiler::{ObfuscationLevel, OptimizationLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// Configuration for the RustCC compiler
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Optimization configuration
    #[serde(default)]
    pub optimization: OptimizationConfig,

    /// Obfuscation configuration
    #[serde(default)]
    pub obfuscation: ObfuscationConfig,

    /// Output configuration
    #[serde(default)]
    pub output: OutputConfig,
    
    /// Preprocessor configuration
    #[serde(default)]
    pub preprocessor: PreprocessorConfig,
}

/// Configuration for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Optimization level
    #[serde(default = "default_optimization_level")]
    pub level: String,

    /// Threshold for function inlining (number of statements)
    #[serde(default = "default_inline_threshold")]
    pub inline_threshold: usize,

    /// Whether to enable constant folding
    #[serde(default = "default_true")]
    pub constant_folding: bool,

    /// Whether to enable dead code elimination
    #[serde(default = "default_true")]
    pub dead_code_elimination: bool,
}

/// Configuration for obfuscation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObfuscationConfig {
    /// Obfuscation level
    #[serde(default = "default_obfuscation_level")]
    pub level: String,

    /// Style for variable renaming
    #[serde(default = "default_variable_rename_style")]
    pub variable_rename_style: String,

    /// Whether to enable string encryption
    #[serde(default = "default_true")]
    pub string_encryption: bool,

    /// Whether to enable control flow flattening
    #[serde(default = "default_true")]
    pub control_flow_flattening: bool,

    /// Ratio of dead code to insert (0.0 to 1.0)
    #[serde(default = "default_dead_code_ratio")]
    pub dead_code_insertion_ratio: f32,

    /// Complexity of opaque predicates
    #[serde(default = "default_opaque_predicate_complexity")]
    pub opaque_predicate_complexity: String,
}

/// Configuration for output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output format
    #[serde(default = "default_output_format")]
    pub format: String,

    /// Whether to include debug information
    #[serde(default = "default_false")]
    pub debug_info: bool,
}

/// Configuration for preprocessor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessorConfig {
    /// Include paths for preprocessing
    #[serde(default)]
    pub include_paths: Vec<String>,
    
    /// Macro definitions
    #[serde(default)]
    pub defines: HashMap<String, Option<String>>,
    
    /// Whether to keep comments in preprocessor output
    #[serde(default = "default_false")]
    pub keep_comments: bool,
}

impl Config {
    /// Load configuration from a file
    #[allow(dead_code)]
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read config file: {}", e))?;

        match path.as_ref().extension().and_then(|ext| ext.to_str()) {
            Some("toml") => {
                toml::from_str(&content).map_err(|e| format!("Failed to parse TOML config: {}", e))
            }
            Some("json") => serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse JSON config: {}", e)),
            Some(ext) => Err(format!("Unsupported config file extension: {}", ext)),
            None => Err("Config file has no extension".to_string()),
        }
    }

    /// Get the optimization level from the configuration
    pub fn get_optimization_level(&self) -> OptimizationLevel {
        match self.optimization.level.to_lowercase().as_str() {
            "none" => OptimizationLevel::None,
            "basic" => OptimizationLevel::Basic,
            "full" => OptimizationLevel::Full,
            _ => OptimizationLevel::None,
        }
    }

    /// Get the obfuscation level from the configuration
    pub fn get_obfuscation_level(&self) -> ObfuscationLevel {
        match self.obfuscation.level.to_lowercase().as_str() {
            "none" => ObfuscationLevel::None,
            "basic" => ObfuscationLevel::Basic,
            "aggressive" => ObfuscationLevel::Aggressive,
            _ => ObfuscationLevel::None,
        }
    }

    /// Get the include paths from the preprocessor configuration
    pub fn get_include_paths(&self) -> Vec<PathBuf> {
        self.preprocessor.include_paths.iter()
            .map(|path| PathBuf::from(path))
            .collect()
    }
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        OptimizationConfig {
            level: default_optimization_level(),
            inline_threshold: default_inline_threshold(),
            constant_folding: default_true(),
            dead_code_elimination: default_true(),
        }
    }
}

impl Default for ObfuscationConfig {
    fn default() -> Self {
        ObfuscationConfig {
            level: default_obfuscation_level(),
            variable_rename_style: default_variable_rename_style(),
            string_encryption: default_true(),
            control_flow_flattening: default_true(),
            dead_code_insertion_ratio: default_dead_code_ratio(),
            opaque_predicate_complexity: default_opaque_predicate_complexity(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        OutputConfig {
            format: default_output_format(),
            debug_info: default_false(),
        }
    }
}

impl Default for PreprocessorConfig {
    fn default() -> Self {
        PreprocessorConfig {
            include_paths: Vec::new(),
            defines: HashMap::new(),
            keep_comments: default_false(),
        }
    }
}

fn default_optimization_level() -> String {
    "none".to_string()
}

fn default_inline_threshold() -> usize {
    10
}

fn default_obfuscation_level() -> String {
    "none".to_string()
}

fn default_variable_rename_style() -> String {
    "random".to_string()
}

fn default_dead_code_ratio() -> f32 {
    0.2
}

fn default_opaque_predicate_complexity() -> String {
    "medium".to_string()
}

fn default_output_format() -> String {
    "asm".to_string()
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}
