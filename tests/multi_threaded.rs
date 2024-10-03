fn main() {
    assert_eq!(safe_fork::fork_join(|| 42).unwrap(), 42);
}
