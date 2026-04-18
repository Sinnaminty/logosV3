# Shop Feature Implementation Plan

Phased blueprint for implementing the `/shop` suite plus supporting systems (inventory split, interaction faucet, gifting, achievements, and curated banner hosting).

Companion to `SHOP_IDEAS.md`. Read that first for the design intent; this doc covers *how* to build it.

---

## Scope

Eight coordinated features, implemented in 9 phases:

| Phase | Feature | Depends on |
|---|---|---|
| 0 | Foundation: `InventoryUser` split, `rand` crate, error type | — |
| 1 | Shop catalog + `/shop browse` + `/shop inventory` | 0 |
| 2 | Titles (priced + custom) | 1 |
| 3 | Paywall colorways (named + custom unlock) | 1 |
| 4 | Paywall banners + GitHub hosting setup | 1 |
| 5 | Tab-reaction faucet (new earning mechanism) | 0 |
| 6 | Gifting | 2–4 |
| 7 | Achievement badges (free, interaction-based) | 5 |
| 8 | Lootbox (with visible odds) | 0, 7 |

Each phase is independently shippable and revertible.

---

## Data Model

### New sub-struct: `InventoryUser`

Path: `src/pawthos/structs/inventory_user.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventoryUser {
    // Owned cosmetic IDs (reference entries in shop_catalog)
    #[serde(default)] pub owned_titles: Vec<String>,
    #[serde(default)] pub owned_colorways: Vec<String>,
    #[serde(default)] pub owned_banners: Vec<String>,
    #[serde(default)] pub owned_badges: Vec<String>,     // lootbox + achievement

    // Custom-text title (only settable if owned)
    #[serde(default)] pub custom_title: Option<String>,

    // Paywall unlocks for /profile set colorway <hex> and /profile set banner <url>
    #[serde(default)] pub unlocked_custom_colorway: bool,
    #[serde(default)] pub unlocked_custom_title: bool,
    #[serde(default)] pub unlocked_custom_banner: bool,

    // Interaction stats (for achievements + debugging)
    #[serde(default)] pub messages_sent: u64,
    #[serde(default)] pub gifts_sent: u32,
    #[serde(default)] pub gifts_received: u32,
    #[serde(default)] pub lootboxes_opened: u32,
    #[serde(default)] pub faucet_claims: u32,
    #[serde(default)] pub tabs_spent_lifetime: i64,

    // Achievement progress
    #[serde(default)] pub unlocked_achievements: Vec<String>,  // achievement IDs
}
```

### Changes to `ProfileUser`

`src/pawthos/structs/profile_user.rs` — add equipped-item pointers, keep existing fields for custom values:

```rust
pub struct ProfileUser {
    #[serde(default)] pub bio: Option<String>,
    #[serde(default)] pub banner_url: Option<String>,      // custom; gated by unlocked_custom_banner
    #[serde(default)] pub colorway: Option<u32>,           // custom hex; gated by unlocked_custom_colorway

    // NEW: equipped items (IDs into catalog)
    #[serde(default)] pub active_title_id: Option<String>,
    #[serde(default)] pub use_custom_title: bool,          // if true, render inventory.custom_title
    #[serde(default)] pub active_colorway_id: Option<String>,  // named; overrides custom hex
    #[serde(default)] pub active_banner_id: Option<String>,    // named; overrides custom url
    #[serde(default)] pub active_badge_ids: Vec<String>,       // max 3, shown in order

    // Legacy — migrate to inventory.owned_badges
    #[serde(default)] pub badges: Vec<Badge>,              // DEPRECATED after phase 2 migration
}
```

**Rendering priority** in `/profile view`:
- Colorway: `active_colorway_id` → catalog hex → `profile.colorway` (if unlocked) → default `LOGOS_GREEN`
- Banner: `active_banner_id` → catalog URL → `profile.banner_url` (if unlocked) → none
- Title: if `use_custom_title` → `inventory.custom_title`; else `active_title_id` → catalog name; else none
- Badges: render `active_badge_ids` lookups against a combined catalog+achievement list

### Shop Catalog

Path: `src/pawthos/structs/shop_catalog.rs` — static registries; adding items = code change.

```rust
pub enum Rarity { Common, Uncommon, Rare, Legendary, Achievement }
pub enum Category { Title, CustomTitleUnlock, Colorway, CustomColorwayUnlock,
                    Banner, CustomBannerUnlock, Badge, Lootbox, Token }

pub struct ShopItem {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub cost: i64,
    pub category: Category,
    pub rarity: Rarity,
}

pub struct TitleDef   { pub item: ShopItem }
pub struct ColorwayDef{ pub item: ShopItem, pub hex: u32 }
pub struct BannerDef  { pub item: ShopItem, pub url: &'static str }
pub struct BadgeDef   { pub item: ShopItem, pub emoji: &'static str }

pub const TITLES: &[TitleDef]       = &[ /* Tab Hoarder @ 10, etc. */ ];
pub const COLORWAYS: &[ColorwayDef] = &[ /* Sunset @ 20, etc. */ ];
pub const BANNERS: &[BannerDef]     = &[ /* Starfield @ 25, etc. */ ];
pub const LOOTBOX_POOL: &[BadgeDef] = &[ /* per-rarity pool */ ];
pub const UNLOCKS: &[ShopItem]      = &[
    ShopItem { id: "unlock_custom_title",    cost: 30, /* ... */ },
    ShopItem { id: "unlock_custom_colorway", cost: 30, /* ... */ },
    ShopItem { id: "unlock_custom_banner",   cost: 30, /* ... */ },
];

pub fn lookup(id: &str) -> Option<&'static ShopItem> { /* linear scan all tables */ }
```

### New error type: `InventoryError`

Path: `src/pawthos/enums/inventory_errors.rs`

```rust
#[derive(thiserror::Error, Debug)]
pub enum InventoryError {
    #[error("you don't have an inventory yet")]            NoUserFound,
    #[error("unknown item id: {0}")]                       UnknownItem(String),
    #[error("you already own {0}")]                        AlreadyOwned(String),
    #[error("you don't own {0}")]                          NotOwned(String),
    #[error("you can't gift items to yourself")]           GiftToSelf,
    #[error("{0} already owns {1}")]                       RecipientOwns(String, String),
    #[error("custom {0} feature is locked — unlock it in the shop first")]
                                                           FeatureLocked(&'static str),
    #[error("can't equip more than {max} badges (you tried {attempted})")]
                                                           TooManyBadges { max: usize, attempted: usize },
    #[error("custom {field} exceeds {max}-char limit")]    CustomTooLong { field: &'static str, max: usize },
    #[error("no faucet bounty on that message")]           NoFaucetBounty,
    #[error("wallet error: {0}")]                          Wallet(#[from] WalletError),
}
```

Add `#[from]` variant to `PawthosErrors`.

### New consts

Add to `src/pawthos/consts/mod.rs`:

```rust
pub const GIFT_FEE: i64 = 2;
pub const MAX_ACTIVE_BADGES: usize = 3;
pub const MAX_CUSTOM_TITLE_LEN: usize = 32;

// Faucet tuning
pub const FAUCET_TRIGGER_CHANCE: f64 = 0.005;  // 0.5% per message
pub const FAUCET_REWARD: i64 = 5;
pub const FAUCET_EXPIRY_SECS: i64 = 600;       // 10 min before reaction removed
pub const FAUCET_GLOBAL_COOLDOWN_SECS: i64 = 120;  // min gap between any two bounties

// Lootbox tuning
pub const LOOTBOX_COST: i64 = 15;
pub const LOOTBOX_SALVAGE: i64 = 3;
pub const LOOTBOX_CHANCE_COMMON: f64    = 0.60;
pub const LOOTBOX_CHANCE_UNCOMMON: f64  = 0.25;
pub const LOOTBOX_CHANCE_RARE: f64      = 0.10;
pub const LOOTBOX_CHANCE_LEGENDARY: f64 = 0.05;
```

---

## Phase 0: Foundation

### Files to change

| File | Action |
|---|---|
| `Cargo.toml` | Add `rand = "0.8"` |
| `src/pawthos/structs/inventory_user.rs` | **NEW** — the sub-struct above |
| `src/pawthos/enums/inventory_errors.rs` | **NEW** — error type above |
| `src/pawthos/structs/user.rs` | Add `inventory: InventoryUser` field with `#[serde(default)]` |
| `src/pawthos/traits/mod.rs` | Add `InventoryDbMarker` + `impl_user_db_spec!` line |
| `src/pawthos/structs/data.rs` | Add `def_db_access!` for inventory |
| `src/pawthos/enums/pawthos_errors.rs` | Add `#[from] InventoryError` variant |
| `src/pawthos/structs/mod.rs`, `enums/mod.rs` | Export new modules |

### Steps

1. **Add `rand` dep** — `cargo add rand` (or hand-edit Cargo.toml). Use `rand = "0.8"` since 0.9 has API churn.
2. **Create `inventory_errors.rs`** with the full enum above. Register in `enums/mod.rs`.
3. **Create `inventory_user.rs`** — empty `InventoryUser` struct first, add methods in later phases.
4. **Extend `User`** by adding `pub inventory: InventoryUser`. The `#[serde(default)]` ensures old `user.json` records load cleanly (same pattern WalletUser uses).
5. **Wire the marker trait:**
   - In `traits/mod.rs` add `pub struct InventoryDbMarker;` and `impl_user_db_spec!(InventoryDbMarker, InventoryUser, inventory);`
   - In `data.rs` add `def_db_access!(with_inventory_user_read, with_inventory_user_write, InventoryDbMarker, InventoryUser, InventoryError, InventoryError::NoUserFound);`
6. **Add error variant** — `#[error("InventoryError: {0}")] Inventory(#[from] InventoryError),` in `pawthos_errors.rs`.
7. **Export everything** — update `structs/mod.rs` and `enums/mod.rs`.

### Verify

- `cargo check` passes
- Bot starts; `user.json` loads unchanged; every user gets a fresh default `inventory` block
- No new commands or behavior changes

### Rollback

Revert the commit. Existing `user.json` records won't have the `inventory` field but they loaded fine without it when deserializing (and will continue to if we revert).

---

## Phase 1: Shop catalog + `/shop browse` + `/shop inventory`

### Files

| File | Action |
|---|---|
| `src/pawthos/structs/shop_catalog.rs` | **NEW** — catalog module |
| `src/commands/shop/mod.rs` | **NEW** — parent command + browse + inventory |
| `src/commands/mod.rs` | Register `shop()` in `return_commands()` |

### Catalog content (initial)

**Titles** (10 tabs each): `Tab Hoarder`, `Early Adopter`, `Certified Gremlin`, `Caffeine Dependent`, `Professional Lurker`, `Night Owl`, `Early Bird`, `Keyboard Warrior`.

**Colorways** (20 tabs each): `Sunset (0xFF6B6B)`, `Ocean (0x4A90E2)`, `Neon Pink (0xFF1493)`, `Midnight (0x1A1A3E)`, `Gold (0xFFD700)`, `Lavender (0xB57EDC)`, `Crimson (0xDC143C)`, `Mint (0x98D8A1)`.

**Banners** (25 tabs standard, 50 premium): stubs for Phase 4.

**Unlocks** (30 tabs each): Custom Title Unlock, Custom Colorway Unlock, Custom Banner Unlock.

### Commands

```rust
/// Shop commands — spend tabs on cosmetics.
#[poise::command(slash_command, subcommands("browse", "inventory"))]
pub async fn shop(_ctx: Context<'_>) -> Result { Ok(()) }

/// Browse available shop items, grouped by category.
#[poise::command(slash_command)]
pub async fn browse(ctx: Context<'_>, category: Option<String>) -> Result { ... }

/// Show what you own.
#[poise::command(slash_command)]
pub async fn inventory(ctx: Context<'_>) -> Result { ... }
```

`/shop browse` renders a paginated embed (reuse Poise's built-in paginator or build a simple one). Each category shows: name, cost, description, owned-checkmark if owned.

`/shop inventory` shows counts + item names grouped by category, plus equipped items at the top.

### Verify

- `/shop browse` lists all initial catalog items
- `/shop inventory` shows empty state for a fresh user
- No buy command yet — that arrives with each feature phase

### Rollback

Remove the files; remove the registration line.

---

## Phase 2: Titles

### Files

| File | Action |
|---|---|
| `src/commands/shop/mod.rs` | Add `buy` subcommand, or split into `shop/buy.rs` |
| `src/commands/profile/set.rs` | Add `title`, `customtitle`, `unset_title` |
| `src/commands/profile/mod.rs` | Render title in `view` |
| `src/pawthos/structs/inventory_user.rs` | Add purchase methods |

### Commands

```rust
// /shop buy title <id>
pub async fn buy_title(ctx: Context<'_>, #[autocomplete="title_ac"] id: String) -> Result

// /profile set title <id>     — equip owned title
pub async fn title(ctx: Context<'_>, #[autocomplete="owned_title_ac"] id: String) -> Result

// /profile set customtitle <text>  — requires unlock
pub async fn customtitle(ctx: Context<'_>, text: String) -> Result

// /profile unset title
pub async fn unset_title(ctx: Context<'_>) -> Result
```

### Purchase flow (reference pattern — reused for every paid feature)

```rust
pub async fn buy_title(ctx: Context<'_>, id: String) -> Result {
    let item = shop_catalog::lookup(&id).ok_or(InventoryError::UnknownItem(id.clone()))?;
    let user_id = ctx.author().id;

    // 1. Check ownership *before* charging
    ctx.data().with_inventory_user_read(user_id, |inv| {
        if inv.owned_titles.iter().any(|t| t == &id) {
            return Err(InventoryError::AlreadyOwned(item.name.into()));
        }
        Ok(())
    }).await?;

    // 2. Charge tabs (returns error if insufficient)
    ctx.data().with_wallet_user_write(user_id, |w| {
        w.remove_tabs(item.cost)
    }).await?;

    // 3. Grant item + update stats
    ctx.data().with_inventory_user_write(user_id, |inv| {
        inv.owned_titles.push(id.clone());
        inv.tabs_spent_lifetime += item.cost;
        Ok(())
    }).await?;

    ctx.send(utils::reply_ok("Purchased", ...)).await?;
    Ok(())
}
```

Note the **two separate writes**: wallet deduction is committed before inventory grant. If the inventory write fails after tabs are deducted, the user loses tabs without getting the item. Acceptable risk given the second write is an in-memory push that essentially can't fail. If we want stronger atomicity later, fold both into a single transaction on `UserDB` (new method on `Data` that acquires the outer `user_db` lock directly).

### `/profile view` render update

```rust
// After display_name, prepend title:
let title_line = render_title(&profile, &inventory);  // handles custom + catalog + none
let display = format!("{title_line}\n{display_name}'s Profile");
```

### Verify

- Buy a title → inventory shows it → equip it → `/profile view` renders it
- Buying duplicate rejected
- Buying without tabs rejected with `WalletError::NotEnoughTabs`
- Custom title without unlock rejected with `FeatureLocked("title")`
- Custom title >32 chars rejected

### Rollback

Revert files. Existing owned titles in JSON are orphaned fields — harmless; serde ignores them on the restored struct.

---

## Phase 3: Paywall colorways

### Paywall model

Two purchasable paths:
- **Named colorways** (20 tabs): buy by ID (`sunset`, `ocean`). Equip via `/profile set namedcolorway <id>`.
- **Custom colorway unlock** (30 tabs): one-time purchase. Enables `/profile set colorway <hex>`.

### Files

| File | Action |
|---|---|
| `src/commands/shop/mod.rs` | Add `buy colorway <id>`, `buy unlock colorway` |
| `src/commands/profile/set.rs` | Gate `colorway` behind unlock; add `namedcolorway`, `unset_colorway` |
| `src/commands/profile/mod.rs` | Update color resolution in `view` |

### Migration for existing users

Wire a one-shot migration in `framework::setup_framework` (after loading the DB, before serving):

```rust
fn migrate_grandfather_colorways(db: &mut UserDB) {
    for (_, user) in db.db.iter_mut() {
        // Grandfather: if they already have a custom colorway set, unlock it so they don't lose it
        if user.profile.colorway.is_some() && !user.inventory.unlocked_custom_colorway {
            user.inventory.unlocked_custom_colorway = true;
            log::info!("Migration: grandfathered colorway for user");
        }
    }
}
```

Run migration once, save, done. (Migration is idempotent — safe to leave running on every startup.)

### Gated `/profile set colorway`

```rust
pub async fn colorway(ctx: Context<'_>, color: String) -> Result {
    let user_id = ctx.author().id;
    ctx.data().with_inventory_user_read(user_id, |inv| {
        if !inv.unlocked_custom_colorway {
            return Err(InventoryError::FeatureLocked("colorway"));
        }
        Ok(())
    }).await?;
    // ...rest as before
}
```

### Verify

- Existing users with colorway set: still have it; `unlocked_custom_colorway == true`
- Fresh users: `/profile set colorway` rejected until they buy the unlock
- Named colorways: buy → equip → `/profile view` shows the named color; `/profile unset namedcolorway` reverts
- Named beats custom when both are set (rendering priority)

### Rollback

Revert; re-enable free colorway access. Previously-bought colorways remain in JSON (orphaned).

---

## Phase 4: Paywall banners + GitHub hosting

Same shape as Phase 3 but with hosted images. See the **GitHub Pages Setup** section below for URL generation.

### Catalog entries

```rust
pub const BANNERS: &[BannerDef] = &[
    BannerDef { item: ShopItem { id: "starfield", name: "Starfield", cost: 25, ... },
                url: "https://<you>.github.io/logosV3/banners/starfield.png" },
    // ...
];
```

### Commands

- `/shop buy banner <id>`
- `/shop buy unlock banner`
- `/profile set namedbanner <id>`
- `/profile unset banner`

### Migration

Grandfather users with existing `banner_url`:
```rust
if user.profile.banner_url.is_some() { user.inventory.unlocked_custom_banner = true; }
```

### Verify

Symmetric to colorway verification.

---

## Phase 5: Tab-reaction faucet

**The big one.** A new earning path based on spontaneous server interaction.

### Mechanic recap

1. On each guild message (at 0.5% probability, subject to a 2-min global cooldown): bot reacts with `TAB_EMOJI`.
2. The first non-bot user to click that same reaction: receives 5 tabs.
3. Both reactions (bot's + user's) are removed. Bounty is cleared.
4. If nobody claims within 10 minutes: bot silently removes its reaction; bounty expires.

### New shared state on `Data`

Path: `src/pawthos/structs/data.rs`

```rust
pub struct Data {
    // ...existing fields
    pub faucet_bounties: RwLock<HashMap<MessageId, BountyState>>,
    pub faucet_last_spawn: RwLock<DateTime<Utc>>,  // global cooldown
}

pub struct BountyState {
    pub channel_id: ChannelId,
    pub amount: i64,
    pub expires_at: DateTime<Utc>,
}
```

In-memory only. Bot restart = current bounties orphaned (bot's reaction lingers until clicked; no tabs awarded because bounty map is empty). Acceptable — restarts are rare and the orphan is cosmetic.

### Message handler extension

`src/handlers.rs::event_handler` — extend the `FullEvent::Message` branch:

```rust
// 1. Mimic auto-mode logic (existing — runs first)
// 2. NEW: faucet spawn logic (runs regardless of mimic, on the same message)
if !new_message.author.bot && new_message.guild_id.is_some() {
    if should_spawn_faucet(data).await {
        spawn_faucet_bounty(ctx, data, new_message).await;
    }
}
```

Where:
```rust
async fn should_spawn_faucet(data: &Data) -> bool {
    let cooldown_ok = {
        let last = data.faucet_last_spawn.read().await;
        (Utc::now() - *last).num_seconds() >= FAUCET_GLOBAL_COOLDOWN_SECS
    };
    if !cooldown_ok { return false; }
    rand::thread_rng().gen_bool(FAUCET_TRIGGER_CHANCE)
}

async fn spawn_faucet_bounty(ctx, data, msg) {
    // Add reaction, then — only if successful — record the bounty
    if msg.react(&ctx.http, tab_reaction()).await.is_ok() {
        let mut bounties = data.faucet_bounties.write().await;
        bounties.insert(msg.id, BountyState { ... });
        *data.faucet_last_spawn.write().await = Utc::now();
    }
}
```

### Reaction handler

Add a new branch to `FullEvent` matching in `event_handler`:

```rust
FullEvent::ReactionAdd { add_reaction } => {
    if add_reaction.user_id.is_some_and(|u| u == bot_user_id) { return Ok(()); }
    if !is_tab_emoji(&add_reaction.emoji) { return Ok(()); }

    let bounty_amount = {
        let mut bounties = data.faucet_bounties.write().await;
        bounties.remove(&add_reaction.message_id).map(|b| b.amount)
    };

    if let Some(amount) = bounty_amount {
        let uid = add_reaction.user_id.unwrap();
        data.with_wallet_user_write(uid, |w| { w.add_tabs(amount); Ok(()) }).await?;
        data.with_inventory_user_write(uid, |inv| { inv.faucet_claims += 1; Ok(()) }).await?;

        // Remove both reactions (bot's + claimer's)
        let chan = add_reaction.channel_id;
        let msg = add_reaction.message_id;
        chan.delete_reaction(&ctx.http, msg, Some(bot_user_id), tab_reaction()).await.ok();
        chan.delete_reaction(&ctx.http, msg, Some(uid),          tab_reaction()).await.ok();

        // No explicit notification — the disappearing reactions are the visual feedback.
        // The user can confirm the payout via /balance. (Achievement announcements,
        // if triggered, still post normally via check_achievements.)
        data.check_achievements(uid, add_reaction.channel_id, &ctx.http).await;
    }
}
```

**Requires new gateway intent:** `GUILD_MESSAGE_REACTIONS`. Add to `INTENTS` in `consts/mod.rs`. Also set Discord Developer Portal bot permissions to include "Add Reactions".

### Cleanup task

Spawn in `setup_framework` alongside the persistence task:

```rust
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let expired = {
            let bounties = data.faucet_bounties.read().await;
            bounties.iter().filter(|(_, b)| b.expires_at < Utc::now())
                .map(|(id, b)| (*id, b.channel_id)).collect::<Vec<_>>()
        };
        for (msg_id, chan_id) in expired {
            chan_id.delete_reaction(&http, msg_id, Some(bot_user_id), tab_reaction()).await.ok();
            data.faucet_bounties.write().await.remove(&msg_id);
        }
    }
});
```

### Emoji identity

`TAB_EMOJI` is currently the custom-emoji *string* `<:tab:1459045305084547123>`. For reactions you need a `ReactionType::Custom { animated: false, id: EmojiId::new(1459045305084547123), name: Some("tab".into()) }`. Add a helper `const TAB_EMOJI_ID: u64 = 1459045305084547123;` + `fn tab_reaction() -> ReactionType`.

### Verify

- Send 500 messages in a test channel → expect ~2–3 bot reactions at 0.5%
- Click the bot's tab reaction as a non-author user → get 5 tabs; both reactions disappear
- Click as the message author → same behavior (simplest UX; can exclude later)
- Wait 10 minutes without claiming → bot's reaction disappears; bounty gone
- Bot restart mid-bounty → reaction lingers, but clicking it awards nothing (bounty map is empty)

### Risks

- **Reaction intent not granted**: bot silently fails to spawn. Fix by adding intent.
- **Rate-limit from too-frequent reactions**: 0.5% + 2-min cooldown keeps us well under.
- **Bots double-reacting**: guard via `author.bot` check.
- **Concurrent claims**: `HashMap::remove` under `RwLock::write` is atomic — whoever gets the write lock first wins; second claimer sees `None` and is ignored.

### Rollback

Remove the reaction branch and the faucet spawn. The `faucet_bounties` map can be dropped. No data cleanup needed (stats counters are harmless).

---

## Phase 6: Gifting

### Model

Sender **purchases for someone else**. Pays `item.cost + GIFT_FEE` from their own wallet. Recipient receives the item.

### Command

```rust
// /shop gift <user> <category> <item_id>
#[poise::command(slash_command)]
pub async fn gift(
    ctx: Context<'_>,
    #[description = "Who to gift to"] recipient: serenity::User,
    #[description = "Category"] #[autocomplete = "category_ac"] category: String,
    #[description = "Item ID"] #[autocomplete = "giftable_item_ac"] id: String,
) -> Result
```

**Giftable categories**: titles, named colorways, named banners, badges (no lootbox pulls — those are services, not items).

### Flow

```rust
// 1. Validations
if recipient.id == sender.id { return Err(InventoryError::GiftToSelf.into()); }
let item = shop_catalog::lookup(&id)?;
ctx.data().with_inventory_user_read(recipient.id, |inv| {
    if already_owns(inv, &category, &id) {
        return Err(InventoryError::RecipientOwns(recipient.name.clone(), item.name.into()));
    }
    Ok(())
}).await?;

// 2. Charge sender
let total = item.cost + GIFT_FEE;
ctx.data().with_wallet_user_write(sender.id, |w| w.remove_tabs(total)).await?;

// 3. Grant to recipient
ctx.data().with_inventory_user_write(recipient.id, |inv| {
    grant_item(inv, &category, &id);
    inv.gifts_received += 1;
    Ok(())
}).await?;

// 4. Record on sender
ctx.data().with_inventory_user_write(sender.id, |inv| {
    inv.gifts_sent += 1;
    inv.tabs_spent_lifetime += total;
    Ok(())
}).await?;

// 5. Announce publicly in the invocation channel (not a ping — embed with recipient mention)
```

### Abuse controls

- Self-gift rejected.
- Dup-gift rejected (recipient already owns).
- Fee makes alt-account farming unprofitable (sender loses at minimum the fee per transfer).
- Future: per-user daily gift count cap if abuse surfaces.

### Verify

- Gift a title → recipient inventory shows it, sender loses `cost + 2` tabs
- Gift to self rejected
- Gift to user who already owns it rejected
- Gift-count stats increment on both sides

### Rollback

Remove gift command + handler. Historical stat counters remain; harmless.

---

## Phase 7: Achievement badges

### Achievement definitions

Path: `src/pawthos/structs/shop_catalog.rs` (reuse file) — add an `ACHIEVEMENTS` list.

```rust
pub struct Achievement {
    pub id: &'static str,
    pub name: &'static str,
    pub emoji: &'static str,
    pub description: &'static str,
    pub check: fn(&InventoryUser, &WalletUser) -> bool,
}

pub const ACHIEVEMENTS: &[Achievement] = &[
    Achievement { id: "ach_chatterbox",    name: "Chatterbox",     emoji: "💬",
                  description: "Send 100 messages",
                  check: |i, _| i.messages_sent >= 100 },
    Achievement { id: "ach_lorekeeper",    name: "Lorekeeper",     emoji: "📚",
                  description: "Send 1,000 messages",
                  check: |i, _| i.messages_sent >= 1_000 },
    Achievement { id: "ach_spender",       name: "Spender",        emoji: "💸",
                  description: "Spend 50 tabs",
                  check: |i, _| i.tabs_spent_lifetime >= 50 },
    Achievement { id: "ach_whale",         name: "Whale",          emoji: "🐋",
                  description: "Spend 500 tabs",
                  check: |i, _| i.tabs_spent_lifetime >= 500 },
    Achievement { id: "ach_generous",      name: "Generous",       emoji: "🎁",
                  description: "Gift 1 item",
                  check: |i, _| i.gifts_sent >= 1 },
    Achievement { id: "ach_philanthropist",name: "Philanthropist", emoji: "💝",
                  description: "Gift 10 items",
                  check: |i, _| i.gifts_sent >= 10 },
    Achievement { id: "ach_beloved",       name: "Beloved",        emoji: "🫂",
                  description: "Receive 5 gifts",
                  check: |i, _| i.gifts_received >= 5 },
    Achievement { id: "ach_committed",     name: "Committed",      emoji: "🔥",
                  description: "7-day daily streak",
                  check: |_, w| w.current_streak >= 7 },
    Achievement { id: "ach_devoted",       name: "Devoted",        emoji: "🌟",
                  description: "30-day daily streak",
                  check: |_, w| w.current_streak >= 30 },
    Achievement { id: "ach_quick_fingers", name: "Quick Fingers",  emoji: "⚡",
                  description: "Claim 5 faucet drops",
                  check: |i, _| i.faucet_claims >= 5 },
    Achievement { id: "ach_first_spend",   name: "First Spend",    emoji: "🎉",
                  description: "Make your first shop purchase",
                  check: |i, _| i.tabs_spent_lifetime >= 1 },
    Achievement { id: "ach_treasure",      name: "Treasure Hunter",emoji: "💎",
                  description: "Open 10 lootboxes",
                  check: |i, _| i.lootboxes_opened >= 10 },
];
```

### Stats hook points (where counters are incremented)

| Event | Location | Counter |
|---|---|---|
| Any message (guild, non-bot) | `handlers.rs::event_handler` Message branch | `inv.messages_sent += 1` |
| Any tab removal | `WalletUser::remove_tabs` — already returns on success; wrap in helper that also increments `inv.tabs_spent_lifetime` OR increment at each call site (pick one) | `inv.tabs_spent_lifetime` |
| Gift sent / received | Phase 6 flow | `inv.gifts_sent` / `inv.gifts_received` |
| Faucet claim | Phase 5 reaction handler | `inv.faucet_claims` |
| Lootbox open | Phase 8 | `inv.lootboxes_opened` |

Cleanest pattern: a single helper on `Data`:
```rust
pub async fn record_tab_spend(&self, user_id: UserId, amount: i64) {
    let _ = self.with_inventory_user_write(user_id, |i| {
        i.tabs_spent_lifetime += amount; Ok(())
    }).await;
    self.check_achievements(user_id).await;
}
```

### Achievement evaluation

After any stat-updating write, call a centralized checker. The announcement posts as a normal (non-ephemeral) message in the channel where the triggering action happened, so the community sees it.

```rust
pub async fn check_achievements(
    &self,
    user_id: UserId,
    channel_id: ChannelId,
    http: &serenity::Http,
) {
    let (inv_snap, wallet_snap) = /* read both sub-structs */;
    let newly_unlocked: Vec<&'static Achievement> = ACHIEVEMENTS.iter()
        .filter(|a| (a.check)(&inv_snap, &wallet_snap))
        .filter(|a| !inv_snap.unlocked_achievements.iter().any(|x| x == a.id))
        .collect();
    if newly_unlocked.is_empty() { return; }

    self.with_inventory_user_write(user_id, |inv| {
        for a in &newly_unlocked {
            inv.unlocked_achievements.push(a.id.into());
            inv.owned_badges.push(a.id.into());  // achievement badges count as owned
        }
        Ok(())
    }).await.ok();

    for a in newly_unlocked {
        let content = format!(
            "🎉 <@{user_id}> unlocked an achievement: **{} {}**\n*{}*",
            a.emoji, a.name, a.description,
        );
        if let Err(e) = channel_id.say(http, content).await {
            log::warn!("Achievement announce failed: {e}");
        }
    }
}
```

Callers thread `channel_id` through:
- Message handler (`handlers.rs::event_handler` Message branch): `data.check_achievements(uid, new_message.channel_id, &ctx.http).await;`
- Slash commands (shop/profile/gift): `ctx.data().check_achievements(ctx.author().id, ctx.channel_id(), ctx.http()).await;`
- Faucet reaction handler: `data.check_achievements(uid, add_reaction.channel_id, &ctx.http).await;`

The recipient side of a gift uses the sender's invocation channel — fine, since that's where the community is watching. If the recipient isn't in that channel, they still get the unlock stored in their inventory; they just won't see the live announcement.

Run this after every stat update. Cost is cheap: 12 predicates running in-memory.

### New commands

- `/achievements` — list all achievements with ✅/🔒 for each
- `/achievements progress` — show progress toward unclaimed ones (e.g., `Chatterbox: 34/100`)

### Verify

- Send 100 messages → channel sees "🎉 @user unlocked an achievement: 💬 Chatterbox"
- Badge appears in `/shop inventory`
- `/achievements` reflects the unlock
- Already-unlocked achievements don't re-trigger
- Achievement triggered from a command posts in the invoking command's channel, not a DM

### Rollback

Stats counters harmless. Remove the evaluation code; remove `/achievements` command. Existing unlocked_achievements fields are orphaned.

---

## Phase 8: Lootbox with visible odds

### Command

`/shop buy lootbox` — 15 tabs per pull.

### Flow

```rust
pub async fn buy_lootbox(ctx: Context<'_>) -> Result {
    let uid = ctx.author().id;
    ctx.data().with_wallet_user_write(uid, |w| w.remove_tabs(LOOTBOX_COST)).await?;

    let rarity = roll_rarity(&mut rand::thread_rng());
    let pool: Vec<_> = LOOTBOX_POOL.iter().filter(|b| b.item.rarity == rarity).collect();
    let pull = pool[rand::thread_rng().gen_range(0..pool.len())];

    let already_owned = ctx.data().with_inventory_user_read(uid, |inv|
        Ok(inv.owned_badges.iter().any(|b| b == pull.item.id))).await?;

    let result = if already_owned {
        ctx.data().with_wallet_user_write(uid, |w| { w.add_tabs(LOOTBOX_SALVAGE); Ok(()) }).await?;
        format!("**Duplicate!** Salvaged for {LOOTBOX_SALVAGE} tabs.")
    } else {
        ctx.data().with_inventory_user_write(uid, |inv| {
            inv.owned_badges.push(pull.item.id.into());
            inv.lootboxes_opened += 1;
            Ok(())
        }).await?;
        format!("You pulled **{} {}** ({} — {}% chance)!",
                pull.emoji, pull.item.name, rarity_name(rarity), odds_for(rarity))
    };

    ctx.send(utils::reply_ok("Lootbox", result)).await?;
    ctx.data().check_achievements(uid).await;
    Ok(())
}
```

### Pool size (v1)

**10 lootbox badges total, separate from achievement badges.** Lootbox IDs use the `box_` prefix; achievement IDs use the `ach_` prefix. A user's `owned_badges` vec contains both kinds, distinguished by prefix when rendering or filtering.

| Rarity | Count | IDs (placeholder) |
|---|---|---|
| Common (60%) | 4 | `box_coffee`, `box_bookworm`, `box_pixel_pusher`, `box_night_owl` |
| Uncommon (25%) | 3 | `box_speedrunner`, `box_trailblazer`, `box_stargazer` |
| Rare (10%) | 2 | `box_alchemist`, `box_code_wizard` |
| Legendary (5%) | 1 | `box_void_walker` |

Start here, grow the pool over time by adding to `LOOTBOX_POOL`. No code changes needed beyond the new constant entries.

### Odds display

`/shop browse lootbox` renders a small table:
```
🟢 Common    60% — 4 items
🔵 Uncommon  25% — 3 items
🟣 Rare      10% — 2 items
🟡 Legendary  5% — 1 item

Each pull: 15 tabs · Duplicates salvage for 3 tabs
```

Odds are in consts — show them directly so there's no mismatch.

### Verify

- Buy 100 lootboxes in a test run — verify rarity distribution roughly matches (60/25/10/5 ±3%)
- Duplicates return 3 tabs
- Odds visible in `/shop browse lootbox`
- Lootbox badges render distinctly from achievement badges (e.g., different section in `/shop inventory`, filtered by `box_`/`ach_` prefix)

### Rollback

Remove the command + pool. Existing owned lootbox badges stay in inventory (harmless).

---

## GitHub Pages Setup (for banner hosting)

You asked how to set up banner hosting on GitHub. Here are both viable options:

### Option A: GitHub Pages (recommended)

1. **Create the image folder** in the repo:
   ```
   /home/fizz/repos/logosV3/docs/banners/
   ```
   Drop PNG or JPG files here. Recommended: 1024×256 px, <500 KB each (Discord embeds downscale).

2. **Enable GitHub Pages**:
   - Go to `github.com/<your-username>/logosV3` → Settings → Pages
   - Under "Source" pick **"Deploy from a branch"**
   - Branch: `main`, folder: `/docs`
   - Click **Save**
   - Wait ~1 minute for the first deploy

3. **Your URLs** will be:
   ```
   https://<your-username>.github.io/logosV3/banners/starfield.png
   https://<your-username>.github.io/logosV3/banners/sakura.png
   ```
   Replace `<your-username>` with your GitHub username.

4. **Verify** by opening one URL in a browser — you should see the image.

5. **Hard-code in the catalog**:
   ```rust
   pub const BANNERS: &[BannerDef] = &[
       BannerDef {
           item: ShopItem { id: "starfield", name: "Starfield", cost: 25, ... },
           url: "https://<your-username>.github.io/logosV3/banners/starfield.png",
       },
       // ...
   ];
   ```

6. **Updating images later**: commit a new version to `docs/banners/`, push, Pages redeploys automatically. Keep filenames stable so catalog URLs don't break. If Discord caches an old version, appending `?v=2` to the URL busts the cache.

### Option B: Raw GitHub Content (no Pages setup)

1. Commit images to `assets/banners/` on `main`.
2. URL format: `https://raw.githubusercontent.com/<user>/logosV3/main/assets/banners/starfield.png`
3. **Caveats**: `raw.githubusercontent.com` has informal rate limits and doesn't set image-friendly headers; Discord sometimes refuses to embed. Works but fragile.

**Recommend Option A.** Five minutes of setup, works permanently, Discord embeds cleanly.

### Optional polish

Drop a simple `docs/index.html` so the Pages root renders a gallery — useful when adding new banners since you can visually verify them before committing to the catalog.

---

## Cross-cutting concerns

### Migration strategy

Two migrations run once at startup, both idempotent:

1. **Grandfather custom colorway/banner** (Phase 3–4): if `profile.colorway.is_some()` → `inventory.unlocked_custom_colorway = true`. Same for banner. Log each migration for observability.
2. **Badge format migration** (Phase 2 optional): `profile.badges` (`Vec<Badge>`) → `inventory.owned_badges` (`Vec<String>`). Only needed if any badges are already in production. If the live DB has none, skip.

Both run in `framework::setup_framework` before starting the bot loop. Snapshot the DB immediately after migration so persistence picks up the changes.

### Locking discipline

All purchases follow this pattern:
1. Read check (does user own? can they afford?)
2. Wallet write (deduct tabs — returns `NotEnoughTabs` on fail, leaves balance alone)
3. Inventory write (grant item, update stats)
4. Fire-and-forget achievement check

Steps 2 and 3 are not atomic across the two sub-structs — but both are in-memory operations backed by the same `RwLock<UserDB>`, so within each individual write the DB is consistent. The small window between step 2 and step 3 is where a crash could leak tabs. Acceptable for MVP. If this ever causes a real incident, add a single `Data::purchase(user_id, cost, grant_fn)` method that holds one write lock across both operations.

### Gateway intents

Phase 5 requires `GUILD_MESSAGE_REACTIONS`. Update `consts/mod.rs::INTENTS`:
```rust
let mut r = GatewayIntents::GUILD_MESSAGES;
r = r.union(GatewayIntents::DIRECT_MESSAGES);
r = r.union(GatewayIntents::MESSAGE_CONTENT);
r = r.union(GatewayIntents::GUILD_MESSAGE_REACTIONS);  // NEW
```

Verify in the Discord Developer Portal that the bot has "Add Reactions" permission on the target server.

### Testing strategy

- **Unit tests**: per-method on `InventoryUser` (add_title, equip_title, grant_badge, etc.). The existing wallet test pattern should carry over.
- **Migration test**: load a fixture `user.json` without `inventory` field → assert it loads cleanly with defaults.
- **Purchase test**: mock `Data`, call `buy_title`, assert wallet -cost, inventory +item.
- **Faucet test**: hard to unit-test end-to-end; add a method-level test for `should_spawn_faucet` with a seeded RNG, plus a method-level test for the reaction-award logic given a bountied message.

### Observability

Add `log::info!` at:
- Faucet spawn ("Spawned faucet bounty in channel X")
- Faucet claim ("User Y claimed bounty, awarded Z tabs")
- Achievement unlock ("User Y unlocked achievement Z")
- Migration events
- Gift transactions

---

## Risks

| # | Risk | Mitigation |
|---|---|---|
| 1 | Existing users lose their banner/colorway when paywall lands | Grandfather migration (Phase 3–4) |
| 2 | Two-step purchase (wallet deduct + inventory grant) can leak tabs on crash | Accept for MVP; add unified `Data::purchase` method if incidents occur |
| 3 | Faucet reaction spam/gaming | 0.5% trigger rate + 2-min global cooldown; only bot-initiated reactions award tabs |
| 4 | Bounty persistence lost on bot restart | In-memory only; leftover reaction is cosmetic — no tabs awarded on claim of orphaned bounty |
| 5 | Reaction intent / perm missing | Document in deploy checklist; bot silently no-ops on reaction add if intent absent |
| 6 | Badge ID collision between lootbox and achievement pools | Namespace: `box_*` vs `ach_*` prefixes |
| 7 | Gift abuse via alt accounts | 2-tab fee makes farming unprofitable; add daily gift cap if exploited |
| 8 | Lootbox perceived as rigged | Odds published in `/shop browse lootbox`; `rand::thread_rng` is cryptographically seeded |
| 9 | Achievement predicates run on every stat update → perf | Only ~12 predicates, in-memory checks; cost is trivial even at scale |
| 10 | Migration runs on every startup | Idempotent by design (checks `if !unlocked` before setting) — safe |

---

## Rollback (overall)

Each phase is a separate commit. To roll back any phase, `git revert` that commit. Data model is forward-compatible (all new fields use `#[serde(default)]`), so reverting the code leaves stale fields in `user.json` that serde will ignore on the restored struct. No destructive data migrations anywhere.

For a full rollback:
1. Revert phases in reverse order (8 → 0).
2. Ship the reverted build.
3. The bot continues serving; stale fields in `user.json` are silently dropped on the next save.

---

## Implementation order suggestion

Ship Phase 0 + 1 + 2 together as the first release — that's the foundation plus a single working feature (titles). It validates the architecture end-to-end before investing in the larger phases.

Then ship Phase 5 (faucet) standalone — it's a self-contained earning mechanism and users will appreciate the new tab source before the next round of sinks lands.

Then Phases 3, 4, 6, 7, 8 in any order based on what feels most fun to build.

---

## Resolved decisions

These were the open questions; all now locked in:

1. ✅ **Faucet self-claim allowed.** Message author can click their own bot-placed tab reaction.
2. ✅ **Achievement notification is an in-channel normal message** in the channel where the triggering action happened. Not a DM, not ephemeral. See Phase 7 for the `check_achievements(user_id, channel_id, http)` signature.
3. ✅ **`MAX_ACTIVE_BADGES = 3`** for profile display.
4. ✅ **Lootbox pool: 10 badges total** (4/3/2/1 split by rarity) — separate from achievement badges via `box_*` / `ach_*` ID prefixes. Grow the pool later by adding entries to `LOOTBOX_POOL`.
5. ✅ **Gifts announce in-channel only**, no DM to recipient.
