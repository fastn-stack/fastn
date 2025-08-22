#[cfg(test)]
mod test {
    use crate::{Db, Hydrate, Reconcile};

    #[derive(Debug, Clone, PartialEq, Hydrate, Reconcile)]
    struct TestDoc {
        name: String,
        value: i32,
        items: Vec<String>,
    }

    // TODO: Test the derive macro once error handling is refined
    // #[derive(crate::Document)]
    // struct TestEntity {
    //     #[document_id52]
    //     entity: fastn_id52::PublicKey,
    //     name: String,
    //     data: Option<String>,
    // }

    #[derive(Debug, Clone, PartialEq, Hydrate, Reconcile)]
    struct NestedDoc {
        title: String,
        nested: TestDoc,
        optional: Option<String>,
    }

    fn temp_db() -> crate::Result<(Db, tempfile::TempDir)> {
        // Use tempfile for better isolation
        let temp_dir = tempfile::TempDir::new().map_err(|e| {
            Box::new(crate::Error::Database(rusqlite::Error::InvalidColumnType(
                0,
                format!("Failed to create temp dir: {e}"),
                rusqlite::types::Type::Text,
            )))
        })?;
        let db_path = temp_dir.path().join("test.db");

        // Use simple test entity ID since init() now handles actor ID setup

        // Create a test PublicKey for the entity
        let test_entity = fastn_id52::SecretKey::generate().public_key();
        let db = Db::init(&db_path, &test_entity)?;

        // Return temp_dir to keep it alive
        Ok((db, temp_dir))
    }

    // Helper function for tests to create document paths easily
    fn doc_path(s: &str) -> crate::DocumentPath {
        crate::DocumentPath::from_string(s).expect("Test document path should be valid")
    }

    #[test]
    fn test_create_and_get() -> crate::Result<()> {
        let (db, _temp_dir) = temp_db()?;

        let doc = TestDoc {
            name: "test document".to_string(),
            value: 42,
            items: vec!["one".to_string(), "two".to_string()],
        };

        // Create document
        db.create(&doc_path("/test/doc1"), &doc)?;

        // Get document
        let retrieved: TestDoc = db.get(&doc_path("/test/doc1"))?;
        assert_eq!(retrieved, doc);

        Ok(())
    }

    #[test]
    fn test_create_duplicate_fails() -> crate::Result<()> {
        let (db, _temp_dir) = temp_db()?;

        let doc = TestDoc {
            name: "test".to_string(),
            value: 1,
            items: vec![],
        };

        // First create should succeed
        db.create(&doc_path("/test/doc"), &doc)?;

        // Second create should fail
        assert!(db.create(&doc_path("/test/doc"), &doc).is_err());

        Ok(())
    }

    #[test]
    fn test_update() -> crate::Result<()> {
        let (db, _temp_dir) = temp_db()?;

        let original = TestDoc {
            name: "original".to_string(),
            value: 1,
            items: vec!["a".to_string()],
        };

        db.create(&doc_path("/test/update"), &original)?;

        let updated = TestDoc {
            name: "updated".to_string(),
            value: 2,
            items: vec!["a".to_string(), "b".to_string()],
        };

        db.update(&doc_path("/test/update"), &updated)?;

        let retrieved: TestDoc = db.get(&doc_path("/test/update"))?;
        assert_eq!(retrieved, updated);

        Ok(())
    }

    #[test]
    fn test_modify() -> crate::Result<()> {
        let (db, _temp_dir) = temp_db()?;

        let doc = TestDoc {
            name: "modify test".to_string(),
            value: 10,
            items: vec!["initial".to_string()],
        };

        db.create(&doc_path("/test/modify"), &doc)?;

        // Modify the document
        db.modify(&doc_path("/test/modify"), |d: &mut TestDoc| {
            d.value *= 2;
            d.items.push("added".to_string());
        })?;

        let retrieved: TestDoc = db.get(&doc_path("/test/modify"))?;
        assert_eq!(retrieved.value, 20);
        assert_eq!(retrieved.items.len(), 2);
        assert_eq!(retrieved.items[1], "added");

        Ok(())
    }

    #[test]
    fn test_delete() -> crate::Result<()> {
        let (db, _temp_dir) = temp_db()?;

        let doc = TestDoc {
            name: "to delete".to_string(),
            value: 999,
            items: vec![],
        };

        db.create(&doc_path("/test/delete"), &doc)?;

        // Verify it exists
        assert!(db.exists(&doc_path("/test/delete"))?);

        // Delete it
        db.delete(&doc_path("/test/delete"))?;

        // Verify it's gone
        assert!(!db.exists(&doc_path("/test/delete"))?);
        assert!(db.get::<TestDoc>(&doc_path("/test/delete")).is_err());

        Ok(())
    }

    #[test]
    fn test_delete_nonexistent_fails() -> crate::Result<()> {
        let (db, _temp_dir) = temp_db()?;

        assert!(db.delete(&doc_path("/nonexistent")).is_err());

        Ok(())
    }

    #[test]
    fn test_exists() -> crate::Result<()> {
        let (db, _temp_dir) = temp_db()?;

        assert!(!db.exists(&doc_path("/test/nonexistent"))?);

        let doc = TestDoc {
            name: "exists test".to_string(),
            value: 1,
            items: vec![],
        };

        db.create(&doc_path("/test/exists"), &doc)?;
        assert!(db.exists(&doc_path("/test/exists"))?);

        Ok(())
    }

    #[test]
    fn test_list() -> crate::Result<()> {
        let (db, _temp_dir) = temp_db()?;

        let doc = TestDoc {
            name: "list test".to_string(),
            value: 1,
            items: vec![],
        };

        // Create multiple documents
        db.create(&doc_path("/docs/a"), &doc)?;
        db.create(&doc_path("/docs/b"), &doc)?;
        db.create(&doc_path("/other/c"), &doc)?;

        // List all
        let all = db.list(None)?;
        assert_eq!(all.len(), 3);
        assert!(all.contains(&"/docs/a".to_string()));
        assert!(all.contains(&"/docs/b".to_string()));
        assert!(all.contains(&"/other/c".to_string()));

        // List with prefix
        let docs_only = db.list(Some("/docs"))?;
        assert_eq!(docs_only.len(), 2);
        assert!(docs_only.contains(&"/docs/a".to_string()));
        assert!(docs_only.contains(&"/docs/b".to_string()));

        Ok(())
    }

    #[test]
    fn test_nested_structures() -> crate::Result<()> {
        let (db, _temp_dir) = temp_db()?;

        let nested = NestedDoc {
            title: "nested document".to_string(),
            nested: TestDoc {
                name: "inner".to_string(),
                value: 123,
                items: vec!["x".to_string(), "y".to_string(), "z".to_string()],
            },
            optional: Some("present".to_string()),
        };

        db.create(&doc_path("/test/nested"), &nested)?;

        let retrieved: NestedDoc = db.get(&doc_path("/test/nested"))?;
        assert_eq!(retrieved, nested);
        assert_eq!(retrieved.nested.items.len(), 3);
        assert_eq!(retrieved.optional, Some("present".to_string()));

        Ok(())
    }

    #[test]
    fn test_get_document() -> crate::Result<()> {
        let (db, _temp_dir) = temp_db()?;

        let doc = TestDoc {
            name: "raw doc".to_string(),
            value: 55,
            items: vec!["item".to_string()],
        };

        db.create(&doc_path("/test/raw"), &doc)?;

        // Get raw AutoCommit document
        let raw_doc = db.get_document(&doc_path("/test/raw"))?;

        // Should be able to hydrate from it
        let hydrated: TestDoc = autosurgeon::hydrate(&raw_doc)?;
        assert_eq!(hydrated, doc);

        Ok(())
    }

    #[test]
    fn test_actor_id_consistency() -> crate::Result<()> {
        let (db, _temp_dir) = temp_db()?;

        let doc1 = TestDoc {
            name: "doc1".to_string(),
            value: 1,
            items: vec![],
        };

        let doc2 = TestDoc {
            name: "doc2".to_string(),
            value: 2,
            items: vec![],
        };

        // Create two documents
        db.create(&doc_path("/test/actor1"), &doc1)?;
        db.create(&doc_path("/test/actor2"), &doc2)?;

        // Update them
        db.update(&doc_path("/test/actor1"), &doc2)?;
        db.update(&doc_path("/test/actor2"), &doc1)?;

        // Both should have consistent actor IDs throughout their history
        let _raw1 = db.get_document(&doc_path("/test/actor1"))?;
        let _raw2 = db.get_document(&doc_path("/test/actor2"))?;

        // Check that we can still retrieve them
        let retrieved1: TestDoc = db.get(&doc_path("/test/actor1"))?;
        let retrieved2: TestDoc = db.get(&doc_path("/test/actor2"))?;

        assert_eq!(retrieved1, doc2);
        assert_eq!(retrieved2, doc1);

        Ok(())
    }
}
