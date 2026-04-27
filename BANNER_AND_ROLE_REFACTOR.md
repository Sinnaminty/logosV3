# Banner & Role Refactor Plan

Two coordinated changes to the cosmetics surface that moved into scope after `SHOP_PLAN.md` Phase 4 was already designed. Companion to `SHOP_PLAN.md` — same tone, same phase shape, but focused on **deletions and consolidations** rather than feature growth.

> **Status: shipped.** Both changes have landed. See commits `951099c` (banner+colorway per-set refactor) and `464b7fb` (role commands moved into shop). The decisions chosen by the user diverge from this doc's original recommendations — see the **Resolved decisions (final)** section immediately below for the as-built model. The rest of the doc is historical record of the planning process.

## Resolved decisions (final)

These are the decisions actually implemented, overriding the recommendations in §1:

| Question | Decision | Notes |
|---|---|---|
| Banner cost model | **Per-set charge** (model b) | `BANNER_SET_COST = 10`. Clearing the banner is free. |
| Custom hex colorway | **Per-set charge** | `CUSTOM_COLORWAY_SET_COST = 5`. Equipping an *owned* named colorway is free. |
| Role colour | **Per-set charge** | `ROLE_COLOR_COST = 10` via `/shop buy rolecolor <hex>`. |
| Role name | **Per-set charge** (new capability) | `ROLE_NAME_COST = 10` via `/shop buy rolename <text>`. |
| `/color preview` | **Kept top-level** | Free; survives as the only subcommand of `/color`. |
| Refunds for prior unlock purchases | **Skipped** | Pre-release; no production data. |
| `unlocked_custom_banner` flag | **Removed** | Per-set model has no unlock state. |
| `unlocked_custom_colorway` flag | **Removed** | Same. |
| `unlocked_custom_title` flag | **Kept** | Custom title remains a one-time unlock (out of scope of this refactor). |

**TL;DR (original)**

1. **Drop preconfigured banners.** The `BannerDef` / `BANNERS` / `lookup_banner` catalog stack and the parallel `owned_banners` / `active_banner_id` storage all go away. Banners become user-supplied URL/attachment only, gated by a paywall.
2. **Move `/color set` into `/shop buy`.** All paid cosmetics live under `/shop buy <category>`; the top-level `/color set` is removed. `/color preview` (free) is a separate decision.

---

## 1. Open questions / decisions to confirm

These are the ambiguities the plan flushes out. Recommendations are marked **(rec.)**; everything else is genuinely undecided and the user should sign off before Phase A starts.

### 1.1 Banner cost model: one-time unlock vs. per-set charge

The phrase *"spend tabs to set their own banner"* admits two readings. Both ship cleanly; the difference is whether `/profile set banner <url>` is free after a single up-front payment, or rebillable on every call.

| Model | Mechanism | Stores | Behaviour |
|---|---|---|---|
| **(a) One-time unlock (rec.)** | Reuse existing `inventory.unlocked_custom_banner` flag. `/shop buy unlock` already grants it for 30 tabs (`shop_catalog.rs:308–312`). `/profile set banner <url>` is then free, like `customtitle` and custom `colorway`. | `unlocked_custom_banner: bool` | Pay once, set/clear/replace freely. |
| **(b) Per-set charge** | New `BANNER_SET_COST` constant. Every successful `/profile set banner <url>` runs a `with_wallet_user_write(_, w.remove_tabs(N))` step before the profile mutation. | Nothing new — the unlock flag becomes obsolete. | Pay every time. Clearing is free; setting always costs. |

**Recommendation: (a) one-time unlock.**

Reasons:
- Symmetric with `unlock_custom_title` and `unlock_custom_colorway` (`shop_catalog.rs:299–307`); the paywall-unlock model is already a project convention.
- Existing migration in `framework.rs:103` grandfathers users who already have `banner_url` set — they keep their banner with no extra work.
- The grandfather logic would need redesign or removal under model (b).
- `/profile set banner <url>` already has the gate at `set.rs:75–82` (`InventoryError::FeatureLocked("banner")`); zero command-flow changes.

If the user prefers (b), the plan changes are noted inline in §2–§5 below.

### 1.2 Fate of `/color preview`

`/color preview <hex>` (`commands/mod.rs:277–318`) is free — it generates a 256×256 PNG swatch and returns it as an attachment. It is *only* coupled to `/color set` by virtue of being in the same Poise subcommand group. Three options:

| Option | Outcome | Trade-off |
|---|---|---|
| **(i) Leave it as a top-level command (rec.)** | `/color preview` becomes a standalone slash command (no parent group, since `set` is gone). | Cheapest change; user-facing surface barely shifts (just `/color preview` instead of `/color preview` under a group). |
| **(ii) Move it under `/profile preview color`** | Lives next to other profile cosmetics. | Surface re-org; users have to relearn the path. Not really a "profile" thing — it's a generic colour-eyeballing tool. |
| **(iii) Delete it** | `/color` group disappears entirely. | Users lose a useful free tool. The PNG-generation code (`image` crate) still has callers? — verified: it's the only consumer. Removable cleanly. |

**Recommendation: (i) keep `/color preview` as a top-level command, drop the wrapping `color()` group.**

The Poise pattern is straightforward — promote `preview` from a `subcommands` entry on `color()` to a top-level `#[poise::command(slash_command)]` registered directly in `return_commands()` (`commands/mod.rs:40–58`). The function body is unchanged.

### 1.3 Should the `/color` parent command group disappear entirely, or just `set`?

Tied to 1.2. Under recommendation (i), the group disappears (since it would have one child — `preview` — and a single-child group is just bureaucracy). If the user picks (ii) or (iii), the group disappears for a different reason (move or delete). **Either way, the `color()` parent function in `commands/mod.rs:266–269` goes away.**

### 1.4 Grandfather existing `unlocked_custom_banner` purchases?

Some users have already bought the Custom Banner Unlock for 30 tabs. Under recommendation (a) — keep the unlock flag — those purchases continue to work; nothing to do.

Under (b) — drop the flag — those users paid for a feature that no longer exists. **Recommend grandfathering: refund 30 tabs to anyone with `unlocked_custom_banner == true` during the migration.** Concrete refund policy is parameterised by the chosen cost model.

### 1.5 Refund `owned_banners` entries?

`inventory.owned_banners` should always be empty in production today because `BANNERS: &[BannerDef] = &[]` (`shop_catalog.rs:192`) — the catalog has been an empty hole since Phase 1. **Verified by code reading: there is no shipped `banner_*` ID anywhere.** So the migration's drop step is a no-op for current production users. Code path still needs to handle non-empty for completeness in case a fixture or test DB has stale entries.

If for some reason `owned_banners` is non-empty in a real user.json — flag for inspection before running migration. If yes, refund 25 tabs per entry (matches Phase 4 spec'd cost) and log the refund.

### 1.6 Does `/profile unset banner` survive?

Yes — but rewired. Today (`unset.rs:57–73`) it clears both `active_banner_id` and `banner_url`. After the refactor `active_banner_id` no longer exists, so it just clears `banner_url`. The command stays so users can remove their banner without having to set it to a new one. Logic shrinks; surface is unchanged.

### 1.7 Naming clash: keep `/shop buy banner`?

The current `/shop buy banner` (`buy.rs:147–195`) takes a catalog ID and grants it to `owned_banners`. After the refactor, there's no catalog and no ownership vec — but we **still want a `/shop buy banner`** in the sense of "purchase the right to set a banner". Two phrasings:

| Phrasing | UX |
|---|---|
| **(α) `/shop buy banner <url>` does it all (rec.)** | One-shot: charge, grant unlock flag if not present, set `banner_url`. Combined unlock+set in one command. |
| **(β) Keep the existing `/shop buy unlock <id>` flow** | User runs `/shop buy unlock unlock_custom_banner`, then `/profile set banner <url>`. Two-step but reuses the unlock plumbing exactly. |

**Recommendation: (β) — change nothing about the unlock command, drop `/shop buy banner` entirely, and let users follow the existing two-step flow.**

Why: keeps the cost model consistent with `unlock_custom_title` and `unlock_custom_colorway` (those don't have `/shop buy title <text>` or `/shop buy colorway <hex>` shortcuts either — you buy the unlock, then set the value through `/profile set …`). Adds zero new code paths. The naming clash dissolves because the old `/shop buy banner` simply goes away.

If the user prefers (α), §3 spells out the additional command.

### 1.8 `/shop gift banner` — drop?

There is no longer anything to gift in the banner category — no catalog, and a banner-unlock can already be self-bought via `/shop buy unlock`. Drop the gift subcommand outright. Nobody can be gifted a "banner unlock" today either (gifts go through `gift.rs:29` and only handle `title`/`colorway`/`banner`). No regression.

If the user wants to add a "gift the unlock" option, that's a separate spec — note it but don't ship it in this refactor.

### 1.9 New constant: `COLOR_ROLE_COST` rename / location?

`COLOR_ROLE_COST = 10` (`consts/mod.rs:41`) currently parameterises `/color set`. After the move, the cost is parameterised by the new shop subcommand. The constant **stays at `consts/mod.rs`** (per architecture invariant: tunables live there); the only question is whether to rename it.

**Recommendation: leave the name `COLOR_ROLE_COST`** — it still describes what's bought (a coloured Discord role). Renaming would touch every call site for no semantic gain.

---

## 2. Data model deltas

Field-by-field walkthrough. All references are to current code; "**REMOVE**" / "**KEEP**" / "**ADD**" annotations are the post-refactor state.

### 2.1 `inventory_user.rs`

```rust
pub struct InventoryUser {
    pub owned_titles: Vec<String>,            // KEEP
    pub owned_colorways: Vec<String>,         // KEEP
    pub owned_banners: Vec<String>,           // REMOVE  (catalog gone, see §2.3)
    pub owned_badges: Vec<String>,            // KEEP
    pub custom_title: Option<String>,         // KEEP
    pub unlocked_custom_colorway: bool,       // KEEP
    pub unlocked_custom_title: bool,          // KEEP
    pub unlocked_custom_banner: bool,         // KEEP if model (a); REMOVE if (b)
    pub messages_sent: u64,                   // KEEP
    pub gifts_sent: u32,                      // KEEP
    pub gifts_received: u32,                  // KEEP
    pub lootboxes_opened: u32,                // KEEP
    pub faucet_claims: u32,                   // KEEP
    pub tabs_spent_lifetime: i64,             // KEEP
    pub unlocked_achievements: Vec<String>,   // KEEP
}
```

Forward-compat note: every field has `#[serde(default)]`. **Verified** there is no `#[serde(deny_unknown_fields)]` anywhere in `src/` (grep returned zero hits). Therefore old `user.json` records with `owned_banners: ["banner_starfield"]` deserialise into a struct that no longer has `owned_banners` — serde **drops the field silently** (default behaviour). This is the desired outcome: stale field data is harmless and disappears on the next save.

### 2.2 `profile_user.rs`

```rust
pub struct ProfileUser {
    pub bio: Option<String>,                  // KEEP
    pub badges: Vec<Badge>,                   // KEEP (legacy field, untouched by this refactor)
    pub banner_url: Option<String>,           // KEEP — now the *only* banner storage
    pub colorway: Option<u32>,                // KEEP
    pub active_title_id: Option<String>,      // KEEP
    pub use_custom_title: bool,               // KEEP
    pub active_colorway_id: Option<String>,   // KEEP
    pub active_banner_id: Option<String>,     // REMOVE  (catalog gone)
    pub active_badge_ids: Vec<String>,        // KEEP
}
```

Same serde-default story: stale `active_banner_id: "banner_xyz"` strings in old user.json deserialise into a struct without that field; serde drops them.

### 2.3 `shop_catalog.rs`

| Item | Action |
|---|---|
| `BannerDef` struct (`:84–88`) | **REMOVE** |
| `BANNERS: &[BannerDef]` (`:192`) | **REMOVE** |
| `lookup_banner(id)` (`:352–354`) | **REMOVE** |
| `lookup()` chain entry that iterates BANNERS (`:335`) | **REMOVE** the `.chain(BANNERS.iter()…)` link |
| `Category::Banner` enum variant (`:48`) | **REMOVE** — only `BannerDef` references it; safe to drop. (Verify with `cargo check` once the variant goes — there's a chance `Category` is matched elsewhere; if so, those arms get pruned too.) |
| `unlock_custom_banner` entry in `UNLOCKS` (`:308–312`) | **KEEP** under model (a); **REMOVE** under model (b) |

### 2.4 `consts/mod.rs`

Under model (a): no changes.

Under model (b): add `pub const BANNER_SET_COST: i64 = N;` (suggested N = 5, smaller than the 30 tab one-time unlock would have totalled across a few sets — but pricing is the user's call).

### 2.5 `color_errors.rs`

Stays. The `ColorError::IncorrectFormat` variant is still raised by `/color preview` and by the new `/shop buy color <hex>` (which will reuse the parsing logic). `ColorError::ImageError` is still used by `preview`.

---

## 3. Command surface changes

Concrete table. **Bold** = user-visible behaviour change.

| Command | Before | After |
|---|---|---|
| `/shop buy title <id>` | unchanged | unchanged |
| `/shop buy colorway <id>` | unchanged | unchanged |
| `/shop buy banner <id>` | Buys catalog banner, grants `owned_banners.push(id)` | **REMOVED** (no catalog anymore) |
| `/shop buy unlock <id>` | Three IDs valid: title, colorway, banner | model (a): unchanged. model (b): drop the `unlock_custom_banner` autocomplete entry from `UNLOCKS`. |
| `/shop buy lootbox` | unchanged | unchanged |
| `/shop buy color <name> <hex>` | — | **NEW**: replaces `/color set`. Charges `COLOR_ROLE_COST` tabs and creates/edits the user's Discord colour role. Verbatim flow from `commands/mod.rs::set` (lines 333–398) lifted under `commands/shop/buy.rs`. `guild_only` flag preserved. |
| `/shop gift title <user> <id>` | unchanged | unchanged |
| `/shop gift colorway <user> <id>` | unchanged | unchanged |
| `/shop gift banner <user> <id>` | Gifts a catalog banner | **REMOVED** (no catalog) |
| `/profile set bio` | unchanged | unchanged |
| `/profile set banner <url\|attachment>` | Gated by `unlocked_custom_banner`; sets `profile.banner_url`; clears `active_banner_id` if URL present | model (a): unchanged. model (b): replace the unlock check with a `with_wallet_user_write(_, remove_tabs(BANNER_SET_COST))` call. Remove the `active_banner_id = None` line either way. |
| `/profile set namedbanner <id>` | Equips an owned named banner | **REMOVED** |
| `/profile set colorway <hex>` | unchanged | unchanged |
| `/profile set namedcolorway <id>` | unchanged | unchanged |
| `/profile set title <id>` | unchanged | unchanged |
| `/profile set customtitle <text>` | unchanged | unchanged |
| `/profile set badges …` | unchanged | unchanged |
| `/profile unset title` | unchanged | unchanged |
| `/profile unset colorway` | unchanged | unchanged |
| `/profile unset banner` | Clears `active_banner_id` and `banner_url` | Now just clears `banner_url`. (Body of `unset.rs::banner` shrinks by one line.) |
| `/profile unset badges` | unchanged | unchanged |
| `/color preview <hex>` | Free PNG swatch | **(rec.)** stays — promoted from subcommand of `color()` group to a top-level slash command. (See §1.2.) |
| `/color set <name> <hex>` | Charges `COLOR_ROLE_COST`, creates/updates colour role | **REMOVED** — replaced by `/shop buy color`. |
| `/color` (parent group) | Holds `preview` + `set` | **REMOVED** — group dissolves. |

### 3.1 Sketch of `/shop buy color`

Mirrors the `unlock` purchase flow (charge then mutate), but the mutation is a Discord guild API call instead of an inventory write. Lifted from `commands/mod.rs::set`:

```rust
/// Buy a custom colour role. Costs COLOR_ROLE_COST tabs.
#[poise::command(slash_command, guild_only)]
pub async fn color(
    ctx: Context<'_>,
    #[description = "Name of your role."] name: String,
    #[description = "Color of your role (hex)."] color: String,
) -> Result {
    let user_id = ctx.author().id;
    let guild_id = ctx.guild_id().unwrap();
    let name = '\u{200B}'.to_string() + &name;
    let trimmed = color.strip_prefix("0x").unwrap_or(&color);
    let color_int = u32::from_str_radix(trimmed, 16)
        .map_err(|_| ColorError::IncorrectFormat)?;
    let color = if color_int == 0 {
        serenity::Colour::from_rgb(1, 1, 1)
    } else {
        serenity::Colour::new(color_int)
    };

    // Existing pattern: do guild API work first, charge after.
    // (Body identical to commands/mod.rs::set lines 358–384.)

    let tabs = ctx.data()
        .with_wallet_user_write(user_id, |w| w.remove_tabs(COLOR_ROLE_COST))
        .await?;

    // Stats: this is a paid cosmetic, so update lifetime spend.
    ctx.data().with_inventory_user_write(user_id, |inv| {
        inv.tabs_spent_lifetime = inv.tabs_spent_lifetime.saturating_add(COLOR_ROLE_COST);
        Ok(())
    }).await?;

    ctx.send(utils::reply_ok("Shop Buy Color",
        format!("Your color has been set! You now have **{tabs} {TAB_EMOJI}!**"))).await?;

    ctx.data().check_achievements(user_id, ctx.channel_id(), ctx.http()).await;
    Ok(())
}
```

Two improvements over the original `/color set` while we're here:
1. **Update `tabs_spent_lifetime`.** The current `/color set` doesn't, which means `ach_first_spend` / `ach_spender` / `ach_whale` don't reflect colour-role purchases. Folding it under shop is the natural place to fix this.
2. **Call `check_achievements`.** Same reason — the paid cosmetic should evaluate achievements like every other shop purchase does.

These are two-line changes that close a pre-existing inconsistency. Worth doing in Phase A. Flag this as an intentional behaviour change in the changelog.

Add `color` to the subcommands list on `pub async fn buy` (`buy.rs:36`).

### 3.2 Promoting `/color preview`

In `commands/mod.rs::return_commands` (line 51), replace `color()` with `preview()`. Strip the `#[poise::command(slash_command, subcommands("preview", "set"))]` from `pub async fn color` (line 266) and delete the function. The `preview` function (`:277–318`) is already a standalone `#[poise::command(slash_command)]` — promoting it requires no body change, only the registration update.

---

## 4. Migration / backwards compat

A new function `migrate_drop_banner_catalog` lands in `framework.rs` next to `run_migrations` (currently at lines 94–119). The existing function already grandfathers `unlocked_custom_banner` from `banner_url`; we extend that file with the new logic.

**All migrations are idempotent.** Re-running on every startup is safe because each step has a "is this already done?" guard.

### 4.1 New migration code (model a)

```rust
fn migrate_drop_banner_catalog(user_db: &mut UserDB) {
    // No-op for users who have nothing to clean up.
    let mut banners_dropped = 0u32;
    let mut active_id_cleared = 0u32;
    let mut owned_refunded = 0u32;

    for user in user_db.db.values_mut() {
        // (a) Clear stale active_banner_id pointing at a catalog ID.
        //     After the struct change, this field no longer exists in code;
        //     the field is automatically dropped by serde on the next save.
        //     Nothing to do in code — listed here to make the lifecycle explicit.

        // (b) owned_banners — drop. Refund 25 tabs per entry as a kindness.
        //     Production: should be empty (BANNERS was always &[]). Defensive code.
        if !user.inventory.owned_banners.is_empty() {
            let refund = (user.inventory.owned_banners.len() as i64) * 25;
            user.wallet.add_tabs(refund);
            owned_refunded += user.inventory.owned_banners.len() as u32;
            user.inventory.owned_banners.clear();  // before the field is removed
        }
        // After the struct field is removed in the same commit, this loop
        // body simplifies to nothing — see Phase C step ordering.
    }

    if banners_dropped > 0 || active_id_cleared > 0 || owned_refunded > 0 {
        log::info!(
            "Banner catalog migration: refunded {owned_refunded} owned banner(s)"
        );
    }
}
```

### 4.2 Migration code (model b — additionally)

```rust
// Refund unlock_custom_banner purchases since the unlock no longer exists.
fn migrate_refund_banner_unlock(user_db: &mut UserDB) {
    let mut refunded = 0u32;
    for user in user_db.db.values_mut() {
        if user.inventory.unlocked_custom_banner {
            user.wallet.add_tabs(30);  // matches UNLOCKS cost
            user.inventory.unlocked_custom_banner = false;
            refunded += 1;
        }
    }
    if refunded > 0 {
        log::info!("Banner-unlock refund: refunded {refunded} user(s) for 30 tabs each");
    }
}
```

Run this **before** removing the `unlocked_custom_banner` field from `InventoryUser`, so the in-memory mutation has somewhere to write. Then remove the field; serde drops the now-unused boolean from disk on next save.

### 4.3 Stale `active_banner_id` strings on disk

When the field is removed from `ProfileUser`, old user.json values like `"active_banner_id": "banner_starfield"` deserialise into a struct that doesn't have that field. **Verified**: no `#[serde(deny_unknown_fields)]` in the project, so serde silently ignores unknown fields. Stale strings disappear on the next snapshot save (the persistence task's `save_user_db` serialises the in-memory struct and overwrites disk — see `framework.rs:52–58`).

No explicit migration code needed for this case.

### 4.4 Concrete migration wiring

In `framework.rs::run_migrations` (line 94), append:

```rust
migrate_drop_banner_catalog(user_db);          // always
migrate_refund_banner_unlock(user_db);         // only under model (b)
```

After `run_migrations` returns, the existing `setup_framework` flow continues unchanged. The first DB write (any user action) triggers a snapshot, which writes back the cleaned struct.

To force a snapshot at boot rather than waiting for the first user action: send a `PersistentData::UserDB(user_db.clone())` message into the persistence channel right after migrations. This is a one-line addition; worth doing for cleanliness.

---

## 5. Phased order with file-level steps

Three phases, each independently shippable and revertible. Each phase ends in a single commit.

### Phase A: Add `/shop buy color`, leave `/color set` intact

**Goal:** new path lands in production while old path still works. Users have a transition window (a few days minimum) to learn the new command before the old one disappears.

**Rough diff size:** small (~80 LOC added in `buy.rs`, ~3 LOC in `mod.rs`).

**Files touched:**

| File | Change |
|---|---|
| `src/commands/shop/buy.rs` | Add `pub async fn color(…)` mirroring `commands/mod.rs::set`; add `"color"` to the `subcommands(…)` macro on `buy()`; add `ColorError` import |
| `src/pawthos/enums/color_errors.rs` | Add `#[from]` glue if not already in `pawthos_errors.rs` (verify — `ColorError` is already wired since `/color set` uses it) |

**Test plan (manual):**
- `/shop buy color "TestRole" FF8800` — should create a role and deduct 10 tabs. Verify role appears and balance decreases.
- `/shop buy color "Updated" 00FF00` (same user) — should edit existing role, deduct another 10 tabs.
- `/shop buy color "Foo" notahex` — `ColorError::IncorrectFormat`.
- `/shop buy color "Foo" FF0000` while broke — `WalletError::NotEnoughTabs`.
- Verify `tabs_spent_lifetime` in `/shop inventory` increases by 10 per call.
- Verify achievement `ach_first_spend` triggers on first call for a user who hasn't spent before.
- `/color set "Old" 0000FF` — still works (parallel command).

**Rollback:** `git revert <Phase-A-commit>`. Removes the new path; existing `/color set` continues to work.

### Phase B: Remove `/color set`, dissolve `color()` group

**Goal:** retire the duplicate. Communicate the cutover to users in advance (Discord announcement / changelog post).

**Rough diff size:** small (~50 LOC removed from `commands/mod.rs`).

**Files touched:**

| File | Change |
|---|---|
| `src/commands/mod.rs` | Delete `pub async fn color`; delete `pub async fn set`; **(rec.)** keep `pub async fn preview` but strip its parent-group association — change `return_commands()`'s `color()` entry to `preview()` |
| `src/pawthos/consts/mod.rs` | Update doc comment on `COLOR_ROLE_COST` to point at `/shop buy color` instead of `/color set` |
| `README.md`, `CLAUDE.md` | Updated in Phase doc step (§6) |

**Decision dependencies:** §1.2 (fate of `/color preview`) and §1.3 (group dissolution).

**Test plan (manual):**
- `/color set …` — slash autocomplete should not list it. Confirms unregistration.
- `/color preview FF8800` — still works as a top-level command (under recommendation (i)).
- Run `cargo check` — should pass cleanly. Any orphaned `use` of `COLOR_ROLE_COST` in `commands/mod.rs` should be in the import block; remove if so.

**Rollback:** revert Phase B commit. Old code restored, both commands available again. Phase A's `/shop buy color` continues working independently.

### Phase C: Banner refactor

**Goal:** delete the catalog scaffolding. No new user-facing commands; existing `/profile set banner <url>` continues to work (gated by the unlock under model a, or paywalled per-set under model b).

**Rough diff size:** medium. Touches every banner reference: 3 struct fields removed, ~5 catalog constants/functions removed, ~6 command handlers shrunk or deleted.

**Files touched (in dependency order):**

1. `src/pawthos/structs/shop_catalog.rs` — remove `BannerDef`, `BANNERS`, `lookup_banner`, the `BANNERS.iter()` link in `lookup()` chain, and `Category::Banner`. (Under model b, also remove the `unlock_custom_banner` entry in `UNLOCKS`.)

2. `src/pawthos/structs/inventory_user.rs` — remove `pub owned_banners: Vec<String>`. (Under model b, also remove `pub unlocked_custom_banner: bool`.)

3. `src/pawthos/structs/profile_user.rs` — remove `pub active_banner_id: Option<String>`. Adjust the doc comment on `banner_url` to drop the "named banner takes precedence" language.

4. `src/commands/shop/buy.rs` — delete `pub async fn banner` (lines 147–195) and `buyable_banners` autocomplete (lines 305–315). Remove `"banner"` from the subcommands list on `buy()` (line 36). Remove `BANNERS` from the import block (line 24). Under model b, also delete the `unlock_custom_banner` arms from the `unlock` subcommand match statements (lines 218, 237, 248).

5. `src/commands/shop/gift.rs` — delete `pub async fn banner` (lines 92–119) and `giftable_banners` autocomplete (lines 246–256). Remove `"banner"` from the subcommands list on `gift()` (line 29). Remove `BANNERS` from imports (line 23).

6. `src/commands/profile/set.rs` — delete `pub async fn namedbanner` (lines 107–141) and `owned_banners_ac` (lines 367–384). Remove `"namedbanner"` from the subcommands list on `set()` (line 22–32). In `pub async fn banner` (lines 64–105), remove the `p.active_banner_id = None;` line (line 91) since the field no longer exists. **Under model b**, also replace the `unlocked` check (lines 75–82) with a `remove_tabs(BANNER_SET_COST)` call.

7. `src/commands/profile/unset.rs` — in `pub async fn banner` (lines 56–73), remove `p.active_banner_id = None;` (line 61). Function now just clears `banner_url`.

8. `src/commands/profile/mod.rs` — delete `fn resolve_banner` (lines 133–143) — or rather, simplify it to `profile.banner_url.clone()` and inline at the call site if there's only one. Verify with grep before deleting.

9. `src/commands/shop/mod.rs` — remove `BANNERS` from imports (line 17). In `browse` (lines 60–69), delete the `if !BANNERS.is_empty()` block. In `inventory`'s render summary (line 159–166), drop the `"**{}** banners"` count and the `inv.owned_banners.len()` arg. Delete `render_banners` (lines 201–213) and its `.field("Banners", …)` call (line 137).

10. `src/framework.rs` — add `migrate_drop_banner_catalog` (and under model b, `migrate_refund_banner_unlock`). Call from `run_migrations` (line 94). **Subtle ordering point:** the migration code mutates `inventory.owned_banners` *before* step 2 above removes the field. Either:
    - **(rec.)** ship steps 2 and 10 in the same commit, with the migration written against the post-removal struct. Since the production DB has empty `owned_banners` everywhere, the loop body is dead code and can be omitted; we just need the *logging* side, which references nothing struct-shaped.
    - **OR** ship a precursor commit that runs the migration against the pre-removal struct, then a follow-up commit that removes the field. Two commits, more steps, no real benefit.

   Going with the recommended single-commit approach: the migration becomes very small — just the model (b) refund logic, if applicable.

**Test plan (manual):**
- Run on a copy of production user.json — verify the bot starts cleanly and serialises back without errors.
- `/profile view` for a user with `banner_url` set — banner still renders.
- `/profile set banner https://…` — under model (a), gated by unlock as today; under model (b), charges `BANNER_SET_COST`.
- `/profile unset banner` — clears the banner.
- `/shop browse` — Banners section absent.
- `/shop inventory` — Banners section absent; counts row no longer mentions banners.
- `/shop buy banner …` (slash autocomplete) — no longer offered.
- `/shop gift banner …` — no longer offered.
- `/profile set namedbanner …` — no longer offered.
- Old user.json with `active_banner_id: "banner_xyz"` and `owned_banners: ["…"]` — loads cleanly, fields are silently dropped on next save (`cat user.json` after a write to confirm).
- Under model (b): a user who previously had `unlocked_custom_banner: true` — verify they got their 30 tabs back.

**Rollback:** Phase C is the largest revert. `git revert <Phase-C-commit>` restores all banner code. Stale data in user.json — `owned_banners`, `active_banner_id`, `unlocked_custom_banner` (model b only) — is gone if a save happened between deploy and rollback. Restoring the *fields* is fine; the *data* is not recoverable from the running DB. If this matters, document in deploy checklist: "After Phase C ships, rollback within N minutes / before any user actions" — but realistically, the lost data is exactly the data we already declared garbage, so the loss is by design. No code change needed for restorability.

### 5.1 Suggested ship cadence

- **Phase A**: ship immediately. Low-risk, additive only.
- **Phase B**: ship at least 24h after A so users can practise the new command. Announce in #announcements or equivalent.
- **Phase C**: ship anytime after A (independent of B). Could ship together with A as a single PR if the user is comfortable with the larger blast radius.

---

## 6. Documentation follow-ups

| File | Edits |
|---|---|
| `SHOP_PLAN.md` | Phase 4 section (lines 416–446) is now obsolete. **Recommend rewriting** to point at this document, keeping the original text as a historical note ("Phase 4 was originally planned as curated hosted images; superseded by `BANNER_AND_ROLE_REFACTOR.md`"). Also update line 1003's risk-table entry #1 ("Existing users lose their banner/colorway when paywall lands") to clarify that grandfather migrations now also cover banner-catalog-removal. Optionally update line 167 in README (which mentions "Phase 4 (paywall banners with hosted images) is the explicit pending hole" — that hole no longer exists; remove the sentence or rewrite to note Phase 4 as deferred indefinitely. |
| `SHOP_IDEAS.md` | Lines 41–48 (curated banner tiers) and line 87 (`banners: Vec<Banner>`) are misleading post-refactor. Mark the Banners section deprecated or rewrite to reflect "user-supplied URL only" model. Leave the tier-pricing prose as historical context if desired. |
| `README.md` | Line 14 (`/shop` row): drop "banners" from the `buy` list. Line 15 (`/color` row): rewrite from "Preview a hex colour … or spend 10 tabs to buy a custom colour role" to "Preview a hex colour as a 256×256 PNG swatch." (assuming recommendation (i)). Line 167 (Roadmap paragraph): update the Phase 4 mention as noted above. Line 117 (`shop/` tree): drop "banner" from the `buy.rs` and `gift.rs` subcommand lists. |
| `CLAUDE.md` | Line 38 needs a small touch: "Phase 4 (paywall banners) is pending — `BANNERS: &[BannerDef] = &[]` in `src/pawthos/structs/shop_catalog.rs` is the explicit hole" should change to "Phase 4 (curated banner catalog) was reworked into `BANNER_AND_ROLE_REFACTOR.md` — banners are now user-supplied URL only." |
| `/home/fizz/.claude/projects/-home-fizz-repos-logosV3/memory/project_architecture.md` | Line 36 mentions BANNERS being empty pending Phase 4 — update to reflect post-refactor reality. Line 37 reference to `active_banner_id` should be deleted. The `WalletUser` row at line 13 currently lists `/color` as one of its commands; after Phase B `/color` (the parent group) is gone, so update the `WalletUser` row to drop `/color` and verify `/color preview` is referenced under the right home (it stays free, hits no DB, so it's not really "wallet" — could be unbinned, or moved to a new "Misc" row). |

---

## 7. Risk register

| # | Risk | Likelihood | Severity | Mitigation |
|---|---|---|---|---|
| 1 | Old user.json has stale `active_banner_id`/`owned_banners`/etc., bot crashes on deserialise | Low | High | **Verified** no `#[serde(deny_unknown_fields)]` exists; stale fields silently dropped by serde. Existing pattern (Phase 0 used the same trick). |
| 2 | Pre-Phase-A user runs `/color set` mid-session; mid-Phase-B user runs `/color set` and gets "unknown command" | Mid | Low | Stagger phases: A→B with at least 24h gap. Discord announces deprecation. Old `/color set` still functional during Phase A. |
| 3 | User loses their banner during Phase C migration (we drop `active_banner_id` but their `banner_url` is None) | Low | Mid | Verified by reading: dropping `active_banner_id` only matters if the user equipped a *named* banner. **`BANNERS` has always been empty**, so there's no shipped data of users with `active_banner_id == Some(_)`. Defensive code in `resolve_banner` already falls through to `banner_url` (`profile/mod.rs:133–143`). |
| 4 | Pre-existing `unlocked_custom_banner: true` purchases stranded under model (b) | Mid (under model b) | Mid | Refund migration in `migrate_refund_banner_unlock`. Logged for observability. |
| 5 | Pending gift transactions in flight (sender used `/shop gift banner` just before Phase C ships) | Very low | Low | Slash commands are synchronous from the user's POV — no "pending" gifts. Phase C dropping the gift subcommand means new invocations get "unknown command", old ones already completed. No data corruption risk. |
| 6 | `tabs_spent_lifetime` divergence: Phase A introduces it for colour-role purchases, but historical purchases pre-Phase-A are not retroactively credited | Low | Trivial | Document the change in commit message. Achievement counters lag for affected users by at most one purchase. Not worth retro-migration code. |
| 7 | Naming collision: someone interprets `/shop buy color` as buying a *colorway* (the existing `colorway` subcommand) | Low | Low | Description text on the slash command should be unambiguous: "Buy a custom Discord colour role for X tabs." Distinct from "Buy a named profile colorway." Discord's autocomplete shows the description. |
| 8 | `Category::Banner` enum variant removal breaks an unrelated match arm we didn't grep | Low | Low | Run `cargo check` after Phase C; rustc enumerates all reachable matches. Trivial to fix any miss. |
| 9 | `image` crate becomes orphan if `/color preview` is deleted under option (iii) | Low | Trivial | Run `cargo build` and check warnings; remove the dep from Cargo.toml if unused. (Recommendation (i) keeps preview, so this risk only surfaces under option (iii).) |
| 10 | Migration runs but operator forgets to back up user.json | Mid | High (irreversible) | Hard advise: snapshot `user.json` to `user.json.pre-banner-refactor` before running the new build. Standard ops hygiene; the bot's atomic `.tmp`-then-rename doesn't help against a logical-corruption rollback. |

---

## 8. Resolved decisions

These are fully baked; no user input needed unless explicitly contesting a recommendation.

- **No `#[serde(deny_unknown_fields)]` exists in the project.** Stale fields in user.json are silently dropped — verified by grep across `src/`.
- **`BANNERS` is currently `&[]` in production.** The catalog was an empty hole since Phase 1 — no real user has `owned_banners` or `active_banner_id` populated with a real catalog ID. Migrations are defensive, but no real-data work is expected.
- **`/color set`'s body is portable as-is** to a new shop subcommand. The Discord API calls (member lookup, role create/edit, role assign) and the `\u{200B}` zero-width-space prefix convention all transplant verbatim. Only the call site changes.
- **Phase A and Phase C are independent.** They can ship in parallel PRs or as a single batched PR. Phase B requires Phase A (otherwise no replacement for `/color set` exists).
- **`/profile unset banner` survives** with one line removed.
- **Architectural invariants hold**: no new sub-struct, no new macro pair, no direct file I/O, all writes through `with_*_user_read/write`. This refactor is structural-but-shallow.

---

## 9. Open decisions summary (action items for user)

Quick checklist — mark each before greenlighting Phase A:

- [ ] **§1.1** — Cost model: (a) one-time unlock [recommended] or (b) per-set charge?
- [ ] **§1.2** — `/color preview`: (i) keep top-level [recommended], (ii) move under `/profile`, or (iii) delete?
- [ ] **§1.3** — implicit in §1.2 (group dissolves either way under recommendations).
- [ ] **§1.5** — Refund existing `owned_banners` entries? [recommended yes — but production should have none, so likely no-op]
- [ ] **§1.7** — `/shop buy banner <url>` shortcut: (α) one-shot combined or (β) reuse existing unlock+set [recommended]?
- [ ] **§1.9** — Rename `COLOR_ROLE_COST`? [recommended no]

Once these are confirmed, Phase A is ready to ship.
