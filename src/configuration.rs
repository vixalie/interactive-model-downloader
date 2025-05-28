use std::sync::{Arc, LazyLock, Mutex};

use reqwest::{Proxy, Url};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CivitaiConfig {
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HuggingFaceConfig {
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub protocol: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl ProxyConfig {
    pub fn get_proxy_url(&self) -> Option<Url> {
        if self.protocol.is_none() || self.host.is_none() || self.port.is_none() {
            return None;
        }
        let url_str = format!(
            "{}://{}",
            self.protocol.clone().unwrap_or("http".to_string()),
            self.host.clone().unwrap_or("127.0.0.1".to_string())
        );
        let mut url = reqwest::Url::parse(&url_str).unwrap();
        if let Some(port) = self.port {
            url.set_port(Some(port)).unwrap();
        }
        if let Some(username) = &self.username {
            url.set_username(username).unwrap();
        }
        if let Some(password) = &self.password {
            url.set_password(Some(password)).unwrap();
        }
        Some(url)
    }

    pub fn get_proxy(&self) -> Option<Proxy> {
        self.get_proxy_url().and_then(|url| Proxy::all(url).ok())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Configuration {
    pub civitai: CivitaiConfig,
    pub huggingface: HuggingFaceConfig,
    pub proxy: ProxyConfig,
}

pub static CONFIGURATION: LazyLock<Arc<Mutex<Configuration>>> = LazyLock::new(|| {
    let config_dir = directories::UserDirs::new()
        .map(|dirs| dirs.home_dir().to_path_buf())
        .map(|home_dir| home_dir.join(".config"))
        .map(|config_dir| config_dir.join("imd"));
    if let Some(conf_dir) = config_dir {
        if !conf_dir.exists() {
            std::fs::create_dir_all(&conf_dir).expect("Failed to create config directory.");
        }
        let config_file_path = conf_dir.join("config.toml");
        if config_file_path.exists() {
            let config =
                std::fs::read_to_string(config_file_path).expect("Failed to read config file.");
            let config: Configuration =
                toml::from_str(&config).expect("Failed to parse config file.");
            return Arc::new(Mutex::new(config));
        }
    } else {
        panic!("Failed to get config directory.");
    }
    Arc::new(Mutex::new(Configuration::default()))
});

impl Configuration {
    fn save(&self) -> Result<(), std::io::Error> {
        let config_dir = directories::UserDirs::new()
            .map(|dirs| dirs.home_dir().to_path_buf())
            .map(|home_dir| home_dir.join(".config"))
            .map(|config_dir| config_dir.join("imd"));
        if let Some(conf_dir) = config_dir {
            if !conf_dir.exists() {
                std::fs::create_dir_all(&conf_dir)?;
            }
            let config_file_path = conf_dir.join("config.toml");
            let config = toml::to_string(self)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            std::fs::write(config_file_path, config)?;
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Failed to get config directory.",
            ))
        }
    }

    pub fn set_civitai_api_key(&mut self, api_key: String) -> Result<(), std::io::Error> {
        self.civitai.api_key = Some(api_key);
        self.save()
    }

    pub fn clear_civitai_api_key(&mut self) -> Result<(), std::io::Error> {
        self.civitai.api_key = None;
        self.save()
    }

    pub fn set_huggingface_api_key(&mut self, api_key: String) -> Result<(), std::io::Error> {
        self.huggingface.api_key = Some(api_key);
        self.save()
    }

    pub fn clear_huggingface_api_key(&mut self) -> Result<(), std::io::Error> {
        self.huggingface.api_key = None;
        self.save()
    }

    pub fn set_proxy(
        &mut self,
        protocol: String,
        host: String,
        port: Option<u16>,
        username: Option<String>,
        password: Option<String>,
    ) -> Result<(), std::io::Error> {
        self.proxy.protocol = Some(protocol);
        self.proxy.host = Some(host);
        self.proxy.port = port;
        self.proxy.username = username;
        self.proxy.password = password;
        self.save()
    }

    pub fn clear_proxy(&mut self) -> Result<(), std::io::Error> {
        self.proxy = ProxyConfig::default();
        self.save()
    }
}
