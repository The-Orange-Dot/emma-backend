use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone)]
pub struct AdminPool(pub Pool<Postgres>);

#[derive(Clone)]
pub struct AccountPools(pub HashMap<Uuid, Pool<Postgres>>);