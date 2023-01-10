use near_sdk::serde::de::DeserializeOwned;
use std::cmp::PartialEq;
use std::fmt::Debug;
use workspaces::result::ExecutionFinalResult;

/// Asserts execution was successful and returned `()`.
pub fn assert_success_with_unit_return(res: ExecutionFinalResult) {
    assert!(res.is_success(), "Transaction should have succeeded");
    assert!(
        res.raw_bytes().unwrap().is_empty(),
        "Unexpected return value"
    );
}

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

pub fn assert_method_is_paused(res: ExecutionFinalResult) {
    let err = res
        .into_result()
        .err()
        .expect("Transaction should have failed");
    let err = format!("{}", err);
    let must_contain = "Pausable: Method is paused";
    assert!(
        err.contains(&must_contain),
        "Expected method to be paused, instead it failed with: {}",
        err
    );
}

pub fn assert_owner_update_failure(res: ExecutionFinalResult) {
    let err = res
        .into_result()
        .err()
        .expect("Transaction should have failed");
    let err = format!("{}", err);
    let must_contain = "Ownable: Only owner can update current owner";
    assert!(
        err.contains(&must_contain),
        "Expected failure due to caller not being owner, instead it failed with: {}",
        err
    );
}

/// Assert failure due to calling a method protected by `#[only]` without required permissions.
pub fn assert_ownable_permission_failure(res: ExecutionFinalResult) {
    let err = res
        .into_result()
        .err()
        .expect("Transaction should have failed");
    let err = format!("{}", err);
    let must_contain = "Method is private";
    assert!(
        err.contains(&must_contain),
        "Expected failure due to insufficient permissions, instead it failed with: {}",
        err
    );
}

/// Assert failure due to calling a method protected by `#[only(owner)]` from an account other than the
/// owner.
pub fn assert_only_owner_permission_failure(res: ExecutionFinalResult) {
    let err = res
        .into_result()
        .err()
        .expect("Transaction should have failed");
    let err = format!("{}", err);
    let must_contain = "Ownable: Method must be called from owner";
    assert!(
        err.contains(&must_contain),
        "Expected failure due to caller not being owner, instead it failed with: {}",
        err
    );
}

/// Asserts the execution of `res` failed and the error contains `must_contain`.
pub fn assert_failure_with(res: ExecutionFinalResult, must_contain: &str) {
    let err = res
        .into_result()
        .err()
        .expect("Transaction should have failed");
    let err = format!("{}", err);
    assert!(
        err.contains(must_contain),
        "The expected message\n'{}'\nis not contained in error\n'{}'",
        must_contain,
        err,
    );
}
