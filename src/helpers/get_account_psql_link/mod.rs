use password_encoder::{get_or_create_dev_key, decrypt_password};

pub fn get_account_psql_link(username: String, encrypted_db_password: String, database_url: String) -> String {
    let key = get_or_create_dev_key()
        .expect("Failed to get key");

    let account_db_password = decrypt_password(&encrypted_db_password, &key)
        .expect("Failed to decrypt user database password");
    
    format!(
        "postgres://{}:{}@{}/{}",
        username, account_db_password, database_url, username
    )
}