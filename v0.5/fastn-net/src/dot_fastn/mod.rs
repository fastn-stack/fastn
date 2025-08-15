//! The fastn folder
//!
//! The location of this folder is platform-specific, on Linux it is either
//! $HOME/.local/share/fastn or $XDG_DATA_HOME/fastn, on MacOS it is $HOME/Library/Application
//! Support/com.FifthTry.fastn and on Windows: {FOLDERID_RoamingAppData}\fastn\data which is usually
//! C:\Users\Alice\AppData\Roaming\FifthTry\fastn\data.
//!
//! The folder contains a lock file, `$fastn/fastn.lock, which is used to ensure only one instance
//! of `fastn` is running.
//!
//! The folder contains more folders like `identities`, `logs` and maybe `config.json` etc. in
//! the future.
//!
//! The identities folder is the most interesting one, it contains one folder for every identity
//! that exists on this machine. The content of single `identity` folder is described
//! in `identity/create.rs`.

mod init_if_required;
mod lock;

pub use init_if_required::init_if_required;
pub use lock::{FASTN_LOCK, MALAI_LOCK, exclusive, kulfi_lock_file, malai_lock_file};
