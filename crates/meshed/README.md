### Quickly turn deserialized data into graphs

# Quick start 

Provides core traits [`Identity`], [`Identifiable`], [`Query`] and [`Edges`] as an
abstraction to traverse arbitrary data and link it up into a graph. Allows generic 
traversals / multiple types of traversals through the same data.

Traits `ExtractData` and node annotations allow extraction of metadata into a graph.

Single threaded (still pretty fast though). 

Full example:
```rust
use meshed::prelude::*;
use std::fmt;
type Id = i32;

struct Item {
    id: Id,
    children: Vec<Id>
}

struct ListOfItems {
    items: Vec<Item>
}

// Core traits 
use meshed::identify::{Identity, Identifiable};

// For convenience this is also implemented by default for i32.
// Ids must be lightweight, comparable and cheaply cloneable. 
// if you're using strings, consider using an Rc

impl Label for Item {
    type Label = Id;
    fn label(&self) -> Self::Label {
        self.id
    }
}

impl Edges<Id, ()> for Item {
    fn next_edge(&self, previous_edge_index: Option<usize>) -> Option<Edge<Id, ()>> {
        let next_index = previous_edge_index.map(|i| i + 1).unwrap_or_default();
        let edge = self.children.get(next_index)?.clone();
        Some(Edge::new(self.get_id(), edge, next_index, ()))
    }
}

impl Identifiable<Id> for Item {
    fn get_id(&self) -> Id {
        self.id
    }
}
// This trait tells the graph system that a type can be used to 
// look up nodes when required
impl Query<Id, Item> for ListOfItems {
    fn query(&self, identifier: &Id) -> Option<&Item> {
        self.items.iter().find(|item| &item.id == identifier)
    }
    
    fn all(&self) -> Vec<&Item> {
        self.items.iter().collect()
    }
}

use meshed::graph::{Graph, SimpleGraphDefinition, GraphDefinition};
// Identifier, Node Metadata, Edge Metadata
type ItemGraph = Graph<SimpleGraphDefinition>;

let data = ListOfItems {
    items: vec![
        Item { id: 0, children: vec![1, 2] },
        Item { id: 1, children: vec![0, 3] },
        Item { id: 2, children: vec![3] },
        Item { id: 3, children: vec![1] }
    ]
};

let graph = SimpleGraphDefinition::build_graph(&data);
```

## Adding node metadata 
The [`ExtractData`] trait provides a way to pull data out of a datasource and 
add it to a node. 

```rust
use std::borrow::Cow;
use meshed::prelude::*;
struct Item<'a> {
    id: i32,
    children: Vec<i32>,
    borrowed_value: Cow<'a, str>
}

struct Meta {
    value: String,
}

impl<'a> ExtractData<Meta> for Item<'a> {
    fn extract_data(&self) -> Meta {
        Meta { value: self.borrowed_value.to_string() }
    }
}
```

The extracted value must be `'static`. This is to prevent the graph grom having
live references from the source data. Source data may be dropped after the graph 
has been built. 

## Providing multiple traversals 

Multiple edge traversals may be provided using unit types as the second parameter to 
[`Edges`].

```rust
use meshed::prelude::*;
struct Item {
    id: i32,
    children: Vec<i32>,
    parents: Vec<i32>,
}

struct Child;
struct Parent;

impl Edges<i32, Parent> for Item {
    fn next_edge(&self, previous_edge_index: Option<usize>) -> Option<Edge<i32, Parent>> {
        let next_index = previous_edge_index.map(|i| i + 1).unwrap_or_default();
        let edge = self.parents.get(next_index)?.clone();
        Some(Edge::new(self.id, edge, next_index, Parent))
    }
}

impl Edges<i32, Child> for Item {
    fn next_edge(&self, previous_edge_index: Option<usize>) -> Option<Edge<i32, Child>> {
        let next_index = previous_edge_index.map(|i| i + 1).unwrap_or_default();
        let edge = self.children.get(next_index)?.clone();
        Some(Edge::new(self.id, edge, next_index, Child))
    }
}
```
This allows multiple traversals to be defined when building a graph 

```rust ignore 
type ItemChildGraph = Graph<Id, (), Child>;
type ItemParentGraph = Graph<Id, (), Parent>;
```

## Traversing the graphs 
BFS and DFS traversals are provided via `EdgeTraversal`. An acyclic version is in the 
same module as `AcyclicTraversal`.

Traversals produce `TraversedNodes` and can be used to prune, or truncate existing graphs.


## Annotating nodes 
Sometimes during traversal we want to annotate nodes with temporary data. These annotations are 
preserved when truncating or pruning graphs, but may not be present for all nodes (or be consistent between traversals).

A good example of this are annotating webpack chunks onto webpack modules. The chunk that a module
is found in is highly dependent on the entrypoint the traversal started from.

```rust ignore
let node = Node::new_default();

struct AnyStruct { value: usize }
enum EnumAnnotation {
  Variant
};

node.annotate(AnyStruct {  value: 3 });
node.annotate(EnumAnnotation::Variant);
```

This relies on `Any` and downcasting internally. Any type can be stored here for as long
as there is only one copy of that type.