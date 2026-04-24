#[derive(Default, Clone)]
pub struct Stack<T> {
    content : Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack {
            content: Vec::new()
        }
    }
    pub fn push(&mut self, elem : T) {
        self.content.push(elem);
    }
    
    pub fn pop(&mut self) -> Option<T> {
        self.content.pop()
    }
    
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
    
    pub fn peek(&self) -> Option<&T> {
        self.content.last()
    }
    
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.content.last_mut()
    }
    
    pub fn push_all<I: IntoIterator::<Item = T>>(&mut self, it : I) {
        for elem in it {
            self.push(elem);
        }
    }
    
    pub fn len(&self) -> usize {
        self.content.len()
    }
}

impl<T> std::iter::FromIterator<T> for Stack<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Stack<T> {
        let mut res = Stack::new();
        for elem in iter {
            res.push(elem);
        }
        res
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stack() {
        let mut stack = Stack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        
        // peek(), .pop() none when stack empty
        assert!(stack.peek().is_none());
        assert!(stack.pop().is_none());
        
        stack.push(1);
        assert!(!stack.is_empty());
        assert_eq!(stack.len(), 1);
        
        assert_eq!(*stack.peek().unwrap(), 1);
        assert!(!stack.is_empty());
        assert_eq!(stack.len(), 1);
        
        assert_eq!(stack.pop().unwrap(), 1);
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        
        let list = [1, 2, 3, 4, 5];
        stack.push_all(list);
        assert!(!stack.is_empty());
        assert_eq!(stack.len(), 5);
        
        let mut list_popped  = Vec::new();
        while let Some(elem) = stack.pop() {
            list_popped.push(elem);
        }
        
        list_popped.reverse();
        assert_eq!(Vec::from(list), list_popped);
    }
    
    #[test]
    fn test_stack_from_iterable() {
        let list = ['a', 'b', 'c', 'd', 'e'];
        
        let mut stack1  = Stack::from_iter(list);
        let mut stack2 = Stack::new();
        stack2.push_all(list);
        
        assert_eq!(stack1.len(), stack2.len());
        while let (Some(elem1), Some(elem2)) = (stack1.pop(), stack2.pop()) {
            assert_eq!(elem1, elem2);
        }
    }
}