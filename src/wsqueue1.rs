//! A work-stealing queue implemented with a double-linked list
//!
//! Heavily based on a safe doubly linked deque from "Learning Rust
//! With Entirely Too Many Linked Lists.

use std::cell::RefCell;
use std::rc::Rc;

/// A double-ended queue implemented with a double-linked list.
///
/// [`push`]: #method.push
/// [`pop`]: #method.pop
/// [`steal`]: #method.steal
#[derive(Debug)]
pub struct WsQueue<T> {
    head: Link<T>,
    tail: Link<T>,
    length: usize,
}

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

#[derive(Debug)]
struct Node<T> {
    elem: T,
    prev: Link<T>,
    next: Link<T>,
}

impl<T> Node<T> {
    fn new(elem: T) -> Rc<RefCell<Node<T>>> {
        Rc::new(RefCell::new(Node {
            elem: elem,
            prev: None,
            next: None,
        }))
    }
}

unsafe impl<T: Send> Send for WsQueue<T> {}

impl<T> WsQueue<T> {
    /// Creates an empty `WsQueue`.
    ///
    /// # Examples
    ///
    /// ```
    /// let wsq: WsQueue<i32> = WsQueue::new();
    /// ```
    pub fn new() -> Self {
        WsQueue {
            head: None,
            tail: None,
            length: 0,
        }
    }

    /// Enqueues an element
    ///
    /// # Examples
    /// ```
    /// let mut wsq = WsQueue::new();
    ///
    /// wsq.push(1);
    /// ```
    pub fn push(&mut self, elem: T) {
        let new_head = Node::new(elem);
        match self.head.take() {
            Some(old_head) => {
                old_head.borrow_mut().prev = Some(new_head.clone());
                new_head.borrow_mut().next = Some(old_head);
                self.head = Some(new_head);
            }
            None => {
                self.tail = Some(new_head.clone());
                self.head = Some(new_head);
            }
        }
        self.length += 1;
    }

    /// Steals an element from the beginning of the queue
    ///
    /// # Examples
    /// ```
    /// let mut wsq = WsQueue::new();
    ///
    /// wsq.push(1);
    /// wsq.push(2);
    /// wsq.push(3);
    ///
    /// assert_eq!(wsq.steal(), Some(3));
    /// ```
    pub fn steal(&mut self) -> Option<T> {
        self.head.take().map(|old_head| {
            match old_head.borrow_mut().next.take() {
                Some(new_head) => {
                    new_head.borrow_mut().prev.take();
                    self.head = Some(new_head);
                }
                None => {
                    self.tail.take();
                }
            }
            self.length -= 1;
            Rc::try_unwrap(old_head).ok().unwrap().into_inner().elem
        })
    }

    /// Dequeues the element from the end of the queue
    ///
    /// # Examples
    ///
    /// ```
    /// let mut wsq = WsQueue::new();
    ///
    /// wsq.push(1);
    /// wsq.push(2);
    /// wsq.push(3);
    ///
    /// assert_eq!(wsq.pop(), Some(1));
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        self.tail.take().map(|old_tail| {
            match old_tail.borrow_mut().prev.take() {
                Some(new_tail) => {
                    new_tail.borrow_mut().next.take();
                    self.tail = Some(new_tail);
                }
                None => {
                    self.head.take();
                }
            }
            self.length -= 1;
            Rc::try_unwrap(old_tail).ok().unwrap().into_inner().elem
        })
    }

    /// Returns the number of enqueued elements
    ///
    /// # Examples
    /// ```
    /// let mut wsq = WsQueue::new();
    ///
    /// wsq.push(1);
    /// wsq.push(2);
    /// wsq.push(3);
    ///
    /// assert_eq!(wsq.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.length
    }
}

#[cfg(test)]
mod test {
    use super::WsQueue;

    #[test]
    fn basics() {
        let mut list = WsQueue::new();

        assert_eq!(list.pop(), None);
        assert_eq!(list.len(), 0);

        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.len(), 3);

        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));
        assert_eq!(list.len(), 1);
    }
}
