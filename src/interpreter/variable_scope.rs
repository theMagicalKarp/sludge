use crate::interpreter::builtins;
use crate::interpreter::value::NamedBuiltin;
use crate::interpreter::value::Value;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct VariableScope {
    variables: RefCell<HashMap<String, Value>>,
    parent: Option<Rc<VariableScope>>,
}

impl VariableScope {
    /// Create a new root scope.
    pub fn new() -> Rc<Self> {
        let new_list = Rc::new(NamedBuiltin {
            name: "list",
            this: Value::Null,
            f: builtins::list::new,
        });

        let new_dict = Rc::new(NamedBuiltin {
            name: "dict",
            this: Value::Null,
            f: builtins::dict::dict,
        });

        let new_set = Rc::new(NamedBuiltin {
            name: "set",
            this: Value::Null,
            f: builtins::set::set,
        });

        Rc::new(Self {
            variables: RefCell::new(HashMap::from([
                (String::from("list"), Value::BuiltinFn(new_list)),
                (String::from("dict"), Value::BuiltinFn(new_dict)),
                (String::from("set"), Value::BuiltinFn(new_set)),
            ])),
            parent: None,
        })
    }

    /// Create a child scope that *references* the given parent.
    pub fn branch(parent: &Rc<Self>) -> Rc<Self> {
        Rc::new(Self {
            variables: RefCell::new(HashMap::new()),
            parent: Some(Rc::clone(parent)),
        })
    }

    /// Look up a name, walking up through parents if needed.
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.variables.borrow().get(name) {
            return Some(v.clone());
        }
        match &self.parent {
            Some(p) => p.get(name),
            None => None,
        }
    }

    /// Declare/overwrite in *this* scope only.
    pub fn declare(&self, name: String, value: Value) -> Option<Value> {
        self.variables.borrow_mut().insert(name, value)
    }

    /// Set in the nearest scope where it exists; otherwise bubble up.
    pub fn set(&self, name: String, value: Value) -> Option<Value> {
        if self.variables.borrow().contains_key(&name) {
            self.variables.borrow_mut().insert(name, value)
        } else if let Some(p) = &self.parent {
            p.set(name, value)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let root = VariableScope::new();
        root.declare("foo".to_string(), Value::Int32(1));
        root.declare("bar".to_string(), Value::Int32(2));

        assert_eq!(root.get("foo"), Some(Value::Int32(1)));
        assert_eq!(root.get("bar"), Some(Value::Int32(2)));
        assert_eq!(root.get("baz"), None);
    }

    #[test]
    fn test_branch() {
        let root = VariableScope::new();
        root.declare("foo".to_string(), Value::Int32(1));
        root.declare("bar".to_string(), Value::Int32(2));

        let b1 = VariableScope::branch(&root);
        assert_eq!(root.get("foo"), Some(Value::Int32(1)));
        assert_eq!(b1.get("foo"), Some(Value::Int32(1)));

        // Child shadowing should not affect parent
        b1.declare("foo".to_string(), Value::Int32(42));
        assert_eq!(root.get("foo"), Some(Value::Int32(1)));
        assert_eq!(b1.get("foo"), Some(Value::Int32(42)));

        // Re-declare in parent shouldn’t change child’s shadowing
        root.declare("foo".to_string(), Value::Int32(51));
        assert_eq!(root.get("foo"), Some(Value::Int32(51)));
        assert_eq!(b1.get("foo"), Some(Value::Int32(42)));

        // New parent var visible in child
        root.declare("hello".to_string(), Value::Int32(109));
        assert_eq!(root.get("hello"), Some(Value::Int32(109)));
        assert_eq!(b1.get("hello"), Some(Value::Int32(109)));
    }

    #[test]
    fn test_set_bubbles_up_to_parent_when_not_shadowed() {
        let root = VariableScope::new();
        root.declare("x".to_string(), Value::Int32(10));

        let child = VariableScope::branch(&root);

        // child doesn't have x; set should bubble up and update parent
        let prev = child.set("x".to_string(), Value::Int32(99));
        assert_eq!(prev, Some(Value::Int32(10)));
        assert_eq!(root.get("x"), Some(Value::Int32(99)));
        assert_eq!(child.get("x"), Some(Value::Int32(99)));
    }

    #[test]
    fn test_set_updates_nearest_binding_when_shadowed() {
        let root = VariableScope::new();
        root.declare("x".to_string(), Value::Int32(10));
        let child = VariableScope::branch(&root);
        child.declare("x".to_string(), Value::Int32(20));

        // set should hit child's binding, not parent's
        let prev = child.set("x".to_string(), Value::Int32(30));
        assert_eq!(prev, Some(Value::Int32(20)));
        assert_eq!(child.get("x"), Some(Value::Int32(30)));
        assert_eq!(root.get("x"), Some(Value::Int32(10)));
    }

    #[test]
    fn test_set_returns_none_when_name_absent_anywhere() {
        let root = VariableScope::new();
        let child = VariableScope::branch(&root);

        let res = child.set("nope".to_string(), Value::Int32(1));
        assert_eq!(res, None);
        assert_eq!(child.get("nope"), None);
        assert_eq!(root.get("nope"), None);
    }

    #[test]
    fn test_sibling_isolation() {
        let root = VariableScope::new();
        root.declare("shared".to_string(), Value::Int32(1));

        let a = VariableScope::branch(&root);
        let b = VariableScope::branch(&root);

        a.declare("only_a".to_string(), Value::Int32(100));
        b.declare("only_b".to_string(), Value::Int32(200));

        assert_eq!(a.get("shared"), Some(Value::Int32(1)));
        assert_eq!(b.get("shared"), Some(Value::Int32(1)));

        assert_eq!(a.get("only_a"), Some(Value::Int32(100)));
        assert_eq!(a.get("only_b"), None);

        assert_eq!(b.get("only_b"), Some(Value::Int32(200)));
        assert_eq!(b.get("only_a"), None);
    }

    #[test]
    fn test_multi_level_lookup_and_set() {
        let root = VariableScope::new();
        root.declare("z".to_string(), Value::Int32(1));

        let level1 = VariableScope::branch(&root);
        let level2 = VariableScope::branch(&level1);
        let level3 = VariableScope::branch(&level2);

        // lookup through 3 parents
        assert_eq!(level3.get("z"), Some(Value::Int32(1)));

        // set from deep child should bubble to the nearest binding (root here)
        let prev = level3.set("z".to_string(), Value::Int32(9));
        assert_eq!(prev, Some(Value::Int32(1)));
        assert_eq!(root.get("z"), Some(Value::Int32(9)));
        assert_eq!(level1.get("z"), Some(Value::Int32(9)));
        assert_eq!(level2.get("z"), Some(Value::Int32(9)));
        assert_eq!(level3.get("z"), Some(Value::Int32(9)));

        // shadow on level2, then set from level3 should hit level2 instead
        level2.declare("w".to_string(), Value::Int32(5));
        assert_eq!(level3.get("w"), Some(Value::Int32(5)));
        let prev_w = level3.set("w".to_string(), Value::Int32(6));
        assert_eq!(prev_w, Some(Value::Int32(5)));
        assert_eq!(level2.get("w"), Some(Value::Int32(6)));
        assert_eq!(root.get("w"), None);
    }

    #[test]
    fn test_redeclare_overwrites_in_same_scope_only() {
        let root = VariableScope::new();
        root.declare("k".to_string(), Value::Int32(1));
        assert_eq!(
            root.declare("k".to_string(), Value::Int32(2)),
            Some(Value::Int32(1))
        );
        assert_eq!(root.get("k"), Some(Value::Int32(2)));

        let child = VariableScope::branch(&root);
        // redeclare in child shouldn't touch parent
        assert_eq!(child.declare("k".to_string(), Value::Int32(3)), None);
        assert_eq!(child.get("k"), Some(Value::Int32(3)));
        assert_eq!(root.get("k"), Some(Value::Int32(2)));
    }

    #[test]
    fn test_parent_changes_visible_after_branch() {
        let root = VariableScope::new();
        root.declare("p".to_string(), Value::Int32(10));

        let child = VariableScope::branch(&root);
        assert_eq!(child.get("p"), Some(Value::Int32(10)));

        // mutate parent after child creation; child should observe new value
        root.set("p".to_string(), Value::Int32(11));
        assert_eq!(root.get("p"), Some(Value::Int32(11)));
        assert_eq!(child.get("p"), Some(Value::Int32(11)));
    }

    #[test]
    fn test_large_number_of_bindings() {
        let root = VariableScope::new();
        for i in 0..500 {
            root.declare(format!("v{i}"), Value::Int32(i));
        }

        let child = VariableScope::branch(&root);
        for i in 0..500 {
            assert_eq!(child.get(&format!("v{i}")), Some(Value::Int32(i)));
        }

        // Shadow a few and ensure lookups prefer child
        for i in (0..500).step_by(57) {
            child.declare(format!("v{i}"), Value::Int32(i + 1000));
        }
        for i in 0..500 {
            let expected = if i % 57 == 0 { i + 1000 } else { i };
            assert_eq!(child.get(&format!("v{i}")), Some(Value::Int32(expected)));
        }
    }
}
