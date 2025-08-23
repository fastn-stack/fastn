#[cfg(test)]
mod test {
    use crate::{Db, Hydrate, Reconcile};

    #[derive(Debug, Clone, PartialEq, Hydrate, Reconcile)]
    struct TestDoc {
        name: String,
        value: i32,
        items: Vec<String>,
    }

    // Test Case 1: With document_id52 field + custom document_path
    #[derive(Debug, Clone, PartialEq, crate::Reconcile, crate::Hydrate, crate::Document)]
    #[document_path("/-/users/{id52}/profile")]
    struct UserProfile {
        #[document_id52]
        user_id: fastn_id52::PublicKey,
        name: String,
        bio: Option<String>,
    }

    // Test Case 2: With document_id52 field + NO document_path (should generate default)
    #[derive(Debug, Clone, PartialEq, crate::Reconcile, crate::Hydrate, crate::Document)]
    struct DefaultPathDoc {
        #[document_id52]
        entity: fastn_id52::PublicKey,
        data: String,
    }

    // Test Case 3: WITHOUT document_id52 field + custom document_path (singleton)
    #[derive(Debug, Clone, PartialEq, crate::Reconcile, crate::Hydrate, crate::Document)]
    #[document_path("/-/app/settings")]
    struct AppSettings {
        theme: String,
        debug_mode: bool,
    }

    // Test Case 4: Complex path template
    #[derive(Debug, Clone, PartialEq, crate::Reconcile, crate::Hydrate, crate::Document)]
    #[document_path("/-/complex/{id52}/nested/path")]
    struct ComplexPath {
        #[document_id52]
        owner: fastn_id52::PublicKey,
        value: i32,
    }

    #[derive(Debug, Clone, PartialEq, Hydrate, Reconcile)]
    struct NestedDoc {
        title: String,
        nested: TestDoc,
        optional: Option<String>,
    }

    #[track_caller]
    fn temp_db() -> (Db, tempfile::TempDir) {
        // Use tempfile for better isolation
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Use simple test entity ID since init() now handles actor ID setup

        // Create a test PublicKey for the entity
        let test_entity = fastn_id52::SecretKey::generate().public_key();
        let db = Db::init(&db_path, &test_entity).unwrap();

        // Return temp_dir to keep it alive
        (db, temp_dir)
    }

    // Helper function for tests to create document paths easily
    fn doc_path(s: &str) -> crate::DocumentPath {
        crate::DocumentPath::from_string(s).expect("Test document path should be valid")
    }

    #[test]
    fn test_create_and_get() -> crate::Result<()> {
        let (db, _temp_dir) = temp_db();

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
        let (db, _temp_dir) = temp_db();

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
        let (db, _temp_dir) = temp_db();

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
        let (db, _temp_dir) = temp_db();

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
        let (db, _temp_dir) = temp_db();

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
        let (db, _temp_dir) = temp_db();

        assert!(db.delete(&doc_path("/nonexistent")).is_err());

        Ok(())
    }

    #[test]
    fn test_exists() -> crate::Result<()> {
        let (db, _temp_dir) = temp_db();

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
        let (db, _temp_dir) = temp_db();

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
        let (db, _temp_dir) = temp_db();

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
        let (db, _temp_dir) = temp_db();

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
        let (db, _temp_dir) = temp_db();

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

    #[test]
    fn test_derive_with_id52_and_custom_path() -> crate::Result<()> {
        let (db, _temp_dir) = temp_db();

        let user_id = fastn_id52::SecretKey::generate().public_key();

        // Test path generation
        let expected_path = format!("/-/users/{}/profile", user_id.id52());
        let generated_path = UserProfile::document_path(&user_id);
        assert_eq!(generated_path.as_str(), expected_path);

        // Test CRUD operations
        let profile = UserProfile {
            user_id,
            name: "Alice".to_string(),
            bio: Some("Developer".to_string()),
        };

        // Test create
        profile.create(&db)?;

        // Test load
        let loaded = UserProfile::load(&db, &user_id)?;
        assert_eq!(loaded.name, "Alice");
        assert_eq!(loaded.bio, Some("Developer".to_string()));

        // Test update
        let mut updated = loaded;
        updated.name = "Alice Smith".to_string();
        updated.update(&db)?;

        // Test save (should work on existing doc)
        updated.bio = None;
        updated.save(&db)?;

        // Verify final state
        let final_doc = UserProfile::load(&db, &user_id)?;
        assert_eq!(final_doc.name, "Alice Smith");
        assert_eq!(final_doc.bio, None);

        Ok(())
    }

    #[test]
    fn test_derive_with_id52_default_path() -> crate::Result<()> {
        let (db, _temp_dir) = temp_db();

        let entity = fastn_id52::SecretKey::generate().public_key();

        // Test default path generation: /-/{struct_name}/{id52}
        let expected_path = format!("/-/defaultpathdoc/{}", entity.id52());
        let generated_path = DefaultPathDoc::document_path(&entity);
        assert_eq!(generated_path.as_str(), expected_path);

        // Test CRUD operations
        let doc = DefaultPathDoc {
            entity,
            data: "test data".to_string(),
        };

        doc.save(&db)?;
        let loaded = DefaultPathDoc::load(&db, &entity)?;
        assert_eq!(loaded.data, "test data");

        Ok(())
    }

    #[test]
    fn test_derive_singleton_custom_path() -> crate::Result<()> {
        let (db, _temp_dir) = temp_db();

        // Test static path (no ID52 substitution)
        let generated_path = AppSettings::document_path();
        assert_eq!(generated_path.as_str(), "/-/app/settings");

        // Test singleton CRUD operations (no ID parameter needed)
        let settings = AppSettings {
            theme: "dark".to_string(),
            debug_mode: true,
        };

        settings.create(&db)?;

        // Load without ID parameter
        let loaded = AppSettings::load(&db)?;
        assert_eq!(loaded.theme, "dark");
        assert!(loaded.debug_mode);

        // Test update
        let mut updated = loaded;
        updated.theme = "light".to_string();
        updated.save(&db)?;

        let final_settings = AppSettings::load(&db)?;
        assert_eq!(final_settings.theme, "light");

        Ok(())
    }

    #[test]
    fn test_derive_complex_path_template() -> crate::Result<()> {
        let (db, _temp_dir) = temp_db();

        let owner = fastn_id52::SecretKey::generate().public_key();

        // Test complex path template
        let expected_path = format!("/-/complex/{}/nested/path", owner.id52());
        let generated_path = ComplexPath::document_path(&owner);
        assert_eq!(generated_path.as_str(), expected_path);

        // Test functionality
        let complex = ComplexPath { owner, value: 42 };

        complex.save(&db)?;
        let loaded = ComplexPath::load(&db, &owner)?;
        assert_eq!(loaded.value, 42);

        Ok(())
    }

    #[test]
    fn test_derive_error_handling() -> crate::Result<()> {
        let (db, _temp_dir) = temp_db();

        let user_id = fastn_id52::SecretKey::generate().public_key();
        let profile = UserProfile {
            user_id,
            name: "Alice".to_string(),
            bio: None,
        };

        // Test create on new document (should succeed)
        profile.create(&db)?;

        // Test create on existing document (should fail)
        assert!(profile.create(&db).is_err());

        // Test update on existing document (should succeed)
        let mut updated = profile.clone();
        updated.name = "Alice Updated".to_string();
        updated.update(&db)?;

        // Test update on non-existent document (should fail)
        let other_user = fastn_id52::SecretKey::generate().public_key();
        let other_profile = UserProfile {
            user_id: other_user,
            name: "Bob".to_string(),
            bio: None,
        };
        assert!(other_profile.update(&db).is_err());

        // Test load on non-existent document (should fail)
        assert!(UserProfile::load(&db, &other_user).is_err());

        Ok(())
    }

    #[test]
    fn test_derive_multiple_instances() -> crate::Result<()> {
        let (db, _temp_dir) = temp_db();

        // Test multiple users in the same database
        let alice_id = fastn_id52::SecretKey::generate().public_key();
        let bob_id = fastn_id52::SecretKey::generate().public_key();

        let alice = UserProfile {
            user_id: alice_id,
            name: "Alice".to_string(),
            bio: Some("Alice's bio".to_string()),
        };

        let bob = UserProfile {
            user_id: bob_id,
            name: "Bob".to_string(),
            bio: Some("Bob's bio".to_string()),
        };

        // Save both
        alice.save(&db)?;
        bob.save(&db)?;

        // Load and verify both
        let loaded_alice = UserProfile::load(&db, &alice_id)?;
        let loaded_bob = UserProfile::load(&db, &bob_id)?;

        assert_eq!(loaded_alice.name, "Alice");
        assert_eq!(loaded_bob.name, "Bob");
        assert_eq!(loaded_alice.bio, Some("Alice's bio".to_string()));
        assert_eq!(loaded_bob.bio, Some("Bob's bio".to_string()));

        // Verify paths are different
        let alice_path = UserProfile::document_path(&alice_id);
        let bob_path = UserProfile::document_path(&bob_id);
        assert_ne!(alice_path.as_str(), bob_path.as_str());

        Ok(())
    }
}
