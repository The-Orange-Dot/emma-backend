use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use uuid::Uuid;
use std::sync::{Arc, RwLock};
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct AdminPool(pub Pool<Postgres>);

#[derive(Clone, Debug)]
pub struct AccountPools(pub Arc<RwLock<HashMap<Uuid, PoolWrapper>>>);

#[derive(Clone, Debug)]
pub struct PoolWrapper {
    pub pool: Pool<Postgres>,
    pub last_used: Instant,
}
