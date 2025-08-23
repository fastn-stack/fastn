#[cfg(test)]
mod test {
    use crate::{Db, Hydrate, Reconcile};

    // Test Case 1: With document_id52 field + custom document_path
    #[derive(Debug, Clone, PartialEq, serde::Serialize, crate::Reconcile, crate::Hydrate, crate::Document)]
    #[document_path("/-/users/{id52}/profile")]
    struct UserProfile {
        #[document_id52]
        user_id: fastn_id52::PublicKey,
        name: String,
        bio: Option<String>,
    }

    // Test Case 2: With document_id52 field + NO document_path (should generate default)
    #[derive(Debug, Clone, PartialEq, serde::Serialize, crate::Reconcile, crate::Hydrate, crate::Document)]
    struct DefaultPathDoc {
        #[document_id52]
        entity: fastn_id52::PublicKey,
        data: String,
    }

    // Test Case 3: WITHOUT document_id52 field + custom document_path (singleton)
    #[derive(Debug, Clone, PartialEq, serde::Serialize, crate::Reconcile, crate::Hydrate, crate::Document)]
    #[document_path("/-/app/settings")]
    struct AppSettings {
        theme: String,
        debug_mode: bool,
    }

    // Test Case 4: Complex path template
    #[derive(Debug, Clone, PartialEq, serde::Serialize, crate::Reconcile, crate::Hydrate, crate::Document)]
    #[document_path("/-/complex/{id52}/nested/path")]
    struct ComplexPath {
        #[document_id52]
        owner: fastn_id52::PublicKey,
        value: i32,
    }

    // Test Case 5: Basic document for comprehensive testing
    #[derive(Debug, Clone, PartialEq, serde::Serialize, Hydrate, Reconcile, crate::Document)]
    #[document_path("/-/test/{id52}")]
    struct TestDoc {
        #[document_id52]
        id: fastn_id52::PublicKey,
        name: String,
        value: i32,
        items: Vec<String>,
    }

    // Test Case 6: Path-based API (no document_path attribute)
    #[derive(Debug, Clone, PartialEq, serde::Serialize, Hydrate, Reconcile, crate::Document)]
    struct PathBasedDoc {
        #[document_id52]
        id: fastn_id52::PublicKey,
        data: String,
    }

    #[track_caller]
    fn temp_db() -> (Db, tempfile::TempDir) {
        // Use tempfile for better isolation
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create a test PublicKey for the entity
        let test_entity = fastn_id52::SecretKey::generate().public_key();
        let db = Db::init(&db_path, &test_entity).unwrap();

        // Return temp_dir to keep it alive
        (db, temp_dir)
    }

    #[test]
    fn test_derive_with_id52_and_custom_path() -> Result<(), Box<dyn std::error::Error>> {
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
    fn test_derive_path_based_api() -> Result<(), Box<dyn std::error::Error>> {
        let (db, _temp_dir) = temp_db();

        let entity_id = fastn_id52::SecretKey::generate().public_key();

        // Path-based API: no document_path attribute means explicit paths required
        let doc_path = crate::DocumentPath::from_string("/-/custom/location/for/default")?;

        let doc = DefaultPathDoc {
            entity: entity_id,
            data: "test data".to_string(),
        };

        // All operations now require explicit path parameter
        doc.create(&db, &doc_path)?;
        let loaded = DefaultPathDoc::load(&db, &doc_path)?;
        assert_eq!(loaded.data, "test data");

        Ok(())
    }

    #[test]
    fn test_derive_singleton_custom_path() -> Result<(), Box<dyn std::error::Error>> {
        let (db, _temp_dir) = temp_db();

        // Test static path generation
        let generated_path = AppSettings::document_path();
        assert_eq!(generated_path.as_str(), "/-/app/settings");

        // Test operations on singleton document
        let settings = AppSettings {
            theme: "dark".to_string(),
            debug_mode: true,
        };

        settings.create(&db)?;
        let loaded = AppSettings::load(&db)?;
        assert_eq!(loaded.theme, "dark");
        assert!(loaded.debug_mode);

        // Test update
        let mut updated = loaded;
        updated.theme = "light".to_string();
        updated.debug_mode = false;
        updated.update(&db)?;

        let final_settings = AppSettings::load(&db)?;
        assert_eq!(final_settings.theme, "light");
        assert!(!final_settings.debug_mode);

        // Test that AppSettings does NOT have document_list() function
        // (This is verified by compilation - if document_list existed, we could uncomment this:)
        // let _ = AppSettings::document_list(&db)?; // This should NOT compile

        Ok(())
    }

    #[test]
    fn test_derive_complex_path_template() -> Result<(), Box<dyn std::error::Error>> {
        let (db, _temp_dir) = temp_db();

        let owner_id = fastn_id52::SecretKey::generate().public_key();

        // Test complex path generation
        let expected_path = format!("/-/complex/{}/nested/path", owner_id.id52());
        let generated_path = ComplexPath::document_path(&owner_id);
        assert_eq!(generated_path.as_str(), expected_path);

        // Test operations
        let doc = ComplexPath {
            owner: owner_id,
            value: 42,
        };

        doc.save(&db)?; // Test save on new document
        let loaded = ComplexPath::load(&db, &owner_id)?;
        assert_eq!(loaded.value, 42);

        Ok(())
    }

    #[test]
    fn test_derive_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        let (db, _temp_dir) = temp_db();

        let user_id = fastn_id52::SecretKey::generate().public_key();

        // Test create duplicate error
        let profile = UserProfile {
            user_id,
            name: "Alice".to_string(),
            bio: None,
        };

        profile.create(&db)?;
        assert!(profile.create(&db).is_err()); // Should fail on duplicate

        // Test load non-existent document
        let non_existent_id = fastn_id52::SecretKey::generate().public_key();
        assert!(UserProfile::load(&db, &non_existent_id).is_err());

        Ok(())
    }

    #[test]
    fn test_derive_multiple_instances() -> Result<(), Box<dyn std::error::Error>> {
        let (db, _temp_dir) = temp_db();

        // Create multiple test documents
        let test_docs = (0..5)
            .map(|i| {
                let id = fastn_id52::SecretKey::generate().public_key();
                TestDoc {
                    id,
                    name: format!("Test Doc {i}"),
                    value: i,
                    items: vec![format!("item-{}", i)],
                }
            })
            .collect::<Vec<_>>();

        // Create all documents
        for doc in &test_docs {
            doc.create(&db)?;
        }

        // Load and verify all documents
        for original_doc in &test_docs {
            let loaded = TestDoc::load(&db, &original_doc.id)?;
            assert_eq!(loaded, *original_doc);
        }

        // Test updating one document doesn't affect others
        let mut first_doc = TestDoc::load(&db, &test_docs[0].id)?;
        first_doc.value = 999;
        first_doc.update(&db)?;

        // Verify the update
        let updated = TestDoc::load(&db, &test_docs[0].id)?;
        assert_eq!(updated.value, 999);

        // Verify other documents unchanged
        for doc in test_docs.iter().skip(1) {
            let unchanged = TestDoc::load(&db, &doc.id)?;
            assert_eq!(unchanged, *doc);
        }

        Ok(())
    }

    #[test]
    fn test_derive_comprehensive_workflow() -> Result<(), Box<dyn std::error::Error>> {
        let (db, _temp_dir) = temp_db();

        // Test a comprehensive workflow using the derive macro
        let user_id = fastn_id52::SecretKey::generate().public_key();

        // 1. Create initial document
        let profile = UserProfile {
            user_id,
            name: "John Doe".to_string(),
            bio: Some("Software Engineer".to_string()),
        };
        profile.create(&db)?;

        // 2. Load and modify
        let mut loaded = UserProfile::load(&db, &user_id)?;
        loaded.bio = Some("Senior Software Engineer".to_string());
        loaded.update(&db)?;

        // 3. Test save() method (create or update)
        loaded.name = "John D. Doe".to_string();
        loaded.save(&db)?; // Should update since document exists

        // 4. Create singleton settings
        let settings = AppSettings {
            theme: "system".to_string(),
            debug_mode: false,
        };
        settings.save(&db)?; // Should create since document doesn't exist

        // 5. Verify all operations
        let final_profile = UserProfile::load(&db, &user_id)?;
        assert_eq!(final_profile.name, "John D. Doe");
        assert_eq!(
            final_profile.bio,
            Some("Senior Software Engineer".to_string())
        );

        let final_settings = AppSettings::load(&db)?;
        assert_eq!(final_settings.theme, "system");
        assert!(!final_settings.debug_mode);

        Ok(())
    }

    #[test]
    fn test_document_list() -> Result<(), Box<dyn std::error::Error>> {
        let (db, _temp_dir) = temp_db();

        // Create multiple test documents with different IDs
        let test_docs = (0..3)
            .map(|i| {
                let id = fastn_id52::SecretKey::generate().public_key();
                TestDoc {
                    id,
                    name: format!("Test Doc {i}"),
                    value: i,
                    items: vec![format!("item-{i}")],
                }
            })
            .collect::<Vec<_>>();

        // Create all documents
        for doc in &test_docs {
            doc.create(&db)?;
        }

        // Also create a user profile to make sure it doesn't interfere
        let user_id = fastn_id52::SecretKey::generate().public_key();
        let profile = UserProfile {
            user_id,
            name: "Alice".to_string(),
            bio: Some("Developer".to_string()),
        };
        profile.create(&db)?;

        // Test document_list for TestDoc
        let test_doc_paths = TestDoc::document_list(&db)?;
        assert_eq!(test_doc_paths.len(), 3);

        // Verify all our test documents are found
        for doc in &test_docs {
            let expected_path = TestDoc::document_path(&doc.id);
            assert!(test_doc_paths.iter().any(|p| p.as_str() == expected_path.as_str()));
        }

        // Test document_list for UserProfile  
        let user_profile_paths = UserProfile::document_list(&db)?;
        assert_eq!(user_profile_paths.len(), 1);

        let expected_profile_path = UserProfile::document_path(&user_id);
        assert_eq!(user_profile_paths[0].as_str(), expected_profile_path.as_str());

        Ok(())
    }

    #[test]
    fn test_document_list_exact_validation() -> Result<(), Box<dyn std::error::Error>> {
        let (db, _temp_dir) = temp_db();

        // Create valid TestDoc
        let valid_id = fastn_id52::SecretKey::generate().public_key();
        let valid_doc = TestDoc {
            id: valid_id,
            name: "Valid Doc".to_string(),
            value: 42,
            items: vec!["valid".to_string()],
        };
        valid_doc.create(&db)?;

        // Test that document_list only returns valid paths
        let paths = TestDoc::document_list(&db)?;
        assert_eq!(paths.len(), 1);

        // Verify the path is exactly what we expect
        let expected_path = TestDoc::document_path(&valid_id);
        assert_eq!(paths[0].as_str(), expected_path.as_str());

        // Verify the ID part is exactly 52 characters
        let path_str = paths[0].as_str();
        let id_part = &path_str[8..60]; // "/-/test/" = 8 chars, then 52 chars for ID
        assert_eq!(id_part.len(), 52);
        assert!(id_part.chars().all(|c| c.is_alphanumeric()));

        Ok(())
    }

    #[test]
    fn test_path_based_api() -> Result<(), Box<dyn std::error::Error>> {
        let (db, _temp_dir) = temp_db();

        let doc_id = fastn_id52::SecretKey::generate().public_key();
        let doc = PathBasedDoc {
            id: doc_id,
            data: "path-based test".to_string(),
        };

        // Path-based API requires explicit DocumentPath parameter
        let doc_path = crate::DocumentPath::from_string("/-/custom/path/for/test")?;

        // Test create with explicit path
        doc.create(&db, &doc_path)?;

        // Test load with explicit path  
        let loaded = PathBasedDoc::load(&db, &doc_path)?;
        assert_eq!(loaded.data, "path-based test");
        assert_eq!(loaded.id, doc_id);

        // Test update with explicit path
        let mut updated = loaded;
        updated.data = "updated data".to_string();
        updated.update(&db, &doc_path)?;

        // Test save with explicit path
        updated.data = "saved data".to_string();
        updated.save(&db, &doc_path)?;

        // Verify final state
        let final_doc = PathBasedDoc::load(&db, &doc_path)?;
        assert_eq!(final_doc.data, "saved data");

        // Test that the document doesn't have document_list function
        // (This is verified by compilation - if it existed, we could call it)

        Ok(())
    }

    #[test]
    fn test_json_queries() -> Result<(), Box<dyn std::error::Error>> {
        let (db, _temp_dir) = temp_db();

        // Create test documents with different data
        let user1_id = fastn_id52::SecretKey::generate().public_key();
        let user1 = UserProfile {
            user_id: user1_id,
            name: "Alice".to_string(),
            bio: Some("Engineer".to_string()),
        };
        user1.save(&db)?;

        let user2_id = fastn_id52::SecretKey::generate().public_key();
        let user2 = UserProfile {
            user_id: user2_id,
            name: "Bob".to_string(),
            bio: None,
        };
        user2.save(&db)?;

        let user3_id = fastn_id52::SecretKey::generate().public_key();
        let user3 = UserProfile {
            user_id: user3_id,
            name: "Alice".to_string(),  // Same name as user1
            bio: Some("Designer".to_string()),
        };
        user3.save(&db)?;

        // Test find_where: find users named "Alice"
        let alice_paths = db.find_where("name", "Alice")?;
        assert_eq!(alice_paths.len(), 2); // user1 and user3

        // Test find_where: find users named "Bob" 
        let bob_paths = db.find_where("name", "Bob")?;
        assert_eq!(bob_paths.len(), 1);

        // Test find_exists: find users with bio
        let users_with_bio = db.find_exists("bio")?;
        assert_eq!(users_with_bio.len(), 2); // user1 and user3 have bio

        // Test nested path queries would work with more complex documents
        // For now, our simple UserProfile doesn't have nested fields

        Ok(())
    }
}
