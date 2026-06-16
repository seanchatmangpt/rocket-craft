## Challenge Summary

**Overall risk assessment**: MEDIUM

While the local environment tests pass cleanly, a production deployment faces risks from browser sandbox restrictions, storage limits, and transient connectivity issues.

## Challenges

### [High] Challenge 1: Browser Storage Quota Exceeded

- **Assumption challenged**: Assumed that the browser will always have enough storage space to perform `cache.put(...)` successfully.
- **Attack scenario**: A user runs the PWA on a device with low storage space. When the service worker tries to fetch and cache larger game assets (like `Brm-HTML5-Shipping.wasm` which is listed in `STATIC_ASSETS`), `cache.put` rejects with a `QuotaExceededError`.
- **Blast radius**: Since the promise returned by `cache.put` is not caught, it triggers an unhandled promise rejection. In some modern browsers, unhandled rejections inside service workers can cause the service worker script execution to terminate, breaking offline availability.
- **Mitigation**: Handle rejections of `cache.put` using `.catch(err => ...)` or wrapping them in `try/catch`.

### [Medium] Challenge 2: Network-First Cache Update Failure in Navigate Strategy

- **Assumption challenged**: Assumed that `cache.put` will always succeed when the network is online.
- **Attack scenario**: In the navigate strategy, `fetch(event.request)` succeeds but returns a response with a status code of 404 or 500, or a response from an external authentication origin.
- **Blast radius**: `cache.put` might throw a TypeError because the request method or scheme is unsupported, or because of service worker sandbox rules. This triggers an unhandled promise rejection.
- **Mitigation**: Check response status and type before writing to dynamic cache, and ensure the promise is chained or caught.

---

## Stress Test Results

- Storage quota full → service worker tries to cache static/dynamic assets → `cache.put` rejects → unhandled promise rejection occurs → FAIL
- Navigation returns 500 error → service worker tries to write 500 response to dynamic cache → succeeds or rejects → if rejects, unhandled promise rejection occurs → FAIL

---

## Unchallenged Areas

- Supabase Database connection offline behavior — reason not challenged: database availability is handled by Supabase SDK, which falls outside the scope of our PWA-staff client code verification.
