/// Configuration and paths loaded from environment / .env
use std::path::PathBuf;
use std::sync::OnceLock;

static CONFIG: OnceLock<Config> = OnceLock::new();

pub struct Config {
    pub base_dir: PathBuf,
    pub prompts_dir: PathBuf,
    pub soul_dir: PathBuf,
    pub memory_dir: PathBuf,
    pub memory_topics_dir: PathBuf,
    pub reference_dir: PathBuf,
    pub credentials_dir: PathBuf,
    pub openai_api_key: String,
    pub model_name: std::sync::RwLock<String>,
    pub llm_port: u16,
    pub auto_port: u16,
    pub perplexity_api_key: String,
}

impl Config {
    fn load() -> Self {
        // Load .env from project root (rust_version/../.env)
        let exe = std::env::current_exe().unwrap_or_default();
        // Walk up to find the .env file
        let mut dir = exe.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
        loop {
            let candidate = dir.join(".env");
            if candidate.exists() {
                let _ = dotenvy::from_path(&candidate);
                break;
            }
            if !dir.pop() {
                // Fallback: try cwd
                let _ = dotenvy::dotenv();
                break;
            }
        }

        // BASE_DIR is two levels above the rust_version/src/ —
        // i.e., the repository root containing frontend/, soul/, etc.
        // At runtime the binary lives in rust_version/target/…, so we
        // derive BASE_DIR from the CARGO_MANIFEST_DIR set at compile time.
        let base_dir = PathBuf::from(
            std::env::var("JARVIS_BASE_DIR")
                .unwrap_or_else(|_| "../..".to_string()),
        );
        // Canonicalize so relative paths work wherever the binary runs
        let base_dir = base_dir.canonicalize().unwrap_or(base_dir);

        let memory_dir = base_dir.join("memory");

        Config {
            prompts_dir: base_dir.join("prompts"),
            soul_dir: base_dir.join("soul"),
            memory_topics_dir: memory_dir.join("topics"),
            credentials_dir: base_dir.join("credentials"),
            reference_dir: base_dir.join("reference"),
            memory_dir,
            base_dir,
            openai_api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            model_name: std::sync::RwLock::new(
                std::env::var("MODEL_NAME")
                    .unwrap_or_else(|_| "gpt-4.1-mini".to_string()),
            ),
            llm_port: std::env::var("LLM_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8000),
            auto_port: std::env::var("AUTO_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8001),
            perplexity_api_key: std::env::var("PERPLEXITY_API_KEY").unwrap_or_default(),
        }
    }
}

/// Global config accessor — initialised once on first call.
pub fn get() -> &'static Config {
    CONFIG.get_or_init(Config::load)
}
