use slab::Slab;

/// Type alias for event handlers.
///
/// Each handler is a boxed function that takes a reference to event arguments.
pub type EventHandler<'a, TEventArgs> = Box<dyn Fn(&TEventArgs) + 'a>;

/// An event that allows multiple handlers to be attached.
///
/// This structure is similar to the C# `event` pattern.
/// You can add, remove, and invoke handlers in order.
///
/// Internally, it uses a `Slab` for efficient handler storage and indexing.
///
/// # Examples
///
/// ```
/// use event_rs::Event;
///
/// #[derive(Debug, PartialEq)]
/// struct EventArgs<'a> {
///     id: u32,
///     message: &'a str,
/// }
///
/// let mut event = Event::<EventArgs>::new();
/// event.add(|args| {
///     println!("Event invoked with args: {:?}", args);
///     assert_eq!(args, &EventArgs {id: 0, message: ""});
/// });
///
/// let arg = EventArgs {id: 0, message: ""};
/// event.invoke(&arg);
/// ```
pub struct Event<'a, TEventArgs> {
    handlers: Slab<EventHandler<'a, TEventArgs>>,
}

impl<'a, TEventArgs> Default for Event<'a, TEventArgs> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, TEventArgs> Event<'a, TEventArgs> {
    /// Creates a new, empty Event
    ///
    /// # Examples
    ///
    /// ```
    /// use event_rs::Event;
    ///
    /// let mut event: Event<()> = Event::new();
    /// ```
    pub fn new() -> Self {
        Self {
            handlers: Slab::new(),
        }
    }

    /// Adds an event handler to the event.
    ///
    /// The handler should be a closure that accepts a reference to the event arguments
    /// and returns nothing. The closure will be executed when the event is invoked.
    ///
    /// Returns a handle that can be used to remove the handler later.
    ///
    /// # Examples
    ///
    /// ```
    /// use event_rs::Event;
    ///
    /// let mut event = Event::<()>::new();
    /// let handle = event.add(|args| {
    ///     println!("Event invoked");
    /// });
    /// ```
    pub fn add<F>(&mut self, handler: F) -> usize
    where
        F: Fn(&TEventArgs) + 'a,
    {
        self.handlers.insert(Box::new(handler))
    }

    /// Removes an event handler using its handle.
    ///
    /// Returns `true` if the handler was found and removed, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use event_rs::Event;
    ///
    /// let mut event = Event::<()>::new();
    /// let handle = event.add(|args| {
    ///     println!("Event invoked");
    /// });
    ///
    /// assert!(event.remove(handle));
    /// assert!(!event.remove(handle)); // Already removed
    /// ```
    pub fn remove(&mut self, handle: usize) -> bool {
        self.handlers.try_remove(handle).is_some()
    }

    /// Removes all event handlers.
    ///
    /// # Examples
    ///
    /// ```
    /// use event_rs::Event;
    ///
    /// let mut event = Event::<()>::new();
    /// event.add(|args| { println!("Handler 1"); });
    /// event.add(|args| { println!("Handler 2"); });
    ///
    /// event.clear(); // Remove all handlers
    /// ```
    pub fn clear(&mut self) {
        self.handlers.clear();
    }

    /// Invokes all event handlers sequentially (one after another).
    ///
    /// Each handler is awaited before the next one is executed.
    ///
    /// # Examples
    ///
    /// ```
    /// use event_rs::Event;
    ///
    /// let mut event = Event::new();
    /// event.add(|args| { println!("Handler 1"); });
    /// event.add(|args| { println!("Handler 2"); });
    ///
    /// event.invoke(&()); // Execute all handlers in order
    /// ```
    pub fn invoke(&self, arg: &TEventArgs) {
        for (_, handler) in self.handlers.iter() {
            handler(arg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_invoke() {
        let counter = Rc::new(RefCell::new(0));
        let mut event = Event::new();

        event.invoke(&());

        event.add(|_| {
            *counter.borrow_mut() += 1;
        });

        event.add(|_| {
            *counter.borrow_mut() += 1;
        });

        event.invoke(&());
        event.invoke(&());
        assert_eq!(*counter.borrow(), 4);
    }

    #[test]
    fn test_remove_handler() {
        let counter = Rc::new(RefCell::new(0));
        let mut event = Event::new();

        let handle = event.add(|_| {
            *counter.borrow_mut() += 1;
        });

        assert!(event.remove(handle));
        event.invoke(&());

        assert_eq!(*counter.borrow(), 0);
    }

    #[test]
    fn test_clear_handlers() {
        let counter = Rc::new(RefCell::new(0));
        let mut event = Event::new();

        for _ in 0..5 {
            event.add(|_| {
                *counter.borrow_mut() += 1;
            });
        }

        event.clear();
        event.invoke(&());

        assert_eq!(*counter.borrow(), 0);
    }

    #[test]
    fn test_remove_handler_twice() {
        let counter = Rc::new(RefCell::new(0));
        let mut event = Event::new();

        let handle = event.add(|_| {
            *counter.borrow_mut() += 1;
        });

        assert!(event.remove(handle));
        assert!(!event.remove(handle));

        event.invoke(&());
        assert_eq!(*counter.borrow(), 0);
    }
}
