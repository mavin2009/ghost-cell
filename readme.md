# Scope-cell: Temporary, Scope-Bound, Reverting Mutations in Rust

`Scope-cell` provides a mechanism for temporary, scope-bound mutability of immutable data in Rust. It allows for non-permanent changes within a defined scope while ensuring the original data remains unaffected. This is useful in scenarios where you need temporary mutation without ownership or long-term modification of the underlying data.

Key features include:

* **Temporary mutation**: Enables mutation of data within a scope without requiring ownership.
* **Automatic reversion**: All changes are discarded when the scope ends, reverting to the original state.
* **Low-overhead mutability**: Only clone when necessary to ensure efficient memory use.
* **Rust safety guarantees**: Operates within Rust’s strict borrowing and ownership rules to ensure thread safety and data integrity.
* **Compile-time safety**: No runtime checks are required; Rust's type system ensures safe use.


## Key Features

* **Temporary mutation:** Modify immutable data within a scope, and ensure the data reverts to its original state once the scope is dropped.
* **Automatic reversion:** Changes are discarded upon scope exit, making it easy to test or simulate changes without committing them.
* **No runtime overhead:** Reversion and data integrity are managed with compile-time guarantees.
* **Non-ownership, minimal cloning:** Only clone when necessary to make modifications, optimizing memory usage.
* **Safety with Rust’s borrowing rules:** Safely borrow and mutate data within the strict rules of Rust's ownership and borrowing system.

## Use Cases

`Scope-cell` is useful in scenarios where you need to temporarily mutate immutable data without affecting the original state:

* **Simulation**: Simulate temporary changes to immutable data and automatically roll back.
* **Transactional systems**: Experiment with temporary changes before committing.
* **Testing**: Make non-persistent changes during tests that should be automatically undone.
* **Concurrency**: Share data across threads with safe, temporary mutability.

## Example Usage

```rust
use Scope_cell::ScopeCell;

fn main() {
    let immutable_data = vec![1, 2, 3];
    {
        let mut Scope = ScopeCell::new(&immutable_data);
        Scope.get_mut().push(4); // Temporarily mutate the data
        assert_eq!(Scope.get().len(), 4); // Verify the change within the scope
    } // Changes revert here
    assert_eq!(immutable_data.len(), 3); // The original data remains unchanged
}

## How it Works

ScopeCell operates by storing a reference to the original data and optionally cloning it into a temporary mutable version. When the ScopeCell is dropped, any changes made are discarded, and the original data remains unaffected.

Key methods:
* **get()** - Borrow the data (either original or modified).
* **get_mut()** - Mutably borrow the data, cloning the original if necessary.
* **revert()** - Explicitly discard any changes, restoring the original data.

## Installation

To use Scope-cell add the following to your Cargo.toml

```toml
[dependencies]
Scope-cell = "0.1.0"

## License
This project is licensed under the MIT License.
