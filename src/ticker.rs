#[derive(Debug)]
pub struct Ticker {
    pub exchange: Option<String>,
    pub symbol: String,
}

impl From<&str> for Ticker {
    fn from(s: &str) -> Self {
        let s = s.trim();

        let parts: Vec<_> = s.splitn(2, ':').collect();
        if parts.len() == 2 {
            Self {
                exchange: Some(parts[0].trim().to_uppercase().to_string()),
                symbol: parts[1].trim().to_uppercase().to_string(),
            }
        } else {
            let exchange = if s.starts_with("600")
                || s.starts_with("601")
                || s.starts_with("603")
                || s.starts_with("688")
            {
                Some("SSE")
            } else if s.starts_with("000") || s.starts_with("002") || s.starts_with("300") {
                Some("SZSE")
            } else {
                None
            };

            Self {
                exchange: exchange.map(|s| s.to_string()),
                symbol: s.to_uppercase().to_string(),
            }
        }
    }
}
