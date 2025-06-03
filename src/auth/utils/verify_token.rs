pub fn verify_token(token: &str) -> Result<String, &'static str> {
    // Implement your JWT or session token verification
    // Example with JWT:
    let validation = Validation::default();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &validation,
    ).map_err(|_| "Invalid token")?;

    Ok(token_data.claims.sub)
}