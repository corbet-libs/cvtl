use cmmr::conformance::{TestClock, limits, run};
use cvtl::*;
use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
#[tokio::test]
async fn identical_suite_through_both_facade_variants() {
    let start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
        + 120_000;
    let clock = TestClock::new(start);
    let a: Volatile<_> = Volatile::memory(
        Scope::new("memory-a", "facade").unwrap(),
        limits(),
        clock.clone(),
    )
    .unwrap();
    let b: Volatile<_> = Volatile::memory(
        Scope::new("memory-b", "facade").unwrap(),
        limits(),
        clock.clone(),
    )
    .unwrap();
    assert!(a.warnings().is_empty());
    assert_eq!(a.clone().state(), State::Ready);
    assert_eq!(a.maintain().await.unwrap(), 0);
    run(&a, &b, &clock).await;
    assert_eq!(format!("{a:?}"), "Volatile(Memory)");
    let bad: Result<Volatile<_>, _> = Volatile::memory(
        Scope::new("bad", "facade").unwrap(),
        Limits { records: 0, ..limits() },
        clock.clone(),
    );
    assert!(matches!(bad, Err(Error::Invalid)));
    clock.set(start);
    let connection =
        redis::Client::open(std::env::var("VALKEY_URL").expect("disposable Valkey required"))
            .unwrap()
            .get_multiplexed_async_connection()
            .await
            .unwrap();
    let network = Arc::new(RedisNetwork::new(connection, Duration::from_secs(5)).unwrap());
    let a = Volatile::valkey(
        network.clone(),
        Scope::new("valkey-a", "facade").unwrap(),
        limits(),
        clock.clone(),
    )
    .await
    .unwrap();
    let b = Volatile::valkey(
        network.clone(),
        Scope::new("valkey-b", "facade").unwrap(),
        limits(),
        clock.clone(),
    )
    .await
    .unwrap();
    assert_eq!(a.warnings(), network.health().await.unwrap().warnings);
    assert_eq!(a.clone().state(), State::Ready);
    assert_eq!(a.maintain().await.unwrap(), 0);
    run(&a, &b, &clock).await;
    assert_eq!(format!("{a:?}"), "Volatile(Valkey)");
    assert!(matches!(
        Volatile::valkey(network, Scope::new("bad", "facade").unwrap(),
            Limits { records: 0, ..limits() }, clock).await,
        Err(Error::Invalid)
    ));
    // Closing one cloned facade closes its child; no independent facade state.
    assert_eq!(a.clone().state(), State::Closed);
}
