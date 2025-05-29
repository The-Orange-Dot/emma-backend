use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct AdminPool(pub Pool<Postgres>);

#[derive(Clone, Debug)]
pub struct AccountPools(pub HashMap<Uuid, Pool<Postgres>>);