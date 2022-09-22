use workspaces::result::ExecutionFinalResult;

/// Asserts transaction failure due to `method` being `#[private]`.
pub fn assert_private_method_failure(res: ExecutionFinalResult, method: &str) {
    let err = res
        .into_result()
        .err()
        .expect("Transaction should have failed");
    let err = format!("{}", err);
    let must_contain = format!("Method {} is private", method);
    assert!(
        err.contains(&must_contain),
        "'{}' is not contained in '{}'",
        must_contain,
        err,
    );
}
