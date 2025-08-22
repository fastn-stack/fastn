# fastn Development Plan - Type-Safe Document System

## Current Status ✅

**Major Achievements:**
- ✅ Type-safe `DocumentId` system with validation
- ✅ Actor ID management with privacy-focused design (entity_id52 + device_number)
- ✅ Specific error types per database operation (CreateError, UpdateError, etc.)
- ✅ Common document patterns (DocumentLoadError, DocumentSaveError)
- ✅ fastn CLI integration (`fastn automerge` subcommands working)
- ✅ Zero-based device numbering with proper initialization tracking
- ✅ 10/11 tests passing, zero compilation errors, minimal clippy warnings

## 🎯 Immediate Next Tasks (Priority Order)

### 1. **Complete Document System** 🚀 CURRENT FOCUS
- **Implement derive macro** - `#[derive(Document)]` to auto-generate load/save methods
  ```rust
  #[derive(Document)]
  struct MyDoc {
      #[document_id_field] 
      id: PublicKey,
      data: String,
  }
  // Auto-generates: load(db, id), save(&self, db), document ID constructor
  ```
- **Fix intermittent list test** - Resolve 4 vs 3 documents test isolation issue
- **Polish error types** - Add Display/Error traits for DocumentLoadError/DocumentSaveError

### 2. **Fix Original Issue**
- **Test `fastn run`** - Verify original failing command now works
- **Complete fastn-account integration** - Fix remaining type mismatches in save() methods
- **Update fastn-rig integration** - Ensure all manual Automerge operations replaced

### 3. **Production CLI**
- **Add actor ID management commands:**
  - `fastn automerge set-actor-id <entity_id52> <device_number>`
  - `fastn automerge next-actor-id <entity_id52>` 
  - `fastn automerge get-actor-id`
  - `fastn automerge actor-info`
- **Replace dummy CLI entity** - Use real account IDs instead of "cli-dummy-entity"

## 🚀 Strategic Next Steps

### 4. **Multi-Device Support**
- **Test actor ID system** - Verify device counter works across scenarios
- **Implement actor ID rewriting** - Privacy-preserving document sharing
- **Device management** - Add/remove devices from accounts

### 5. **Developer Experience**
- **Update documentation** - New actor ID patterns and privacy implications
- **Create examples** - Show proper document system usage
- **Migration guide** - Transition from old to new APIs

### 6. **Performance & Polish**
- **Database optimization** - Indexes, connection pooling if needed
- **Error message improvements** - User-friendly error descriptions
- **Logging integration** - Proper tracing throughout

## 🔒 Privacy Design Notes

**Critical**: Actor ID rewriting prevents account linkage attacks:
- **Problem**: Same actor ID across aliases reveals account relationships
- **Solution**: Rewrite actor IDs per shared alias (`alias1-0`, `alias2-0`) 
- **Benefit**: Recipients cannot correlate aliases to same account
- **Implementation**: Only supports `id52-count` format for privacy rewriting

## 📋 Technical Debt

**Known Issues:**
- Intermittent list test failure (test isolation)
- Some functions still use global `eyre::Result` instead of specific errors
- Missing derive macro for document boilerplate elimination
- CLI uses dummy entity ID (needs real account integration)

## 🎯 Success Criteria

**For Next Milestone:**
- [ ] `#[derive(Document)]` macro working and tested
- [ ] All tests passing consistently (fix list test)
- [ ] `fastn run` command working without errors
- [ ] fastn-account fully integrated with type-safe documents
- [ ] Clean API documentation with examples

**For Production Ready:**
- [ ] Real entity IDs throughout (no more dummy values)
- [ ] Actor ID management CLI commands implemented
- [ ] Multi-device scenarios tested and working
- [ ] Comprehensive error handling with user-friendly messages
- [ ] Performance validated under load

---

*Last Updated: 2025-08-21*
*Status: Implementing derive macros for clean API*