#[derive(Debug)]
pub enum TransactionType {
    PACS008,
    PACS002,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TransactionType::PACS002 => "pacs.002.001.12",
                TransactionType::PACS008 => "pacs.008.001.12",
            }
        )
    }
}

/// pacs.008.001.12
pub mod pacs008 {
    tonic::include_proto!("iso20022.pacs008");
}

/// pacs.002.001.12
pub mod pacs002 {
    tonic::include_proto!("iso20022.pacs002");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_pacs008() {
        let t = TransactionType::PACS008;
        assert_eq!(t.to_string(), "pacs.008.001.12");
    }

    #[test]
    fn display_pacs002() {
        let t = TransactionType::PACS002;
        assert_eq!(t.to_string(), "pacs.002.001.12");
    }
}
