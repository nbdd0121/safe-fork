fn main() {
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                assert!(safe_fork::fork_join(|| 42).is_err());
            })
            .join()
            .unwrap();
    });
}
