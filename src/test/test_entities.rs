use crate::error::Result;
use crate::AutoIncrementEntity;
use serde_derive::{Deserialize, Serialize};
use sled::Db;
use reindeer_macros::entity;

use crate::DeletionBehaviour;
use crate::Entity;

#[derive(Serialize, Deserialize)]
#[entity]
pub struct Entity1 {
    pub id: u32,
    pub prop1: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Entity2 {
    pub id: String,
    pub prop2: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Entity3 {
    pub id: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChildEntity1 {
    id: (String, u32),
}

#[derive(Serialize, Deserialize)]
pub struct ChildEntity2 {
    id: (u32, u32),
}

#[derive(Serialize, Deserialize)]
pub struct GrandChildEntity {
    id: ((String, u32), u32),
}

impl Entity for Entity1 {
    type Key = u32;

    fn store_name() -> &'static str {
        "entity_1"
    }

    fn get_key(&self) -> &Self::Key {
        &self.id
    }

    fn set_key(&mut self, key: &Self::Key) {
        self.id = *key;
    }
    fn get_sibling_stores() -> Vec<(&'static str, DeletionBehaviour)> {
        vec![("entity_3", DeletionBehaviour::Cascade)]
    }
}

impl Entity for Entity2 {
    type Key = String;

    fn store_name() -> &'static str {
        "entity_2"
    }

    fn get_key(&self) -> &Self::Key {
        &self.id
    }

    fn set_key(&mut self, key: &Self::Key) {
        self.id = key.clone();
    }

    fn get_child_stores() -> Vec<(&'static str, crate::DeletionBehaviour)> {
        vec![("child_entity_1", DeletionBehaviour::Cascade)]
    }
}

impl Entity for Entity3 {
    type Key = u32;

    fn store_name() -> &'static str {
        "entity_3"
    }

    fn get_key(&self) -> &Self::Key {
        &self.id
    }

    fn set_key(&mut self, key: &Self::Key) {
        self.id = *key;
    }
    fn get_sibling_stores() -> Vec<(&'static str, DeletionBehaviour)> {
        vec![("entity_1", DeletionBehaviour::Error)]
    }
    fn get_child_stores() -> Vec<(&'static str, DeletionBehaviour)> {
        vec![("child_entity_2", DeletionBehaviour::Error)]
    }
}

impl Entity for ChildEntity1 {
    type Key = (String, u32);

    fn store_name() -> &'static str {
        "child_entity_1"
    }

    fn get_key(&self) -> &Self::Key {
        &self.id
    }

    fn set_key(&mut self, key: &Self::Key) {
        self.id = key.clone();
    }

    fn get_child_stores() -> Vec<(&'static str, DeletionBehaviour)> {
        vec![("grand_child_entity", DeletionBehaviour::Cascade)]
    }
}

impl Entity for ChildEntity2 {
    type Key = (u32, u32);

    fn store_name() -> &'static str {
        "child_entity_2"
    }

    fn get_key(&self) -> &Self::Key {
        &self.id
    }

    fn set_key(&mut self, key: &Self::Key) {
        self.id = *key;
    }
}

impl Entity for GrandChildEntity {
    type Key = ((String, u32), u32);

    fn store_name() -> &'static str {
        "grand_child_entity"
    }

    fn get_key(&self) -> &Self::Key {
        &self.id
    }

    fn set_key(&mut self, key: &Self::Key) {
        self.id = key.clone();
    }
}

pub fn set_up(name: &str) -> Result<Db> {
    let mut dir = std::env::temp_dir();
    dir.push(name);

    let db = crate::open(dir.to_str().unwrap())?;
    Entity1::register(&db)?;
    Entity2::register(&db)?;
    Entity3::register(&db)?;
    ChildEntity1::register(&db)?;
    ChildEntity2::register(&db)?;
    GrandChildEntity::register(&db)?;
    Ok(db)
}

pub fn set_up_content(db: &Db) -> Result<()> {
    let mut e1 = Entity1 {
        id: 0,
        prop1: String::from("Hello, World!"),
    };
    e1.save_next(db)?;
    e1.prop1 = String::from("Hello, Nancy!");
    e1.save_next(db)?;
    e1.prop1 = String::from("Hello, Jack!");
    e1.save_next(db)?;
    let mut e2 = Entity2 {
        id: String::from("id1"),
        prop2: 3,
    };
    e2.save(db)?;
    e2.set_key(&String::from("id2"));
    e2.prop2 = 5;
    e2.save(db)?;
    let e2_2 = e2.clone();
    e2.set_key(&String::from("id3"));
    e2.prop2 = 1000;
    e2.save(db)?;
    let mut e3 = Entity3 { id: 0 };
    e3.save_next(db)?;
    e3.save_next(db)?;
    let mut e4 = ChildEntity1 {
        id: (String::from("id0"), 0),
    };
    e3.save_next(db)?;

    e2.save_next_child(&mut e4, db)?;
    e2.save_next_child(&mut e4, db)?;
    e2.save_next_child(&mut e4, db)?;
    let mut e4_2 = e4.clone();
    e2_2.save_next_child(&mut e4_2, db)?;

    let mut e5 = ChildEntity2 { id: (0, 0) };
    e3.save_next_child(&mut e5, db)?;
    e3.save_next_child(&mut e5, db)?;
    e4.create_relation(
        &e5,
        DeletionBehaviour::BreakLink,
        DeletionBehaviour::BreakLink,
        None,
        db,
    )?;
    e3.save_next_child(&mut e5, db)?;
    let mut grand_child = GrandChildEntity {
        id: ((String::from("id0"), 1), 0),
    };
    e4.save_next_child(&mut grand_child, db)?;
    e4.save_next_child(&mut grand_child, db)?;
    e4.save_next_child(&mut grand_child, db)?;
    assert_eq!(grand_child.get_key().0 .0, "id3");
    assert_eq!(grand_child.get_key().0 .1, 2);
    assert_eq!(grand_child.get_key().1, 2);
    Ok(())
}

pub fn tear_down(name: &str) -> Result<()> {
    let mut dir = std::env::temp_dir();
    dir.push(name);
    std::fs::remove_dir_all(dir.to_str().unwrap())?;
    Ok(())
}
