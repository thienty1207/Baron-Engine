---
name: mobile-application-engineering
description: Use for normal Android or iOS application development, lifecycle, navigation, offline behavior, permissions, storage, networking, and device validation.
routing_triggers: mobile app,android app,ios app,lifecycle,navigation,offline sync,local storage,permission flow,foreground,background,deep link,emulator,device connectivity,battery,app state
routing_exclusions: apk reverse,apk manifest,binary reverse,decompile,disassembly,malware triage,static artifact analysis
profile_affinities: mobile,fullstack
routing_dependencies: superpowers
routing_conflicts: apk-mobile-analysis,binary-reverse-analysis,malware-triage
evidence_requirements: affected app flow,platform state,device or emulator conditions
verification_hints: lifecycle state,offline or network failure,permission state,platform build or release check
---

# Mobile Application Engineering

## Baron Contract

Use when the task changes a normal Android or iOS application flow. Use this
skill for ordinary Android and iOS application development. It is a
thin domain layer over Baron Core and Superpowers. It does not perform APK
reverse engineering or malware analysis; those tasks use their specialized
defensive skills only when artifact-analysis evidence is present.

## Lifecycle and State

- Model foreground, background, suspended, resumed, terminated, and restored
  states explicitly for every flow that can lose work.
- Keep UI state, durable local state, and server state separate. Define which
  state wins after process death or a reconnect.
- Make navigation transitions safe across cold start, warm start, rotation,
  multi-window, and interrupted background work where the platform supports it.
- Preserve user intent through loading, error, retry, and cancellation states.

## Navigation and Interaction

- Treat navigation as a typed, validated boundary. Define back behavior,
  authentication transitions, deep-link destinations, and unavailable routes.
- Keep screen state recoverable without duplicating server truth or exposing
  sensitive parameters in a URL, intent, or notification.
- Test realistic device sizes, input methods, accessibility settings, and
  platform-specific navigation behavior for changed flows.

## Offline, Network, and Cache

- Identify connectivity transitions, request timeouts, retries, cancellation,
  and partial responses. Do not assume a network is available.
- Define cache freshness, invalidation, conflict handling, and replay behavior
  before adding offline writes or synchronization.
- Make queued work idempotent and bounded. Preserve a recoverable local record
  when a background upload is interrupted.
- Keep API contracts and mobile clients version-compatible during rollout.

## Storage and Permissions

- Store secrets and personal data in platform-secure storage with a clear
  retention and deletion path. Never log credentials or tokens.
- Request the least privilege needed, explain denial and revocation states, and
  keep the feature usable when an optional permission is unavailable.
- Validate external intents, deep links, files, and notification payloads at
  the trust boundary before using them as application state.

## Background and Device Conditions

- Define behavior for foreground/background transitions, OS task limits,
  battery saver, low memory, clock changes, and connectivity changes.
- Keep native platform boundaries small and documented. Isolate Android/iOS
  differences behind a tested interface where shared behavior is intended.
- Measure startup, frame stability, memory, battery, and network cost when a
  change claims a performance improvement; otherwise label the risk unknown.

## Verification and Release

- Verify changed flows in unit/domain tests plus emulator or realistic-device
  conditions appropriate to the risk.
- Exercise lifecycle loss, offline/network failure, permission denial, deep
  links, and platform-specific behavior when those paths are touched.
- Run the supported OS/build matrix and record signing, store, migration, and
  release checks without claiming a store submission occurred.
- Preserve proof and trace evidence for high-risk data, auth, permission, or
  release work. Keep recovery state actionable if a device test is interrupted.

## Output Contract

Report the affected user journey, lifecycle/platform assumptions, state and
failure handling, security/privacy risks, device evidence, and exact tests or
build checks. Mark unavailable emulator/device facts unknown. Superpowers
owns planning, TDD, review, proof, and trace; this skill only supplies the
mobile engineering decisions needed by the selected route.
