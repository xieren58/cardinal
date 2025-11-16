use search_cancel::{ACTIVE_SEARCH_VERSION, CancellationToken};
use std::{sync::atomic::Ordering, thread};

#[test]
fn multiple_tokens_cancelled_independently() {
    let t1 = CancellationToken::new(1);
    assert!(!t1.is_cancelled());
    let t2 = CancellationToken::new(2);
    assert!(t1.is_cancelled());
    assert!(!t2.is_cancelled());
    let t3 = CancellationToken::new(3);
    assert!(t2.is_cancelled());
    assert!(!t3.is_cancelled());
}

#[test]
fn concurrent_version_bump_cancels_prior_tokens() {
    // Spawn threads that bump the version; earlier tokens should observe cancellation quickly.
    let t_initial = CancellationToken::new(10);
    assert!(!t_initial.is_cancelled());

    let handles: Vec<_> = (11..21)
        .map(|v| {
            thread::spawn(move || {
                let _t = CancellationToken::new(v);
                // read back active version
                ACTIVE_SEARCH_VERSION.load(Ordering::SeqCst)
            })
        })
        .collect();

    for h in handles {
        let _ = h.join();
    }

    assert!(
        t_initial.is_cancelled(),
        "initial token should be cancelled after bumps"
    );
    let final_version = ACTIVE_SEARCH_VERSION.load(Ordering::SeqCst);
    assert!(final_version >= 20);
}
