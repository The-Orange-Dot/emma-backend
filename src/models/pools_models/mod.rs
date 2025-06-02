use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use uuid::Uuid;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct AdminPool(pub Pool<Postgres>);

// #[derive(Clone, Debug)]
// pub struct AccountPools(pub HashMap<Uuid, Pool<Postgres>>);

#[derive(Clone, Debug)]
pub struct AccountPools(pub Arc<RwLock<HashMap<Uuid, Pool<Postgres>>>>);
