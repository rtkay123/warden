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

/// Pacs008 file descriptor
pub const ISO20022_FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("iso20022_descriptor");

/// pacs.008.001.12
pub mod pacs008 {
    tonic::include_proto!("iso20022.pacs008");
}

/// pacs.002.001.12
pub mod pacs002 {
    tonic::include_proto!("iso20022.pacs002");
}
