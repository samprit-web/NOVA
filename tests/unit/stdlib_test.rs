// Unit tests for NOVA Standard Library

use nova_std::prelude::*;
use nova_std::collections::*;
use nova_std::string::*;
use nova_std::option::*;
use nova_std::result::*;
use nova_std::io::*;

#[test]
fn test_vec_basic() {
    let mut v = Vec::new();
    assert!(v.is_empty());
    assert_eq!(v.len(), 0);
    
    v.push(1);
    v.push(2);
    v.push(3);
    assert_eq!(v.len(), 3);
    
    assert_eq!(v.pop(), Some(3));
    assert_eq!(v.pop(), Some(2));
    assert_eq!(v.pop(), Some(1));
    assert_eq!(v.pop(), None);
}

#[test]
fn test_vec_indexing() {
    let mut v = Vec::new();
    v.push(10);
    v.push(20);
    v.push(30);
    
    assert_eq!(v.get(0), Some(&10));
    assert_eq!(v.get(1), Some(&20));
    assert_eq!(v.get(2), Some(&30));
    assert_eq!(v.get(3), None);
    
    v[1] = 25;
    assert_eq!(v.get(1), Some(&25));
}

#[test]
fn test_vec_iter() {
    let mut v = Vec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    
    let mut sum = 0;
    for x in &v {
        sum += x;
    }
    assert_eq!(sum, 6);
    
    for x in &mut v {
        *x *= 2;
    }
    assert_eq!(v.as_slice(), &[2, 4, 6]);
}

#[test]
fn test_hashmap() {
    let mut m = HashMap::new();
    m.insert("a", 1);
    m.insert("b", 2);
    m.insert("c", 3);
    
    assert_eq!(m.len(), 3);
    assert_eq!(m.get(&"a"), Some(&1));
    assert_eq!(m.get(&"b"), Some(&2));
    assert_eq!(m.get(&"d"), None);
    
    assert_eq!(m.remove(&"a"), Some(1));
    assert_eq!(m.get(&"a"), None);
    assert_eq!(m.len(), 2);
}

#[test]
fn test_hashset() {
    let mut s = HashSet::new();
    s.insert(1);
    s.insert(2);
    s.insert(3);
    s.insert(2); // Duplicate
    
    assert_eq!(s.len(), 3);
    assert!(s.contains(&1));
    assert!(s.contains(&2));
    assert!(s.contains(&3));
    assert!(!s.contains(&4));
    
    assert!(s.remove(&2));
    assert!(!s.contains(&2));
    assert_eq!(s.len(), 2);
}

#[test]
fn test_string() {
    let mut s = String::new();
    assert!(s.is_empty());
    
    s.push_str("Hello");
    assert_eq!(s.as_str(), "Hello");
    
    s.push(' ');
    s.push_str("World");
    assert_eq!(s.as_str(), "Hello World");
    
    assert_eq!(s.len(), 11);
    assert_eq!(s.pop(), Some('d'));
    assert_eq!(s.as_str(), "Hello Worl");
}

#[test]
fn test_string_from() {
    let s = String::from("test");
    assert_eq!(s.as_str(), "test");
    
    let s: String = "hello".into();
    assert_eq!(s.as_str(), "hello");
}

#[test]
fn test_option() {
    let some = Option::Some(42);
    assert!(some.is_some());
    assert!(!some.is_none());
    assert_eq!(some.unwrap(), 42);
    
    let none: Option<i32> = Option::None;
    assert!(none.is_none());
    assert!(!none.is_some());
    assert_eq!(none.unwrap_or(0), 0);
}

#[test]
fn test_option_map() {
    let some = Option::Some(42);
    let doubled = some.map(|x| x * 2);
    assert_eq!(doubled, Option::Some(84));
    
    let none: Option<i32> = Option::None;
    let mapped = none.map(|x| x * 2);
    assert_eq!(mapped, Option::None);
}

#[test]
fn test_option_and_then() {
    let some = Option::Some(42);
    let result = some.and_then(|x| if x > 0 { Option::Some(x * 2) } else { Option::None });
    assert_eq!(result, Option::Some(84));
    
    let none: Option<i32> = Option::None;
    let result = none.and_then(|x| Option::Some(x * 2));
    assert_eq!(result, Option::None);
}

#[test]
fn test_result() {
    let ok: Result<i32, &str> = Result::Ok(42);
    assert!(ok.is_ok());
    assert!(!ok.is_err());
    assert_eq!(ok.unwrap(), 42);
    
    let err: Result<i32, &str> = Result::Err("error");
    assert!(err.is_err());
    assert!(!err.is_ok());
    assert_eq!(err.unwrap_err(), "error");
}

#[test]
fn test_result_map() {
    let ok = Result::Ok(42);
    let doubled = ok.map(|x| x * 2);
    assert_eq!(doubled, Result::Ok(84));
    
    let err: Result<i32, &str> = Result::Err("error");
    let mapped = err.map(|x| x * 2);
    assert_eq!(mapped, Result::Err("error"));
}

#[test]
fn test_result_and_then() {
    let ok = Result::Ok(42);
    let result = ok.and_then(|x| if x > 0 { Result::Ok(x * 2) } else { Result::Err("negative") });
    assert_eq!(result, Result::Ok(84));
    
    let err: Result<i32, &str> = Result::Err("error");
    let result = err.and_then(|x| Result::Ok(x * 2));
    assert_eq!(result, Result::Err("error"));
}

#[test]
fn test_vecdeque() {
    let mut d = VecDeque::new();
    d.push_front(1);
    d.push_back(2);
    d.push_front(0);
    
    assert_eq!(d.pop_front(), Some(0));
    assert_eq!(d.pop_front(), Some(1));
    assert_eq!(d.pop_back(), Some(2));
    assert_eq!(d.pop_front(), None);
}

#[test]
fn test_linked_list() {
    let mut list = LinkedList::new();
    list.push_front(1);
    list.push_back(2);
    list.push_front(0);
    
    assert_eq!(list.pop_front(), Some(0));
    assert_eq!(list.pop_front(), Some(1));
    assert_eq!(list.pop_front(), Some(2));
    assert_eq!(list.pop_front(), None);
}

#[test]
fn test_binary_heap() {
    let mut h = BinaryHeap::new();
    h.push(1);
    h.push(3);
    h.push(2);
    
    assert_eq!(h.pop(), Some(3));
    assert_eq!(h.pop(), Some(2));
    assert_eq!(h.pop(), Some(1));
}

#[test]
fn test_btree_map() {
    let mut m = BTreeMap::new();
    m.insert(3, "c");
    m.insert(1, "a");
    m.insert(2, "b");
    
    assert_eq!(m.get(&1), Some(&"a"));
    assert_eq!(m.get(&2), Some(&"b"));
    assert_eq!(m.get(&3), Some(&"c"));
    
    // Should be sorted
    let keys: Vec<_> = m.keys().cloned().collect();
    assert_eq!(keys, vec![1, 2, 3]);
}

#[test]
fn test_io_cursor() {
    let mut cursor = Cursor::new(vec![1, 2, 3, 4, 5]);
    let mut buf = [0; 3];
    assert_eq!(cursor.read(&mut buf).unwrap(), 3);
    assert_eq!(buf, [1, 2, 3]);
    cursor.set_position(3);
    assert_eq!(cursor.read(&mut buf).unwrap(), 2);
    assert_eq!(buf, [4, 5]);
}

#[test]
fn test_io_buf_reader() {
    let data = b"Hello, World!";
    let mut reader = BufReader::new(&data[..]);
    let mut buf = [0; 5];
    assert_eq!(reader.read(&mut buf).unwrap(), 5);
    assert_eq!(&buf, b"Hello");
}

#[test]
fn test_io_buf_writer() {
    let mut buf = Vec::new();
    let mut writer = BufWriter::new(&mut buf);
    writer.write_all(b"Hello").unwrap();
    writer.flush().unwrap();
    assert_eq!(buf, b"Hello");
}