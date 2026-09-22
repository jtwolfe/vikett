# Ontology

Humans author pages. The referee only chooses among doors that are live.

Dump: [`ontology/pages.json`](../ontology/pages.json) (shared catalogue, including `browser.*`, `term.*`, `files.*`, `notes.*`, `read.*`, and `chat.*`), [`ontology/modules.json`](../ontology/modules.json). Research extras still add host patches on top. `ontology/holdout.json` is unread until the train/holdout loader lands.

Schema: [`schemas/page.schema.json`](../schemas/page.schema.json).

## Modules

| id | Driver | Snap |
| --- | --- | --- |
| wm | hyprctl dispatch | clients / workspaces / activewindow |
| audio | wpctl | default sink volume + mute |
| launch | hyprctl exec / focus | allowlist + running classes |
| mail | notmuch / local IMAP | speaker-bound unread |
| media | playerctl | now playing |
| bucky | buckyboi.sock | listening, chip, gate |
| scene | GlassSpear Surface API | scene library + active |
| notify | mako / dunst history | last notification (private), dismiss all confirms |
| lights | HA light group | room brightness 0–1 |
| timer | local daemon | remaining_sec or null |
| calendar | khal / CalDAV socket | next event, private |
| display | brightnessctl | panel 0–1, authored `hdmi` / `edp` notch. Layout presets reserved |
| climate | HA climate group | °C or null |
| capture | grim / hyprshot | focused output |
| weather | HA weather / LAN cache | short condition or null |
| browser | `hl.dsp.focus` / `hl.dsp.send_shortcut` / `hl.dsp.exec_cmd` | clients, allowlist, `bins`, `lists.bookmark_folder`, `downloads` |
| term | `hl.dsp.focus` / `hl.dsp.send_shortcut`; `exec_cmd` only for unmapped focus | clients, allowlist, `bins` |
| files | `hl.dsp.focus` / `hl.dsp.send_shortcut`; `exec_cmd` for an authored folder | clients, allowlist, `bins`, `lists.dir` |
| notes | `hl.dsp.focus` / `hl.dsp.send_shortcut` | clients |
| read | `hl.dsp.focus` / `hl.dsp.send_shortcut` | clients |
| chat | `hl.dsp.focus` when mapped. Mute and mark-read reserved | clients, `chatUnread` |
| network | ask stays `{ connection }`. VPN is `nmcli connection up` for `lists.vpn` | `network`, `lists.vpn`, `bins` |
| bluetooth | ask stays `{ powered }`. Connect and disconnect one `lists.bt_device` id | `bluetoothOn`, `lists.bt_device`, `bins` |
| power | `powerprofilesctl set` of power-saver, balanced, or performance. Battery is a bucket | `lists.power_profile`, `battery`, `onAc`, `bins` |
| disk | free-space bucket. Timeshift create is owner and confirm | `diskFree`, `bins` |
| updates | pending bucket. Full upgrade is owner and confirm | `updatesPending`, `bins` |
| secrets | unlocked bool. Lock and autotype reserved. Autotype is private and confirm | `secretsUnlocked` |
| sync | status and ping reserved. Send-file stays refused | clients |
| boxes | start/stop an id in `lists.box`. Stop confirms. Runtime not claimed | `lists.box` |
| games | focus a mapped window. Play binds `lists.game` and stays reserved | clients, `lists.game` |
| obs | record confirms and stays reserved. Scene is `desk` or `cam` | `lists.obs_scene` |
| print | print and scan confirm and stay reserved. N copies stay refused | none |
| input | Solaar battery bucket. DPI reserved | `solaarBattery` |

## Page kinds

- **act** — walk a driver. Idempotent when possible (workspace two while already on two is legal).
- **ask** — return `askShape` JSON. Not a paragraph.

## Slots are enums

`audio.bump` direction is `up|down`. Amount is `little|lot`. There is no `percent` slot. “Set volume to 37%” is a refuse, not a fill.

`launch.app` app is an allowlist id. There is no free `bin` string.

`mail.from` who is a contact-book id (`dave`, `school`, `anyone`). The model does not invent an email address.

`scene.apply` scene is a library id. “Make it cozy” is a refuse.

`timer.start` mins is `5|10|15|20`. “Wake me at seven” is a refuse.

`term` app is `foot|kitty|ghostty|alacritty|wezterm`. Font amount is `little|lot` (unnamed is little). There is no shell slot and no free command.

`files.open` dir is `home|downloads|pictures|documents`, and the id must also be in `lists.dir`. The driver maps that id to a path. There is no free path and no `..`.

`notes` app is `obsidian|logseq|joplin`. Vault is `work|personal`. Daily and vault are private. There is no text slot.

`read` app is `zathura|evince|papers|foliate`. Zoom amount is `little|lot`. There is no page number.

`chat` app is `signal|element|vesktop`. Class `discord` is Vesktop. Focus does not exec. There is no send slot.

`calendar.ask_today` is `{ remaining, bucket }`. `how busy am i` stays that page. There is no `calendar.ask_busy`.

## When-clauses (prune)

A page that cannot happen is not offered:

| Page | Dead when |
| --- | --- |
| wm.focus | no client matches target |
| launch.app | app not on allowlist |
| audio.mute | already muted |
| audio.bump up | volume ≥ 0.99 |
| mail.* | guest, or mail offline |
| calendar.* | guest |
| media.* | no player |
| timer.add / ask / cancel | no running timer |
| climate.* | no climate entity in this room |
| weather.ask | weather snap null |
| scene.handoff | always, until the driver is wired |
| browser.focus | no matching client, and not (allowlisted and `bins` contains the binary) |
| browser.ask_open | app not on the allowlist |
| other browser acts | no matching client, or no chord (`mods` empty only for F11) |
| browser.container / container_tab | class is not firefox, or no chord (none builtin) |
| browser.profile / profile_window / tab_group_collapse | class is not chrome or chromium, or no chord |
| Zen next, prev, sidebar, split, unsplit | class is not zen |
| Zen index, compact, glance, new workspace, web panel, essential, move-tab | no builtin chord. Compact and glance are not guessed |
| browser.downloads | `downloads` is null, or the caller is a guest |
| browser.dev_tools | guest, or `who` is not the owner |
| term.focus | no matching client, and not (allowlisted and `bins` contains the binary) |
| term.new_window and other term acts | no matching client, or no chord for that app. New window never execs |
| term.next / term.prev | aliases are `next terminal` / `previous terminal`, not `next tab` |
| files.focus | that class is not mapped. Focus does not exec |
| files.open | dir missing, not authored, or not in `lists.dir`; or the app is neither mapped nor (allowlisted and in `bins`) |
| files.back / up / hidden | no matching client, or no chord for that app. Yazi has none. Dolphin hidden has none |
| files.sort / files.trash | no chord. Trash is still `confirm: true` |
| notes.focus | that class is not mapped |
| notes.daily / notes.vault | guest, or no chord. Daily is Logseq Alt+J only |
| notes.sidebar | no chord |
| read.focus | that class is not mapped |
| read.next_page / prev_page | not Evince or Papers, or no client |
| read.zoom | not Evince, Papers, or Foliate |
| read.dark | not Zathura |
| read.chapter_next / chapter_prev | no chord |
| chat.focus | that class is not mapped. Focus does not exec |
| chat.mute_app / chat.mark_read | no modifier chord |
| chat.ask_unread | guest, or `chatUnread` has no key for that app |
| mail.next_unread / archive / mark_read | guest, empty who, or mail offline. Archive confirms. No send, reply, or forward |
| session.suspend / reboot / poweroff | guest, empty who, unknown who, or who is not the owner. `confirm` is separate from policy |
| session.idle | always. `force_idle` is not an inhibitor |
| session.lock / session.dpms | unchanged. Lock needs hyprlock. DPMS stays the v0 string |
| display.output | output class is not in `lists.output` and is not the classified focused output |
| display.layout | reserved. A layout preset needs a mode; no brightnessctl or hl.dsp walk |
| notify.dismiss_all | no notification history. `notify.read_last` stays private |
| wm.split_ratio / group_next / layout | always legal. `special workspace` stays `wm.workspace` |
| network.ask | `network` is null. The ask does not connect or disconnect |
| network.vpn | name missing from `lists.vpn`, or `nmcli` is not in `bins`. Confirm. Not an SSID |
| bluetooth.ask | adapter is off |
| bluetooth.connect / disconnect | adapter off, device not in `lists.bt_device`, or `bluetoothctl` not in `bins`. Pair stays refused |
| power.profile | profile not in `lists.power_profile`, or `powerprofilesctl` not in `bins` |
| power.ask_battery | `battery` is null or outside 0..=1. On AC the bucket is `ac` |
| disk.ask | `diskFree` is null or outside 0..=1 |
| disk.timeshift | guest, empty or unknown who, who is not the owner, or `timeshift` is not in `bins`. Confirm. Not format or delete |
| updates.ask | `updatesPending` is null |
| updates.upgrade | guest, empty or unknown who, who is not the owner, or no full-upgrade binary is in `bins`. Confirm. Not a package name |
| secrets.ask_unlocked | `secretsUnlocked` is null, or the caller is a guest. The body is `{ unlocked }` only |
| secrets.lock | reserved. No vault lock chord. Not `session.lock` |
| secrets.autotype | always reserved. Private and confirm. It would send secret bytes |
| sync.ask / sync.ping | reserved. No sync command is claimed |
| boxes.start / boxes.stop | name missing or not in `lists.box`, or the runtime is unclaimed. Stop is confirm. An image is refused |
| games.focus | no matching client. Unmapped does not exec |
| games.play | title missing or not in `lists.game`. Reserved. No store URL. Buy stays refused |
| obs.record | no obs client. Confirm. Mapped walk stays `reserved`. Naming obs does not walk the screen recorder. Stream stays refused |
| obs.scene | scene not in `lists.obs_scene`, or no chord |
| print.print / print.scan | reserved. Confirm. N copies stay refused |
| input.ask_battery | `solaarBattery` is null or outside 0..=1. Not `power.ask_battery` |
| input.dpi | reserved. No solaar chord |

Missing focus on “switch to jellyfin” **promotes** to `launch.app` if jellyfin is allowlisted — that is an engine rule, not a new page.

## Everyday coverage (v0)

Desk: focus / launch / volume / headphones / screenshot / night light / calendar ask / mail ask / hide buddy.

Kitchen: scene kitchen-cook, timer, climate notch, lights, weather.

Living + guest: media, volume, guest mode / lock private. Mail and calendar gone.

House-scale later (not v0 pages): vacuum, locks, cameras. Print and scan are confirm and reserved. VPN connect is a named confirm. Bluetooth pair, wifi off, DNS, format, a free package upgrade, revealing a password, buying a game, streaming, and running an image stay refused. Do not add a page whose walk is “the LLM will figure it out.”

## Adding a page

1. Write the page in the catalogue with aliases, slots, when, policy, examples, refuses.
2. Add a snap field if prune needs new state.
3. Add at least one golden that takes it and one that refuses a nearby cheat.
4. Guest-test if policy is private.
5. Do not teach the referee a walk string.

Family pages (`browser` and later modules) live in `src/drivers/`. `slots::fill` may default `app` from the focused class. A dead family alias that is a strictly longer phrase than every live alias silences instead of walking a neighbor. No `browser.*` alias is the bare word `browser`.
