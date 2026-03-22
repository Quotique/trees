use std::fmt;

use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize,
};

use crate::{Node, Tree};

impl<T> Serialize for &Node<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Node", 2)?;
        s.serialize_field("data", &self.data())?;
        if self.degree() > 0 {
            s.serialize_field("children", &self.iter().collect::<Vec<_>>())?;
        }
        s.end()
    }
}

impl<'de, T> Deserialize<'de> for Tree<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            "Node",
            &["data", "children"],
            NodeVisitor(std::marker::PhantomData),
        )
    }
}

struct NodeVisitor<T>(std::marker::PhantomData<T>);

impl<'de, T> Visitor<'de> for NodeVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = Tree<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a Node struct")
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut data: Option<T> = None;
        let mut children: Option<Vec<Tree<T>>> = None;

        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "data" => data = Some(map.next_value()?),
                "children" => children = Some(map.next_value()?),
                _ => {}
            }
        }

        let mut node = Tree::new(data.ok_or_else(|| serde::de::Error::missing_field("data"))?);
        for child in children.unwrap_or_default() {
            node.push_back(child);
        }

        Ok(node)
    }
}

#[cfg(test)]
mod tests {
    use serde_derive::{Deserialize, Serialize};

    use crate::Tree;

    #[test]
    fn serialize_complex_struct() {
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        struct Point {
            x: i32,
            y: i32,
        }
        let mut tree = Tree::new(Point { x: 1, y: 1 });
        tree.root_mut().push_back(Tree::new(Point { x: 10, y: 10 }));
        tree.root_mut().push_back(Tree::new(Point { x: 20, y: 30 }));

        let serialized = serde_json::to_string(&tree.root()).unwrap();
        assert_eq!(
            serialized,
            r#"{"data":{"x":1,"y":1},"children":[{"data":{"x":10,"y":10}},{"data":{"x":20,"y":30}}]}"#
        );
        let deserialized: Tree<Point> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.data(), &Point { x: 1, y: 1 });
        assert_eq!(deserialized.degree(), 2);
    }

    #[test]
    fn serialize_deserialize_scattered() {
        let mut tree = Tree::new(42);
        tree.root_mut().push_back(Tree::new(10));
        tree.root_mut().push_back(Tree::new(20));

        let serialized = serde_json::to_string(&tree.root()).unwrap();
        assert_eq!(
            serialized,
            r#"{"data":42,"children":[{"data":10},{"data":20}]}"#
        );
        let deserialized: Tree<i32> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.data(), &42);
        assert_eq!(deserialized.degree(), 2);
    }

    #[test]
    fn serialize_deserialize_nested() {
        let tree: Tree<i32> = Tree::from_tuple((0, (1, 2, 3), (4, 5)));

        let serialized = serde_json::to_string(&tree.root()).unwrap();
        let deserialized: Tree<i32> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.data(), &0);
        assert_eq!(deserialized.degree(), 2);
        assert_eq!(deserialized.iter().next().unwrap().data(), &1);
    }

    #[test]
    fn tree_roundtrip() {
        let tree: Tree<i32> = Tree::from_tuple((10, (20, 30, 40), (50, 60)));

        let serialized = serde_json::to_string(&tree.root()).unwrap();
        let deserialized: Tree<i32> = serde_json::from_str(&serialized).unwrap();

        // Verify structure
        assert_eq!(deserialized.data(), &10);
        assert_eq!(deserialized.degree(), 2);

        let children: Vec<_> = deserialized.iter().collect();
        assert_eq!(children[0].data(), &20);
        assert_eq!(children[1].data(), &50);
    }
}
