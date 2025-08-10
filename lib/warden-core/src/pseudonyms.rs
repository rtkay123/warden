pub mod account {
    tonic::include_proto!("pseudonyms.account");
}

pub mod entity {
    tonic::include_proto!("pseudonyms.entity");
}

pub mod transaction_relationship {
    tonic::include_proto!("pseudonyms.transaction_relationship");
}

pub mod account_holder {
    tonic::include_proto!("pseudonyms.account_holder");
}
