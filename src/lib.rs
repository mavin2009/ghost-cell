use std::cell::UnsafeCell;
//use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

/// A GhostCell allows temporary, scope-bound mutations to a value.  The underlying
/// data must implement `Copy` so that the original value can be efficiently stored
/// and restored.  Changes made within the GhostCell's scope are reverted when the
/// GhostCell is dropped.
pub struct GhostCell<'a, T: Clone> {
    original_data: &'a T,
    modified_data: UnsafeCell<Option<T>>, // Holds temporary modified data
}

impl<'a, T: Clone> GhostCell<'a, T> {
    // Create a new GhostCell from an immutable reference
    pub fn new(data: &'a T) -> Self {
        GhostCell {
            original_data: data,
            modified_data: UnsafeCell::new(None),
        }
    }

    // Consume the GhostCell and return the inner modified data if it exists, otherwise return the original data
    pub fn into_inner(self) -> T {
        if let Some(modified) = unsafe { (*self.modified_data.get()).take() } {
            modified
        } else {
            self.original_data.clone()
        }
    }

    // Revert the changes made to the data by dropping the modified data
    pub fn revert(&mut self) {
        unsafe {
            *self.modified_data.get() = None;
        }
    }

    // Borrow the data, showing either the original or the modified version
    pub fn get(&self) -> &T {
        if let Some(ref modified) = unsafe { &*self.modified_data.get() } {
            modified
        } else {
            self.original_data
        }
    }

    // Mutably borrow the data, creating a temporary mutable copy if necessary
    pub fn get_mut(&self) -> &mut T {
        if unsafe { &*self.modified_data.get() }.is_none() {
            // If no modification exists, clone the original data
            unsafe {
                *self.modified_data.get() = Some(self.original_data.clone());
            }
        }

        unsafe { (*self.modified_data.get()).as_mut().unwrap() }
    }
}

pub struct GhostBorrow<'b, T: Clone> {
    cell: &'b GhostCell<'b, T>,
}

impl<'b, T: Clone> Deref for GhostBorrow<'b, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.cell.get()
    }
}

pub struct GhostBorrowMut<'b, T: Clone> {
    cell: &'b mut GhostCell<'b, T>,
}

impl<'b, T: Clone> Deref for GhostBorrowMut<'b, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.cell.get()
    }
}

impl<'b, T: Clone> DerefMut for GhostBorrowMut<'b, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.cell.get_mut()
    }
}

// When the GhostCell is dropped, changes are discarded automatically.
impl<'a, T: Clone> Drop for GhostCell<'a, T> {
    fn drop(&mut self) {
        self.revert(); // Drop the modified data, reverting any changes.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_revert() {
        let data = 10;
        {
            let mut ghost = GhostCell::new(&data);
            *ghost.get_mut() = 20;
            assert_eq!(*ghost.get(), 20);
        } // GhostCell is dropped here, and data should revert
        assert_eq!(data, 10); // Original value should be restored
    }

    #[test]
    fn test_revert_mid_scope() {
        let data = vec![1, 2, 3];
        {
            let mut ghost = GhostCell::new(&data);
            ghost.get_mut().push(4);
            ghost.revert(); // Revert midway through mutation
            assert_eq!(ghost.get().len(), 3); // Should revert to original length
            ghost.get_mut().push(5);
        }
        assert_eq!(data, vec![1, 2, 3]); // Original data should not change
    }

    #[test]
    fn test_into_inner_no_revert() {
        let data = vec![1, 2, 3];
        let inner;
        {
            let mut ghost = GhostCell::new(&data);
            ghost.get_mut().push(4); // Modify the data inside the GhostCell
            inner = ghost.into_inner(); // Take ownership of the modified data
        }
        assert_eq!(inner, vec![1, 2, 3, 4]); // The modified value should be `[1, 2, 3, 4]`
        assert_eq!(data, vec![1, 2, 3]); // Ensure the original data remains `[1, 2, 3]`
    }

    #[test]
    fn test_multiple_reverts() {
        let data = vec![1, 2, 3];
        {
            let mut ghost = GhostCell::new(&data);
            ghost.get_mut().push(4);
            ghost.revert(); // Revert first mutation
            ghost.get_mut().push(5);
            ghost.revert(); // Revert second mutation
        }
        assert_eq!(data, vec![1, 2, 3]); // Original data should be restored
    }

    #[test]
    fn test_with_string_mutation() {
        let data = String::from("hello");
        {
            let mut ghost = GhostCell::new(&data);
            ghost.get_mut().push_str(" world");
            assert_eq!(*ghost.get(), "hello world"); // Check the modified string
        }
        assert_eq!(data, "hello"); // Original string should remain unchanged
    }

    #[test]
    fn test_with_copy_type() {
        let data = 10;
        {
            let mut ghost = GhostCell::new(&data);
            *ghost.get_mut() = 20;
            assert_eq!(*ghost.get(), 20); // Mutated value
        }
        assert_eq!(data, 10); // Reverted to original value
    }

    #[test]
    fn test_with_needs_drop_type() {
        let data = vec![1, 2, 3];
        {
            let mut ghost = GhostCell::new(&data);
            ghost.get_mut().push(4);
            assert_eq!(*ghost.get(), vec![1, 2, 3, 4]); // Mutated vector
        }
        assert_eq!(data, vec![1, 2, 3]); // Reverted to original vector
    }

    #[test]
    fn test_nested_borrows() {
        let data = vec![1, 2, 3];
        {
            let mut ghost = GhostCell::new(&data);
            let mut borrowed = ghost.get_mut();
            borrowed.push(4);
            assert_eq!(borrowed.len(), 4); // Ensure the borrow mutates
            borrowed.pop(); // Modify through the mutable borrow
        }
        assert_eq!(data, vec![1, 2, 3]); // Reverted to original vector
    }

    #[test]
    fn test_multiple_ghost_cells() {
        let data1 = vec![1, 2, 3];
        let data2 = vec![4, 5, 6];

        {
            let mut ghost1 = GhostCell::new(&data1);
            let mut ghost2 = GhostCell::new(&data2);

            ghost1.get_mut().push(4);
            ghost2.get_mut().push(7);
        }
        assert_eq!(data1, vec![1, 2, 3]); // Must revert
        assert_eq!(data2, vec![4, 5, 6]); // Must revert
    }

    #[test]
    fn test_needs_drop_after_into_inner() {
        let data = vec![1, 2, 3];
        {
            let ghost = GhostCell::new(&data);
            let _inner = ghost.into_inner(); // Take ownership
        }
        // Ensure original data is unaffected after the GhostCell is dropped
        assert_eq!(data.len(), 3);
    }

    #[test]
    fn test_no_mutation_revert() {
        let data = vec![1, 2, 3];
        {
            let ghost = GhostCell::new(&data);
            // No mutation performed
        }
        assert_eq!(data, vec![1, 2, 3]); // Original data should remain unchanged
    }

    #[test]
    fn test_multiple_borrow_same_scope() {
        let data = vec![1, 2, 3];
        {
            let mut ghost = GhostCell::new(&data);
            let borrowed1 = ghost.get();
            let borrowed2 = ghost.get();
            assert_eq!(borrowed1.len(), 3);
            assert_eq!(borrowed2.len(), 3);
        }
        assert_eq!(data, vec![1, 2, 3]); // No mutation, data should remain unchanged
    }

    #[test]
    fn test_mutation_after_revert() {
        let data = vec![1, 2, 3];
        {
            let mut ghost = GhostCell::new(&data);
            ghost.get_mut().push(4);
            ghost.revert();
            assert_eq!(ghost.get().len(), 3); // Reverted to original length
            ghost.get_mut().push(5); // New mutation after revert
            assert_eq!(ghost.get().len(), 4); // Should reflect the new mutation
        }
        assert_eq!(data, vec![1, 2, 3]); // Original data should not change
    }

    #[test]
    fn test_borrow_and_mut_borrow() {
        let data = vec![1, 2, 3];
        {
            let mut ghost = GhostCell::new(&data);
            let borrowed = ghost.get(); // Immutable borrow
            assert_eq!(borrowed.len(), 3);

            let mut borrowed_mut = ghost.get_mut(); // Mutable borrow
            borrowed_mut.push(4);
            assert_eq!(borrowed_mut.len(), 4);
        }
        assert_eq!(data, vec![1, 2, 3]); // Original data should remain unchanged
    }

    #[test]
    fn test_nested_ghost_cell() {
        let data1 = vec![1, 2, 3];
        let data2 = vec![4, 5, 6];
        {
            let mut outer_ghost = GhostCell::new(&data1);
            let mut inner_ghost = GhostCell::new(&data2);
            inner_ghost.get_mut().push(7);
            outer_ghost.get_mut().push(4);

            assert_eq!(inner_ghost.get(), &vec![4, 5, 6, 7]);
            assert_eq!(outer_ghost.get(), &vec![1, 2, 3, 4]);
        }
        assert_eq!(data1, vec![1, 2, 3]); // Must revert
        assert_eq!(data2, vec![4, 5, 6]); // Must revert
    }
}
