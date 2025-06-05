use std::time::Duration;
use crate::AccountPools;

pub async fn start_pool_cleanup_task(account_pools: AccountPools, cleanup_interval: Duration, idle_timeout: Duration) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(cleanup_interval); 
        loop {
            interval.tick().await;
            account_pools.cleanup_idle_pools(idle_timeout); 
        }
    });
}