# WinTrack - Project State

## Overview

WinTrack is a Windows application usage tracker built with:

* Tauri v2
* React
* TypeScript
* Rust
* SQLite

Goal:

Provide a Windows equivalent of Android Digital Wellbeing.

Core functionality:

* Track active application usage
* Store usage history locally
* Display analytics
* Allow app limits and reminders
* Run completely offline
* Run in the background
* Start automatically with Windows

---

## Current Database Location

Fixed location:

C:\ProgramData\WinTrack\Database

Database relocation feature has been removed from product scope.

---

## Product Decisions

### Removed Features

The following are intentionally removed and should NOT be reintroduced unless explicitly requested:

* Productivity score
* Productivity analytics
* Productivity percentages
* Theme system
* Light mode
* Custom user-created categories
* Database relocation feature

### Retained Features

* Dark mode only
* App categories
* Show Ignored toggle
* App limits
* Reminders
* Soft Lock
* User app renaming

---

## App Breakdown Design

Current columns:

App | Today | Total | Category | Limit | Reminder | Soft Lock | Status | Ignore

Keep this structure.

Category changes save immediately from the dropdown.

Show Ignored toggle remains.

---

## Display Name Strategy

Preferred display name resolution order:

1. User custom rename
2. Start Menu shortcut name
3. File description metadata
4. Executable name fallback

Examples:

Visual Studio Code
instead of
Code.exe

Google Chrome
instead of
chrome.exe

Start Menu shortcut matching has been implemented and is mostly working.

---

## Icon System Status

Current status:

Partially implemented.

Database contains:

* icon_path
* icon_data

Frontend currently renders icon_data.

Base64 icon extraction experiments were performed.

The icon pipeline is not considered finished.

Future work:

* Decide whether to store icons only as base64 in database
* Or store icons as cached PNG files

Avoid maintaining two systems long-term.

---

## Tracking Status

### Working

Traditional Win32 desktop applications:

* Chrome
* VS Code
* Explorer
* Brave
* Spotify
* Telegram
* WhatsApp (partially)

### Known Problem

Some Windows/UWP applications are not tracked correctly.

Examples:

* Calculator
* Settings
* Microsoft Store

Likely related to:

* ApplicationFrameHost.exe
* RuntimeBroker.exe
* UWP window ownership

This is currently the highest-priority engineering task.

---

## Limits System

Current fields:

* daily_limit_minutes
* reminder_interval_minutes
* soft_lock_enabled

Need continued verification that:

* Limits persist correctly
* Reminders fire correctly
* Soft lock survives restart
* Ignored apps bypass restrictions

---

## Current Architecture

Frontend:

* React
* TypeScript
* Zustand

Backend:

* Rust
* SQLite
* Tauri Commands

Storage:

* SQLite database
* Local-only
* Offline-first

---

## Cleanup Tasks Remaining

### Backend

* Remove remaining productivity-related code
* Remove dead FocusPulse leftovers
* Remove dead database relocation code
* Remove unused migrations if any remain
* Remove unused commands and helpers

### Frontend

* Remove dead utility functions
* Remove unused state
* Remove unused components
* Remove debugging code

### General

* Continue WinTrack rename audit
* Finish icon subsystem
* Improve UWP app tracking

---

## Current Priority Order

1. Fix UWP tracking (Calculator, Settings, Microsoft Store)
2. Complete display-name system
3. Clean icon subsystem
4. Remove dead code
5. Audit limits/reminders/soft-lock behavior
6. Performance and optimization review

---

## Development Preference

User prefers:

* Small incremental changes
* One task at a time
* No massive refactors in a single step
* Verify each change with cargo check / cargo test before proceeding

---

## Current Version

WinTrack v2.4.0

Last stable checkpoint:

Checkpoint commit created before UWP tracking investigation.
