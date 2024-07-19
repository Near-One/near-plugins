use near_sdk::serde::de::DeserializeOwned;
use near_sdk::Duration;
use near_workspaces::network::Sandbox;
use near_workspaces::result::{ExecutionFinalResult, ExecutionOutcome};
use near_workspaces::{AccountId, Block, Worker};
use std::cmp::PartialEq;
use std::fmt::Debug;
use std::str::FromStr;

/// Converts `account_id` to a `near_sdk::AccountId` and panics on failure.
///
/// Only available in tests, hence favoring simplicity over efficiency.
#[must_use]
pub fn as_sdk_account_id(account_id: &AccountId) -> near_sdk::AccountId {
    near_sdk::AccountId::from_str(account_id.as_str())
        .expect("Conversion to near_sdk::AccountId should succeed")
}

/// Convenience function to create a new `near_sdk::Duration`. Panics if the conversion fails.
#[must_use]
pub fn sdk_duration_from_secs(seconds: u64) -> Duration {
    std::time::Duration::from_secs(seconds)
        .as_nanos()
        .try_into()
        .expect("Conversion from std Duration to near_sdk Duration should succeed")
}

/// Asserts execution was successful and returned `()`.
pub fn assert_success_with_unit_return(res: ExecutionFinalResult) {
    match res.into_result() {
        Ok(res) => {
            assert!(
                res.raw_bytes().unwrap().is_empty(),
                "Unexpected return value"
            );
        }
        Err(err) => panic!("Transaction should have succeeded but failed with: {err}"),
    }
}

/// Asserts execution was successful and returned the `expected` value.
pub fn assert_success_with<T>(res: ExecutionFinalResult, expected: T)
where
    T: DeserializeOwned + PartialEq + Debug + Copy,
{
    let actual = res
        .into_result()
        .expect("Transaction should have succeeded")
        .json::<T>()
        .expect("Return value should be deserializable");
    assert_eq!(actual, expected);
}

/// Asserts transaction failure due `MethodNotFound` error.
pub fn assert_method_not_found_failure(res: ExecutionFinalResult) {
    assert_failure_with(res, "Action #0: MethodResolveError(MethodNotFound)");
}

/// Asserts transaction failure due to `method` being `#[private]`.
pub fn assert_private_method_failure(res: ExecutionFinalResult, method: &str) {
    let err = res
        .into_result()
        .expect_err("Transaction should have failed");
    let err = format!("{err}");
    let must_contain = format!("Method {method} is private");
    assert!(
        err.contains(&must_contain),
        "'{must_contain}' is not contained in '{err}'",
    );
}

/// Asserts transaction failure due to insufficient `AccessControllable` (ACL)
/// permissions.
pub fn assert_insufficient_acl_permissions(
    res: ExecutionFinalResult,
    method: &str,
    _allowed_roles: &[String],
) {
    let err = res
        .into_result()
        .expect_err("Transaction should have failed");
    let err = format!("{err}");

    // TODO fix escaping issue to also verify second sentence of the error
    // Using `format!` here it'll be: Requires one of these roles: ["LevelA", "LevelB"]
    // However, roles contained in `err` are escaped, i.e. [\"LevelA\", \"LevelB\"]
    let must_contain =
        format!("Insufficient permissions for method {method} restricted by access control.");

    assert!(
        err.contains(&must_contain),
        "'{must_contain}' is not contained in '{err}'",
    );
}

pub fn assert_method_is_paused(res: ExecutionFinalResult) {
    let err = res
        .into_result()
        .expect_err("Transaction should have failed");
    let err = format!("{err}");
    let must_contain = "Pausable: Method is paused";
    assert!(
        err.contains(must_contain),
        "Expected method to be paused, instead it failed with: {err}"
    );
}

pub fn assert_pausable_escape_hatch_is_closed(res: ExecutionFinalResult, feature: &str) {
    let must_contain = format!("Pausable: {feature} must be paused to use this function");
    assert_failure_with(res, &must_contain);
}

pub fn assert_owner_update_failure(res: ExecutionFinalResult) {
    let err = res
        .into_result()
        .expect_err("Transaction should have failed");
    let err = format!("{err}");
    let must_contain = "Ownable: Only owner can update current owner";
    assert!(
        err.contains(must_contain),
        "Expected failure due to caller not being owner, instead it failed with: {err}"
    );
}

/// Assert failure due to calling a method protected by `#[only]` without required permissions.
pub fn assert_ownable_permission_failure(res: ExecutionFinalResult) {
    let err = res
        .into_result()
        .expect_err("Transaction should have failed");
    let err = format!("{err}");
    let must_contain = "Method is private";
    assert!(
        err.contains(must_contain),
        "Expected failure due to insufficient permissions, instead it failed with: {err}"
    );
}

/// Assert failure due to calling a method protected by `#[only(owner)]` from an account other than the
/// owner.
pub fn assert_only_owner_permission_failure(res: ExecutionFinalResult) {
    let err = res
        .into_result()
        .expect_err("Transaction should have failed");
    let err = format!("{err}");
    let must_contain = "Ownable: Method must be called from owner";
    assert!(
        err.contains(must_contain),
        "Expected failure due to caller not being owner, instead it failed with: {err}"
    );
}

/// Asserts the execution of `res` failed and the error contains `must_contain`.
pub fn assert_failure_with(res: ExecutionFinalResult, must_contain: &str) {
    let err = res
        .into_result()
        .expect_err("Transaction should have failed");
    let err = format!("{err}");
    assert!(
        err.contains(must_contain),
        "The expected message\n'{must_contain}'\nis not contained in error\n'{err}'"
    );
}

pub fn assert_access_key_not_found_error(
    res: near_workspaces::Result<ExecutionFinalResult, near_workspaces::error::Error>,
) {
    let err = res.expect_err("Transaction should not have been executed");

    // Debug formatting is required to get the full error message containing `AccessKeyNotFound`.
    // Assume that is acceptable since this function is available only in tests.
    let err = format!("{err:?}");
    let must_contain = "InvalidAccessKeyError";

    assert!(
        err.contains(must_contain),
        "The expected message\n'{must_contain}'\nis not contained in error\n'{err}'"
    );
}

/// Returns the block timestamp in nanoseconds. Panics on failure.
async fn block_timestamp(worker: &Worker<Sandbox>) -> u64 {
    worker
        .view_block()
        .await
        .expect("Should view block")
        .timestamp()
}

/// Returns the block in which a transaction or receipt was included.
pub async fn get_transaction_block(
    worker: &Worker<Sandbox>,
    result: &ExecutionOutcome,
) -> near_workspaces::Result<Block> {
    let block_hash = result.block_hash;
    worker.view_block().block_hash(block_hash).await
}

/// [Time travels] `worker` forward by at least `duration`. This is achieved by a very naive
/// approach: fast forward blocks until `duration` has passed. Keeping it simple since this function
/// is available only in tests.
///
/// Due to this approach, it is recommended to pass only relatively small values as `duration`. Fast
/// forwarding provided by this function is reasonly fast in our tests for durations that correspond
/// to less than 100 seconds.
///
/// [Time travels]: https://github.com/near/near-workspaces-rs#time-traveling
pub async fn fast_forward_beyond(worker: &Worker<Sandbox>, duration: Duration) {
    let initial_timestamp = block_timestamp(worker).await;

    // Estimating a number of blocks to skip based on `duration` and calling `fast_forward` only
    // once seems more efficient. However, that leads to jittery tests as `fast_forward` may _not_
    // forward the block timestamp significantly.
    while block_timestamp(worker).await - initial_timestamp < duration {
        worker
            .fast_forward(1)
            .await
            .expect("Fast forward should succeed");
    }
}
