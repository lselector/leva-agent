/// Configuration and paths loaded from environment / .env
use std::path::PathBuf;
use std::sync::OnceLock;

static CONFIG: OnceLock<Config> = OnceLock::new();

pub struct Config {
    pub base_dir: PathBuf,
    /// Private repo dir (LEVA_AGENT_PRIV_DIR), if set.
    /// Used for credentials, memory, and private skills/resources.
    pub priv_dir: Option<PathBuf>,
    pub prompts_dir: PathBuf,
    pub soul_dir: PathBuf,
    pub memory_dir: PathBuf,
    pub memory_topics_dir: PathBuf,
    pub reference_dir: PathBuf,
    pub credentials_dir: PathBuf,
    pub anthropic_api_key: String,
    pub model_name: std::sync::RwLock<String>,
    pub llm_port: u16,
    pub auto_port: u16,
    pub perplexity_api_key: String,
}

impl Config {
    fn load() -> Self {
        let priv_dir: Option<PathBuf> = std::env::var("LEVA_AGENT_PRIV_DIR").ok().map(|p| {
            let pb = PathBuf::from(p);
            pb.canonicalize().unwrap_or(pb)
        });

        if let Some(ref pd) = priv_dir {
            let priv_env = pd.join(".env");
            if priv_env.exists() {
                let _ = dotenvy::from_path(&priv_env);
            }
        }

        let exe = std::env::current_exe().unwrap_or_default();
        let mut dir = exe.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
        loop {
            let candidate = dir.join(".env");
            if candidate.exists() {
                let _ = dotenvy::from_path(&candidate);
                break;
            }
            if !dir.pop() {
                let _ = dotenvy::dotenv();
                break;
            }
        }

        let base_dir = PathBuf::from(
            std::env::var("LEVA_BASE_DIR").unwrap_or_else(|_| "../..".to_string()),
        );
        let base_dir = base_dir.canonicalize().unwrap_or(base_dir);

        let data_root = priv_dir.clone().unwrap_or_else(|| base_dir.clone());
        let memory_dir = data_root.join("memory");
        let credentials_dir = data_root.join("credentials");
        let memory_topics_dir = memory_dir.join("topics");

        Config {
            priv_dir,
            prompts_dir: base_dir.join("prompts"),
            soul_dir: base_dir.join("soul"),
            memory_topics_dir,
            credentials_dir,
            reference_dir: base_dir.join("reference"),
            memory_dir,
            base_dir,
            anthropic_api_key: std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
            model_name: std::sync::RwLock::new(
                std::env::var("MODEL_NAME")
                    .unwrap_or_else(|_| "claude-sonnet-4-6".to_string()),
            ),
            llm_port: std::env::var("LLM_PORT").ok().and_then(|v| v.parse().ok()).unwrap_or(8000),
            auto_port: std::env::var("AUTO_PORT").ok().and_then(|v| v.parse().ok()).unwrap_or(8001),
            perplexity_api_key: std::env::var("PERPLEXITY_API_KEY").unwrap_or_default(),
        }
    }
}

/// Global config accessor — initialised once on first call.
pub fn get() -> &'static Config {
    CONFIG.get_or_init(Config::load)
}
