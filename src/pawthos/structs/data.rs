use crate::commands::mimic::MimicError;
use crate::pawthos::enums::persistant_data::PersistantData;
use crate::pawthos::structs::mimic_db::MimicDB;
use crate::pawthos::structs::mimic_user::MimicUser;
use poise::serenity_prelude::UserId;
use tokio::sync::RwLock;

/// User data, which is stored and accessible in all command invocations
#[derive(Debug)]
pub struct Data {
    pub mimic_db: RwLock<MimicDB>,
    pub persistant_data_channel: tokio::sync::mpsc::Sender<PersistantData>,
}

impl Data {
    /// Read-only access to a user's [`MimicUser`], without exposing lock mechanics to callers.
    ///
    /// # What this does
    /// - Acquires a **shared** read lock (`RwLock::read`) on the in-memory [`MimicDB`].
    /// - Looks up the `user_id` and passes an `Option<&MimicUser>` to your closure `f`.
    ///   - `Some(&MimicUser)` if the user exists in the DB.
    ///   - `None` if the user has no record yet.
    /// - Returns the value produced by your closure.
    ///
    /// # Why `Option<&MimicUser>`?
    /// Reads should not implicitly create users. Returning `Option` lets call sites
    /// clearly handle the “no data yet” case without mutating the DB.
    ///
    /// # Locking and lifetimes
    /// - The read lock is **held only while** your closure runs. As soon as the closure
    ///
    ///   returns, the lock is released.
    /// - Your closure **must be synchronous** (non-`async`). This prevents holding the
    ///   lock across `.await` points—avoiding deadlocks and contention explosions.
    ///
    /// # Concurrency characteristics
    /// - Multiple calls to `with_user_read` can run concurrently. This is ideal for
    ///   highly parallel read paths like autocomplete, listing mimics, etc.
    ///
    /// # Panics
    ///
    /// - If your closure panics, the read lock is still released (via RAII). The panic
    ///   will unwind as usual.
    ///
    /// # Examples
    /// Read-only formatting of the current user's state:
    /// ```
    /// # use poise::serenity_prelude::{UserId};
    /// # async fn demo(data: &Data, user_id: UserId) -> String {
    /// let summary = data.with_user_read(user_id, |maybe_user| {
    ///     match maybe_user {
    ///         Some(u) => format!(
    ///             "Active: {}\nMimics: {}",
    ///             u.active_mimic.as_ref().map(|m| m.name.as_str()).unwrap_or("<none>"),
    ///             u.mimics.len()
    ///         ),
    ///         None => "No mimics yet.".into(),
    ///     }
    /// }).await;
    ///
    /// # summary
    /// # }
    /// ```
    pub async fn with_user_read<R, F>(&self, user_id: UserId, f: F) -> Result<R, MimicError>
    where
        F: for<'a> FnOnce(&'a MimicUser) -> Result<R, MimicError>,
    {
        let db_guard = self.mimic_db.read().await;
        let maybe_user = db_guard.get_user(user_id);
        f(maybe_user.ok_or(MimicError::NoUserFound)?)
    }

    /// Mutable access to a user's [`MimicUser`] with **automatic persistence** of changes.
    ///
    /// # What this does
    /// - Acquires an **exclusive** write lock (`RwLock::write`) on the in-memory [`MimicDB`].
    ///
    /// - Ensures the user exists and hands your closure `f` a `&mut MimicUser` to edit.
    /// - After your closure returns, takes a **snapshot** (`clone`) of the entire [`MimicDB`]
    ///   while the write lock is still held, guaranteeing a consistent view.
    /// - Drops the write lock **before** awaiting the persistence send (never holds a lock
    ///   across `.await`).
    /// - Enqueues the snapshot over `persistant_data_channel` so your persistence task
    ///   can write it to disk (or other storage) out of band.
    /// - Returns the value produced by your closure.
    ///
    ///
    /// # Why snapshot under the lock?
    /// Cloning the DB while holding the write lock ensures the persisted state reflects
    /// exactly the mutations you just made. The lock is released **before** the `await`
    /// on the channel send, so database mutation latency stays minimal.
    ///
    /// # Closure requirements
    /// - The closure is **synchronous** (non-`async`) and should **only** perform in-memory
    ///   edits. Do not block or do I/O inside the closure—keep it fast to minimize the write
    ///   lock hold time.
    ///
    /// # Error handling
    /// - If the persistence send fails (e.g., channel closed), we log a warning and continue.
    ///   The in-memory changes still succeeded.
    ///
    /// # Performance considerations
    /// - This design favors **correctness and simplicity**. Cloning the `MimicDB` each write
    ///   is typically fine for small–moderate datasets. If your DB grows large or writes are
    ///   very frequent, consider:
    ///   - batching/smoothing saves (debounce, buffer, or periodic flush),
    ///   - persisting only the diff or user-level payload,
    ///   - moving to an embedded store (sled/sqlite) with its own WAL.
    ///
    /// # Panics
    /// - If your closure panics, the write lock is released via RAII and no snapshot is sent.
    ///
    /// # Examples
    /// Add a mimic and set it active; return the added name for UX:
    /// ```
    /// # use poise::serenity_prelude::UserId;
    /// # async fn add_one(data: &Data, user_id: UserId) -> String {
    /// let name = "Cool Fox".to_string();
    /// let avatar = Some("https://…".to_string());
    ///
    /// let chosen = data.with_user_write(user_id, |user| {
    ///     let m = Mimic { name: name.clone(), avatar_url: avatar.clone() };
    ///     user.mimics.push(m.clone());
    ///     user.active_mimic = Some(m);
    ///     name.clone()
    ///
    /// }).await;
    ///
    ///
    /// // persistence already enqueued; just use `chosen` in your response
    /// chosen
    /// # }
    /// ```
    pub async fn with_user_write<T, F>(&self, user_id: UserId, f: F) -> Result<T, MimicError>
    where
        F: for<'a> FnOnce(&'a mut MimicUser) -> Result<T, MimicError>,
    {
        let mut db_guard = self.mimic_db.write().await;
        // Mutate in-memory under write lock
        let user_entry = db_guard.get_user_mut(user_id);
        let result = f(user_entry)?;
        // Capture a consistent snapshot while still holding the lock
        let snapshot = db_guard.clone();

        // Release the lock before any await
        drop(db_guard);

        if let Err(e) = self
            .persistant_data_channel
            .send(PersistantData::MimicDB(snapshot))
            .await
        {
            log::warn!("Failed to queue MimicDB save: {:?}", e);
        }
        Ok(result)
    }
}
