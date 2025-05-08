use dashmap::DashMap;
use std::collections::hash_map::DefaultHasher;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// base type, holds a unique has and a name
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Symbol {
  hash: u64,
  name: String
}

impl Symbol {
  /// creates new symbol from a borrowed string
  pub fn new(name: &str) -> Self {
    let hash = Self::hash_name(name);
    Self {
      hash: hash,
      name: name.to_owned()
    }
  }

  /// creates a new symbol from a String
  pub fn from_string(name: String) -> Self {
    let hash = Self::hash_name(&name);
    Self {
      hash: hash,
      name: name
    }
  }

  /// creates a hash
  fn hash_name(name: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    hasher.finish()
  }

  /// gets the hash TODO chanage to get_hash
  pub fn hash(&self) -> u64 {
    self.hash
  }

  /// gets the name TODO change to get_name
  pub fn name(&self) -> &str {
    &self.name
  }
}

///
impl Display for Symbol {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, ":{}\n", self.name)
  }
}

// impl Display for Vec<Symbol> {
//   fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//     write!(f, "[");
//     for val in self.iter() {
//       write!(f, "{}, ", val);
//     }
//     write!(f, "]\n");
//   }
// }

/// symbol table of symbols that have unique hashes
#[derive(Debug)]
pub struct SymbolTable {
  table: DashMap<String, Arc<Symbol>>
}

impl SymbolTable {
  /// creates a new symbol table that is empty by default
  pub fn new() -> Self {
    Self {
      table: DashMap::new()
    }
  }

  /// pushes a new symbol, ensuring uniqueness.
  pub fn push(&self, name: &str) -> Arc<Symbol> {
    if let Some(existing) = self.table.get(name) {
      return Arc::clone(&existing);
    }

    let symbol = Arc::new(Symbol::new(name));
    self.table.insert(name.to_owned(), Arc::clone(&symbol));
    symbol
  }

  /// Looks up a symbol without adding it.
  pub fn lookup(&self, name: &str) -> Option<Arc<Symbol>> {
    self.table.get(name).map(|entry| Arc::clone(&entry))
  }
}

// probably need below for some thread safefty stuff
#[derive(Debug)]
pub struct UniqueSymbolGenerator {
  counter: AtomicUsize,
  table: SymbolTable,
}

impl UniqueSymbolGenerator {
  pub fn new() -> Self {
    Self {
      counter: AtomicUsize::new(0),
      table: SymbolTable::new(),
    }
  }

  /// Generates a unique symbol with a specified prefix.
  pub fn generate(&self, prefix: &str) -> Arc<Symbol> {
    let id = self.counter.fetch_add(1, Ordering::Relaxed);
    let name = format!("{}{}", prefix, id);
    self.table.push(&name)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_symbol_new() {
    let sym = Symbol::new("Any");
    assert_eq!(sym.name(), "Any");
  }
}