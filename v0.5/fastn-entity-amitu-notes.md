# fastn-entity Original Notes

## Original Design Notes (Preserved from fastn-entity/amitu-notes.md)

lets create a new crate called fastn-entity. it will have a type Entity and
EntityManager. EntityManager will read all entries from provided path, and if
path is not provided it will read from dot-fastn folder. each entity is
stored in a folder named after entity's id52. each entity has a sqlite file call
db.sqlite. each entity has a id52 secret key. entity manager has a method to
create default entity if there are no entities. in fact we auto create default
entity if there are no entities and we are creating an instance of entity
manager. entity manager need not keep list of entities, as it can go stale, and
should read off disc when need be. entity manager has explicit method to
create a new entity as well. when creating an entity entity manager creates the
identity also, or maybe entity::create will take care of this logic. lets
add all this to README/lib.rs of the new crate.

the default behaviour for entity folder is to store entity.id52 file, its public
key, and get the private key from the keyring. does id52 crate take care of
reading secret key from keyring? if the entity.private-key is found it will be
read first. both .private-key and .id52 is an ERROR (we are strict). when an
identity is getting created we try to store the key in keyring by default.

how can multiple identity exist in new? i think this new is both new and load.
in new mode it should create a default entity but not in load. actually there is
more, we will need config.json in this folder to decide the last identity,
we will discuss this later, make it the new entity whenever a new entity is
created, and then we need which entities are online or offline, not all
entities are online all the time. so even more reason to have new vs load.

we should store online status for each entity, so update Entity struct. also
store last: String. actually we should store fastn_id52::PublicKey instead of
String when storing id52

## Evolution from fastn-entity

The original fastn-entity concept has evolved into the current three-entity system:

1. **Rig** - The coordinator (was the EntityManager concept)
2. **Account** - User identities with aliases (evolved from Entity)
3. **Device** - Client entities (planned, not yet implemented)

The key improvements:
- Separation of concerns: Rig manages, Accounts hold user data
- Three-database architecture for better isolation
- Explicit online/offline status tracking in Rig database
- Multi-alias support in Accounts for privacy