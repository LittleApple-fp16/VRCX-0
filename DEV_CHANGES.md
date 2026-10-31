# dev 分支改动与上游同步记录

核对日期：2026-10-31。同步前本地分支为 master（8820a76ad），远程默认分支为 master；当时不存在 dev。原版 Map1en/VRCX-0 master 为 f0b642a75，落后 140 个提交，自定义提交 2 个。

## 同步前的自定义改动

- 39c1bea91：所有账号必须手动登录。删除前端自动登录服务、倒计时与偏好设置；删除后端自动登录、后台重新认证和无界面启动自动选择上次账号逻辑及对应 IPC/类型。
- 启动不恢复全局 Cookie；主动选择已保存账号仍能使用该账号凭据。会话失效后回到登录；无界面登录需要显式 --login/-l。保留当前进程已登录会话的前后台切换。
- 增加启动 Cookie、登录页初始化和无界面启动回归测试；更新多语言文案、生成的绑定及手动登录说明。
- 增加 Windows 手动登录版 Actions 发布流程，原流程关闭更新检查、不生成更新签名，提供安装包、便携 ZIP 与校验值。
- 8820a76ad：会话失效的账号归属转换为字符串，修复类型问题。

原自定义提交共影响 74 个文件，详情：

```text
 .github/workflows/manual-login-release.yml         | 119 +++++
 MANUAL_LOGIN.md                                    |   9 +
 .../application/src/auth/auth_credentials/mod.rs   |   6 +-
 .../src/auth/auth_credentials/snapshot.rs          |  74 +--
 .../application/src/auth/auth_credentials/types.rs |  13 -
 .../src/auth/background_auth_recovery.rs           | 509 ---------------------
 .../src/auth/login_session/auto_login.rs           | 222 ---------
 .../src/auth/login_session/auto_login/tests.rs     | 497 --------------------
 crates/application/src/auth/login_session/mod.rs   |   2 -
 .../application/src/auth/login_session/runtime.rs  | 154 -------
 .../application/src/auth/login_session/service.rs  |  52 +--
 crates/application/src/auth/login_session/tests.rs | 335 +-------------
 crates/application/src/auth/mod.rs                 |  24 +-
 .../src/auth/noninteractive_auth/mod.rs            |   8 +-
 .../src/auth/noninteractive_auth/runtime.rs        | 498 --------------------
 .../src/auth/noninteractive_auth/service.rs        |  94 ----
 crates/composition/src/context.rs                  |   9 -
 crates/composition/src/state/auth_session.rs       |  55 +--
 crates/composition/src/state/background_auth.rs    | 119 -----
 crates/composition/src/state/mod.rs                |   7 +-
 crates/composition/src/state/runtime_host_state.rs |   3 -
 .../runtime_host_state/profile_bundle_tests.rs     |  43 ++
 crates/composition/src/state/startup.rs            |  10 +-
 crates/outbound-adapters/src/lib.rs                |   2 -
 .../outbound-adapters/src/noninteractive_auth.rs   | 100 ----
 crates/outbound-adapters/src/web_client.rs         |   5 +-
 .../tests/auth_credentials_secrets.rs              |   5 -
 .../tests/manual_login_startup.rs                  |  43 ++
 crates/runtime-host-desktop/src/state.rs           |  34 +-
 src-tauri/src/bindings_export.rs                   |   1 -
 src-tauri/src/bootstrap/adapters.rs                |   4 +-
 src-tauri/src/bootstrap/notification.rs            |  11 +-
 src-tauri/src/commands/host/window.rs              |   2 +-
 src-tauri/src/commands/vrchat/auth/service.rs      |  17 +-
 src/features/auth/LoginPage.test.tsx               |   2 -
 src/features/auth/LoginPage.tsx                    |   3 -
 src/features/auth/components/LoginFormCard.tsx     |   5 -
 src/features/auth/components/SavedAccountsCard.tsx |   3 -
 src/features/auth/useLoginAutoLogin.test.tsx       | 145 ------
 src/features/auth/useLoginAutoLogin.ts             | 162 -------
 src/features/auth/useLoginPageController.ts        |   2 -
 src/features/auth/useLoginPageState.test.tsx       | 122 +++++
 src/features/auth/useLoginPageState.ts             |  17 -
 .../settings-tabs/SettingsSystemTab.test.tsx       |   2 -
 .../components/settings-tabs/SettingsSystemTab.tsx |  40 --
 .../components/useSettingsSystemTabState.ts        |  15 -
 .../shellSystemSections.ts                         |   3 -
 .../settings/settingsPageStateSectionTypes.ts      |   1 -
 .../settings/settingsPageStateSections.test.ts     |   1 -
 .../settings/useSettingsMaintenanceActions.ts      |  25 -
 src/features/settings/useSettingsPageState.ts      |   2 -
 src/localization/cs.json                           |  35 +-
 src/localization/de.json                           |  35 +-
 src/localization/en.json                           |  35 +-
 src/localization/es.json                           |  35 +-
 src/localization/fr.json                           |  35 +-
 src/localization/ja.json                           |  35 +-
 src/localization/ko.json                           |  35 +-
 src/localization/pt.json                           |  35 +-
 src/localization/ru.json                           |  35 +-
 src/localization/zh-CN.json                        |  35 +-
 src/localization/zh-TW.json                        |  35 +-
 src/platform/tauri/bindings.ts                     |  21 -
 src/repositories/configKeys.ts                     |   2 -
 src/services/authAutoLoginService.test.ts          | 412 -----------------
 src/services/authAutoLoginService.ts               | 402 ----------------
 src/services/authExecutionService.test.ts          |  13 -
 src/services/authSessionRecoveryService.test.ts    |   6 +-
 src/services/authSnapshotService.test.ts           |  31 +-
 src/services/authSnapshotService.ts                |  42 +-
 .../preferences/preferenceSnapshotLoader.ts        |   8 -
 src/state/preferencesStore.test.ts                 |   2 -
 src/state/preferencesStore.ts                      |   8 -
 src/state/runtimeStore.ts                          |  11 +-
 74 files changed, 423 insertions(+), 4556 deletions(-)
```

## 本次 dev 改动

完整合并原版以下 140 个提交；冲突保留手动登录行为，并保留新版即时重连入口和设置页的搜索、定位、重置能力。移除上游新设置清单中残留的自动登录设置。主要新增内容包括商店与购买记录、群组角色与活动管理、实例名称/语言/活动、账号隔离模式、截图收藏与浏览优化、设置搜索与深层链接、通知分组与未读体验、连接重试与唤醒恢复、活动统计、社区主题更新，以及持久化、Cookie、媒体、遥测和依赖修复。

建立 dev 默认分支和独立发布链路：

- 推送 dev 触发 CI 与 Dev Release；Windows 成品只在 GitHub Actions 构建。
- 更新目录 API、清单下载地址和清单中的安装包地址统一使用 LittleApple-fp16/VRCX-0。安装入口拒绝原版及其他仓库的清单。
- 更新检查启用；使用独立 Tauri 签名私钥（加密存入 Actions Secret）和公钥。原版 Windows 证书配置移除。
- 每个构建生成递增的稳定语义版本标签、安装包、便携 ZIP、更新签名、latest_windows.json 和 SHA256SUMS.txt。
- 详细发布与版本规则见 MANUAL_LOGIN.md。

## 本次合并的上游提交

```text
f0b642a75 fix(notifications): resolve group names for group instance locations across all channels
a38aadbd7 docs(readme): add screenshot favorites and per-account isolation to the VRCX comparison
e38114fe2 style(notifications): draw tree branches for expanded notification groups
421fbd437 fix(charts): default community separation to 0.5 and scope mutual graph reset to layout settings
030623196 fix(charts): keep small mutual graph nodes visible by fixing the outer border width
721f67cb0 style(settings): mark modified settings with an accent edge bar and drop reset for switches
be59f79b4 style(sidebar): enlarge friend row avatars to 40px to match the self header
1915565ab fix(ci): sum badge download count across paginated release pages
113df9c69 feat(realtime): show reconnect state and retry-now in the status bar, flag the tray, and reconnect promptly after system resume
6b8e6b382 feat(activity): zero-fill the time series, page and pick overview windows, and explain empty states per platform
9d574e96e feat(empty-states): distinguish empty history from filtered results and add a first-run feed tip
502643020 refactor(shortcuts): route app shortcuts through a static registry and one guarded dispatcher
bbf6e9d55 feat(notifications): fold adjacent repeated notifications by a backend group key
c265ce3e5 refactor(status-bar): let each status bar segment subscribe to its own store slices
b1b2e822f fix(media): delay the viewer thumbnail so fast originals no longer jump in size
969837734 feat(telemetry): classify truncated responses and malformed tool arguments apart from invalid_args
67a66c76f refactor(screenshots): move favorite relink, prune and refresh rules into application behind store and library ports
11ac1d15f refactor(runtime-host-desktop): route the desktop widget and VR overlay facades through a per-platform imp module instead of per-method cfg branches
5ec09067b refactor(realtime): drive friend baseline tests through the causal watermark and drop the direct sync mode
e9a82ee65 refactor(vr-overlay): carry the device role in device snapshots instead of parsing it back from display labels
4ee268c9b chore(deps): record why rust-ini is pinned and drop the placeholder tracking line
d6c7ccffd refactor(groups): define the group role mutation once in contracts and drop its adapter copy
f26ca97f2 test(overlay): cover device list refresh and tracker disconnect watching apart from the headset snapshot
e06e8b77a test(groups): pin role, member, profile and post request mapping in the group request adapter
7870334b2 refactor: share the video error prefix, game log resume key, tauri block-on helper and instance field PUT builder
69cb23e11 chore: remove the unused avatar cache clear, current user status fields, location-without-instance localizer and overlay sink wrapper
ce9b68921 refactor(auth): keep cookie probe rejection wording in contracts and drop the unreachable two-factor branch
07e697c87 fix(clipboard): refuse oversized copied files before reading them and take the image type from the file content
caa8be33a fix(game-log-import): store imported rows and their stay durations in one transaction
7e0c935da fix(current-user): count an avatar wear checkpoint again when storing it fails
017c87fec fix(vrchat-client): send trimmed ids in group member, invite, ban and role requests and trimmed group event titles and times
afd0bc4a8 fix(logger): show connection failures in the recorder status and document the Windows recorder
b3d7c6100 refactor(logger-protocol): define the recorder snapshot frame once and read and write it through that type
25268dc18 test(feed-logger): cover recorder sync and presence through an injectable transport, keep the presence loop idle without a recorder and pass it only narrow dependencies
9963ff157 fix(mcp): resolve timeWindow calendar forms in the caller's utcOffsetMinutes and give the assistant the local date
7d9288a3e perf(persistence): index the logger backfill duplicate lookup and bootstrap global tables once while duplicate favorites remain
5c98a1417 fix(telemetry): report only the error kind for assistant LLM transport failures
f24741d18 perf(ipc): offload wear segment, local favorite order and reorder commands to blocking threads
9bc35b128 fix(activity): invalidate the page cache when video, portal or resource events extend an unclosed stay
44ef71086 fix(game-isolation): run tray toggles off the main thread and hide the local location and now playing while isolated
2158e142a feat(game): add game isolation mode for playing VRChat on another account, close #534
a91de3c4a feat(ui): navigate screenshots in grid order, preload neighbors, restore gallery focus and virtualize VRChat media grids
289a30053 feat(settings): add settings manifest, search, deep links, modified markers and quick search settings group
cf17c17af feat(user-dialog): add friend request banner, relationship badges, hover intent and refresh-stable tabs
2702851d3 feat(ui): hold new notifications while reading, add expiry, session unread, tab counts and theme safe mode
32c360136 fix(ui): align screenshot selection with grid order, keep quick search results while loading and guard repeat notification actions
193183a95 feat(ui): unify log list columns, timestamps, right-click filters and live insertion
19ef26f32 fix(ui): restore table column menus, header meta classes and stable log paging
816b50f38 feat(ui): restore scroll position on settings, tools and themes pages
2b00e6757 feat(sidebar): move focus between rows with arrow keys
6007fc52e refactor(ui): unify delayed loading reveal into useDelayedFlag
5a67911a1 chore: bump
a6b1b021e fix(persistence): adapt progress_handler to rusqlite 0.41 mutable borrow
579a595e4 chore(deps): bump react-resizable-panels from 4.14.8 to 4.15.0 (#533)
18cd17200 chore(deps): bump zip from 9.0.5 to 9.1.0 (#532)
43f067f4b chore(deps): bump rusqlite from 0.40.2 to 0.41.0 (#530)
eaf2d03df chore(deps): bump react-i18next from 17.1.6 to 17.1.8 (#531)
44683e556 chore(deps): bump i18next from 26.5.8 to 26.5.10 (#529)
883a54967 fix(ci): restore rust-ini 0.21.1 pin for deep-link on Linux and ignore it in dependabot
0027632f0 fix(user-dialog): return to main view after saving profile details
02d80b472 refactor(dialogs): rebuild invite message picker as shared list and dropdown components
70d721a07 feat(tools): add invite message option to invite request auto-reply
4e19d2a2a refactor(tools): share image upload intake and unify text entry checks
8c8d7a8d7 feat(tools): add image paste shortcuts to gallery and inventory (#528)
9fb88be1f chore: remove QQ group links
ca92a224d chore(deps): bump react-i18next from 17.1.3 to 17.1.6 (#525)
5886a5e55 chore(deps): bump i18next from 26.5.6 to 26.5.8 (#523)
cf29ddbc1 chore(deps): bump rust-ini from 0.21.1 to 0.21.3 (#527)
e0b70769a chore(deps): bump uuid from 1.28.0 to 1.29.0 (#526)
936354eef chore(deps): bump rmcp from 3.5.1 to 3.7.0 (#524)
786b73d8b test(persistence): start the vacuum only after the concurrent reader completes its first read
5ab5a1dda test(i18n): drop the stale French fallback assertion now that the desktop overlay labels are translated
2f71c1673 2.33.0
8c7441964 update fr.json (#506)
cb9e8175f feat(telemetry): report sign-in state and opted-out basic starts, with early heartbeats at one and five minutes
421d84d86 feat(assistant): include transport error causes and finish reasons in turn error summaries
0386fa6f1 feat(groups): translate group posts from a hover action that replaces and expands the body
7181fdc18 feat(groups): show group posts as cards grouped by day with relative times, edited marker and collapsible text
991dbe3f4 feat(tools): show quick access tools as a button grid with recent tools and edit toggle in its footer
02ae7570c feat(screenshots): remove favorites whose files can no longer be found
03f8eae9f feat(screenshots): favorite screenshots with a favorites view that follows moved files
2facf925a fix(instances): show the creator name instead of the user id in world instance history
a0c3d5a1d feat(gallery): paste clipboard images and image files into gallery, icon and print uploads
b38c35fcd feat(overlay): add a right-click menu to the status bar desktop overlay toggle
d158cb766 feat(notifications): show per-row progress while marking all notifications read
391315241 fix(overlay): fit wrist trackers by measured font width instead of estimates (#512)
512a00aba chore: bump
a277a9446 feat(ui): use a server icon for the self-hosted feed logger
85b85186f feat(overlay): show each wrist tracker battery while the header has room (#512)
5eddf1dd6 feat(assistant): fold tool calls into a shadcn trace with a running thinking line
13a7b7100 feat(settings): show MCP client configs in a tabbed dialog and label code blocks by language
12749b151 feat(ui): add shadcn code block for webhook examples, assistant code fences, and account standing JSON
d5bf5878f fix(assistant): return malformed and truncated tool calls to the model, surface broken LLM streams, and retry rate-limited requests
45ec8f269 fix(mcp): accept this year, last year, and a UTC calendar month as timeWindow
55c0fa882 perf(game-log): refresh only the live session head and decouple session paging from table page size (closes #521)
01816adc9 fix(ci): pin rust-ini 0.21.1 in manifest for deep-link on Linux
3870c58eb chore: bump
2bec0794d feat(telemetry): count group moderation and my groups opens from user and group dialogs
88c635893 feat(store): show avatar listings and ownership, purchased avatars, purchase history, live VRChat Credits, world and group store tabs, and unopened bundle counts
d65aabb19 feat(user): show own account standing from the user dialog menu and recognize moderation sanction notifications
fe9f3fadc feat(instances): list the current instance first and instances with friends before the rest in the world dialog
c53f5839c feat(instances): redesign world instance rows with aligned names, right-aligned language badges and an 18+ marker
24a63da45 feat(instances): rename instances, show instance names, languages and linked events, mark closed instances, read slim world instances, and show the active instance count
a5ef9bc4f feat(groups): manage roles, create and edit group events, link events to group instances, and handle invites and blocked groups in My Groups
d5fc4bc40 feat: report users through a moderation report dialog, reorder and delete avatar gallery images, and limit invite automation by status
3bac0c4fd feat(launch): switch avatar before joining from the launch dialog (closes #520)
85b2fb482 fix(updater): run application exit cleanup before the Windows installer exits the process
7b40e7f5c feat(sidebar): open world rooms in the world dialog and fall back on broken room images (closes #519)
57da82967 feat(reports): filter my reports by content type (closes #518)
564d92245 feat(profile): merge self profile editing into one panel with solid banners and VRC+ media locks
3842a6ced test(realtime): discard the expired pending-offline wake before it races the reconnect
27652049c fix(ci): pin rust-ini 0.21.1 for deep-link on Linux and export CommunityThemeUpdate
2211fbc19 chore(deps): bump tauri-plugin-deep-link from 2.6.1 to 2.6.2 (#517)
3c64a263d chore(deps): bump tauri-plugin-single-instance from 2.5.3 to 2.5.4 (#515)
a4b65ba65 chore(deps): bump getrandom from 0.3.4 to 0.4.3 (#516)
e505a6184 chore(deps): bump cn from 0.4.0 to 0.5.0 (#514)
27b745c9c feat(themes): check community themes for updates at startup and offer one-click updates
766dd311f fix(i18n): use Chinese double quotes instead of corner brackets in Chinese copy
56085ad1f fix(feed-logger): store the recorder password obfuscated so every machine sharing the data directory can use it
49f69b040 fix(presence): measure online duration from when the friend was seen coming online
fdd18e4c4 fix(secrets): keep each machine's auth secrets when a data directory is shared across machines
fd8acef25 feat(titlebar): change the background image from the title bar
78be931f9 feat(screenshots): let the random pick draw from subfolders too
ac489328e feat(app-menu): open Themes from the VRCX-0 menu
74eb2bbea feat(favorites): remember the last selected group on each favorites page
7d754630a test(feed-logger): give each settings test its own temp dir so parallel runs no longer share a locked database
20735a704 fix(mcp): accept a UTC calendar day as timeWindow
e34938f91 fix(persistence): check and alter columns in one write transaction so concurrent adds no longer hit duplicate column
3cc791795 fix(moderation-reports): keep the report list from shrinking so the dialog scrolls and Load More shows new rows (closes #513)
3e52da500 feat(vr-overlay): alert tracker disconnects on the HMD and show online time on the wrist (closes #512)
0e793a2d4 fix(ci): drop the unused updater zip feature and allow font variance in the avatar alignment test
11c131aee fix(ci): keep Linux checks green for the Windows-only desktop widget
b243aa52c fix(ci): restore green builds after the dependency bump
d99cf86a5 chore: bump
d0676b583 feat(notifications): add translation actions while preserving headings (#508)
4ec25d3aa feat(dashboard): allow the VRChat log viewer as a dashboard panel (closes #511)
71423e63b feat(invite): allow inviting when VRChat runs on another PC (closes #509)
bbd1d6c9e fix(users): show the 18+ badge only for adult age verification
b3afa365f feat(logger): add a Windows window for recorder status and settings
943aec704 ci(logger): publish a Windows x86_64 logger exe with its own third-party notices
```
