## Review Summary

**Verdict**: REQUEST_CHANGES

The build, tests, lint checks, and XSS issues are completely resolved. However, two unhandled promise rejections exist in `worker.ts`, and styling inconsistencies exist across the HTML templates and in the stylesheet.

## Findings

### [Major] Finding 1: Unhandled Promise Rejections in Service Worker

- **What**: The promise returned by `cache.put(...)` is not returned or caught locally inside the service worker fetch handlers.
- **Where**: `pwa-staff/worker.ts` lines 97-100 and lines 127-131.
- **Why**: If `cache.put` rejects (due to storage quota exhaustion, offline error, or invalid request method), the promise chain is broken and causes an unhandled promise rejection.
- **Suggestion**:
  - In navigate strategy (lines 97-100), return the `cache.put` promise or catch it locally:
    ```typescript
    return caches.open(DYNAMIC_CACHE_NAME).then((cache) => {
      return cache.put(event.request, networkResponse.clone())
        .catch(err => console.error('[SW] Dynamic cache put failed:', err))
        .then(() => networkResponse);
    });
    ```
  - In cache-first strategy (lines 127-131), return the `cache.put` promise inside `.then`:
    ```typescript
    caches.open(DYNAMIC_CACHE_NAME).then((cache) => {
      return cache.put(event.request, responseToCache);
    }).catch(err => {
      console.warn('[Service Worker] Failed to update dynamic cache:', err);
    });
    ```

### [Minor] Finding 2: Styling Inconsistencies

- **What**: Inconsistent path types in HTML imports and unstyled form controls.
- **Where**: `pwa-staff/index.html` (line 7), `pwa-staff/css/style.css`, and various form HTML templates.
- **Why**: `index.html` links to `/dist/style.css`, while others link to `dist/style.css`. Additionally, inputs, form buttons, tables, and modals are unstyled and default to standard browser UI.
- **Suggestion**: Standardize all stylesheet links (e.g. use absolute `/dist/style.css` everywhere) and add form/table/modal selectors to `css/style.css`.

---

## Verified Claims

- Build compiles cleanly → verified via running `npm run build` → PASS
- Unit tests pass → verified via running `npm run test` → PASS
- ESLint passes cleanly → verified via running `npm run lint` → PASS
- XSS vulnerabilities resolved → verified via code review of `textContent` bindings in `admin.ts` and `leaderboard.ts` → PASS
- ESLint `sourceType` configured as module → verified via inspecting `.eslintrc.json` → PASS

---

## Coverage Gaps

- Playwright E2E tests - risk level: low - recommendation: accept risk as vitest unit tests verify the components in isolation.

---

## Unverified Items

- None.
