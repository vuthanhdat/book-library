# Google Drive Desktop Filesystem Spike

## Status

- **Windows read-only library observation:** completed
- **Disposable watcher sequence:** completed without using the user's source
  library as a destructive fixture
- **macOS Intel File Provider observation:** validated by the maintainer

ADR-002 remains unchanged: Google Drive Desktop is an external filesystem sync
layer. The application does not use the Google Drive API.

## Official behavior relevant to scanning

Google documents two My Drive modes:

- streaming uses minimal local space and downloads files when opened or marked
  available offline;
- mirroring keeps a full local copy.

Both modes expose content through File Explorer or Finder. On macOS 12.1 and
later, streaming uses Apple's File Provider, has macOS-controlled cache behavior,
and requires the applicable privacy permissions.

Sources:

- [Stream and mirror files](https://support.google.com/drive/answer/13401938)
- [Drive for desktop advanced guide](https://support.google.com/drive/answer/16631477)
- [Drive for desktop troubleshooting](https://support.google.com/drive/answer/2565956)

## Windows observation

The configured path `H:/My Drive/07_NEW_KINDLE` exists and is readable through
ordinary filesystem enumeration. The virtual `H:` drive is not returned as a
normal `MSFT_Volume`, so scanner behavior must not depend on volume metadata,
drive type, NTFS identity, or a successful `Get-Volume` equivalent.

A read-only sample of 50 top-level entries contained 50 directories. No content
was opened, hydrated, modified, renamed, or deleted during this observation.
Absolute child paths and filenames were not recorded.

## Shared scanner contract proposed for M1

The filesystem adapter reports a result for every entry instead of pretending
that enumeration implies availability:

```text
Available(metadata)
TemporarilyUnavailable(reason)
PermissionDenied
EscapedAuthorizedRoot
UnsupportedEntry
Failed(retryable, safe_code)
```

Opening and metadata reads may hydrate streamed files and must therefore be
separate, cancellable operations. A scan should:

- enumerate without modifying source content;
- isolate per-entry failures;
- use relative paths in application/domain results;
- re-check root containment after resolving symlinks or indirections;
- treat placeholders and transient sync failures as unavailable, not missing;
- debounce watcher bursts and reconcile final state rather than interpreting
  every raw event as a business fact;
- never delete catalog history because Drive is offline or unmounted.

## Watcher experiment result

The maintainer confirmed the required disposable synchronized-folder validation
on Windows and macOS Intel. The real book library was not used as a destructive
fixture. Production debounce, targeted reconciliation, and restart recovery
remain M5 work under `REL-001` and `REL-002`.
