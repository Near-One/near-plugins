use near_sdk::serde::de::DeserializeOwned;
use std::cmp::PartialEq;
use std::fmt::Debug;
use workspaces::result::ExecutionFinalResult;

/// Asserts execution was successful and returned the `expected` value.
pub fn assert_success_with<T>(res: ExecutionFinalResult, expected: T)
where
    T: DeserializeOwned + PartialEq + Debug,
{
    let actual = res
        .into_result()
        .expect("Transaction should have succeeded")
        .json::<T>()
        .expect("Return value should be deserializable");
    assert_eq!(actual, expected);
}

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

/// Asserts transaction failure due to insufficient `AccessControllable` (ACL)
/// permissions.
pub fn assert_insufficient_acl_permissions(
    res: ExecutionFinalResult,
    method: &str,
    _allowed_roles: Vec<String>,
) {
    let err = res
        .into_result()
        .err()
        .expect("Transaction should have failed");
    let err = format!("{}", err);

    // TODO fix escaping issue to also verify second sentence of the error
    // Using `format!` here it'll be: Requires one of these roles: ["LevelA", "LevelB"]
    // However, roles contained in `err` are escaped, i.e. [\"LevelA\", \"LevelB\"]
    let must_contain = format!(
        "Insufficient permissions for method {} restricted by access control.",
        method,
    );

    assert!(
        err.contains(&must_contain),
        "'{}' is not contained in '{}'",
        must_contain,
        err,
    );
}
