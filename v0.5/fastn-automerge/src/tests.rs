#[cfg(test)]
mod test {
    use crate::{Db, Hydrate, Reconcile};

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

    // Test Case 5: Basic document for comprehensive testing
    #[derive(Debug, Clone, PartialEq, Hydrate, Reconcile, crate::Document)]
    #[document_path("/-/test/{id52}")]
    struct TestDoc {
        #[document_id52]
        id: fastn_id52::PublicKey,
        name: String,
        value: i32,
        items: Vec<String>,
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
    fn test_derive_with_id52_default_path() -> Result<(), Box<dyn std::error::Error>> {
        let (db, _temp_dir) = temp_db();

        let entity_id = fastn_id52::SecretKey::generate().public_key();

        // Test default path generation (should be /-/{struct_name}/{id52})
        let expected_path = format!("/-/defaultpathdoc/{}", entity_id.id52());
        let generated_path = DefaultPathDoc::document_path(&entity_id);
        assert_eq!(generated_path.as_str(), expected_path);

        // Test basic operations
        let doc = DefaultPathDoc {
            entity: entity_id,
            data: "test data".to_string(),
        };

        doc.create(&db)?;
        let loaded = DefaultPathDoc::load(&db, &entity_id)?;
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
}
