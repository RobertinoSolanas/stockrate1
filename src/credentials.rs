use std::collections::HashMap;
use std::fs;

pub struct Credentials {
    map: HashMap<String, String>,
}

impl Credentials {
    pub fn load_from_file(path: &str) -> Self {
        let map = match fs::read_to_string(path) {
            Ok(content) => {
                let mut map = HashMap::new();
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some(key_value) = line.split_once('=') {
                        map.insert(key_value.0.trim().to_string(), key_value.1.trim().to_string());
                    }
                }
                map
            }
            Err(_) => HashMap::new(),
        };
        Credentials { map }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.map.get(key).cloned()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_temp_creds(suffix: &str, content: &str) -> std::path::PathBuf {
        let hash = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let path = std::env::temp_dir().join(format!("test_credentials_{}_{}.txt", hash, suffix));
        let mut file = fs::File::create(&path).unwrap();
        write!(file, "{}", content).unwrap();
        path
    }

    #[test]
    fn test_load_from_file_existing() {
        let path = create_temp_creds("existing", "FINNHUB_API_KEY=test_key_123\nOTHER=value\n");
        let creds = Credentials::load_from_file(path.to_str().unwrap());
        assert_eq!(creds.get("FINNHUB_API_KEY"), Some("test_key_123".to_string()));
        assert_eq!(creds.get("OTHER"), Some("value".to_string()));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_load_from_file_missing_key() {
        let path = create_temp_creds("missing", "FINNHUB_API_KEY=mykey\n");
        let creds = Credentials::load_from_file(path.to_str().unwrap());
        assert_eq!(creds.get("MISSING_KEY"), None);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_load_from_file_nonexistent() {
        let path = create_temp_creds("nonexistent", "NONEXISTENT_FILE");
        let creds = Credentials::load_from_file(path.to_str().unwrap());
        assert!(creds.is_empty());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_load_from_file_empty() {
        let path = create_temp_creds("empty", "");
        let creds = Credentials::load_from_file(path.to_str().unwrap());
        assert!(creds.is_empty());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_load_from_file_comments() {
        let path = create_temp_creds("comments", "# comment\n\nFINNHUB_API_KEY=abc123\n");
        let creds = Credentials::load_from_file(path.to_str().unwrap());
        assert_eq!(creds.get("FINNHUB_API_KEY"), Some("abc123".to_string()));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_credentials_file_exists_and_loaded_by_app() {
        use crate::providers::StockDataProvider;

        let cred_path = "resources/credentials.txt";
        let content = fs::read_to_string(cred_path).expect("credentials.txt must exist at startup");
        assert!(!content.is_empty(), "credentials.txt must not be empty");

        let creds = Credentials::load_from_file(cred_path);
        assert!(!creds.is_empty(), "credentials.txt must contain at least one key-value pair");

        let key = creds.get("FINNHUB_API_KEY");
        assert!(key.is_some(), "credentials.txt must contain FINNHUB_API_KEY");

        let api_key = key.unwrap();
        assert!(!api_key.is_empty(), "FINNHUB_API_KEY must not be empty");

        assert!(api_key.len() >= 20, "FINNHUB_API_KEY must be a valid Finnhub key (min 20 chars)");

        let provider = crate::providers::finnhub::FinnhubDataProvider::new(api_key);
        assert_eq!(provider.provider_name(), "FinnhubDataProvider");

        let tickers = provider.list_supported_tickers();
        assert!(!tickers.is_empty(), "Finnhub provider must support at least one ticker");
    }
}
