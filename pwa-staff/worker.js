"use strict";
const sw = self;
const CACHE_VERSION = "v2";
const STATIC_CACHE_NAME = `static-assets-${CACHE_VERSION}`;
const DYNAMIC_CACHE_NAME = `dynamic-content-${CACHE_VERSION}`;
const OFFLINE_URL = "offline.html";
const STATIC_ASSETS = [
  OFFLINE_URL,
  "cache.js",
  "lucide.min.js",
  "index.html",
  "admin.html",
  "leaderboard.html",
  "login.html",
  "signup.html",
  "profile.html",
  "favicon.ico",
  "dist/style.css",
  "dist/admin.js",
  "dist/leaderboard.js",
  "dist/login.js",
  "dist/signup.js",
  "dist/profile.js",
  "dist/auth.js",
  "/manufactured/Brm-HTML5-Shipping.html",
  "/manufactured/Brm-HTML5-Shipping.js",
  "/manufactured/Brm-HTML5-Shipping.wasm",
  "/manufactured/Brm-HTML5-Shipping.data",
  "/manufactured/receipt.json",
  "SurvivalGame-HTML5-Shipping.html",
  "SurvivalGame-HTML5-Shipping.js",
  "SurvivalGame-HTML5-Shipping.wasm",
  "SurvivalGame-HTML5-Shipping.data",
  "RealisticRendering-HTML5-Shipping.html",
  "RealisticRendering-HTML5-Shipping.js",
  "RealisticRendering-HTML5-Shipping.wasm",
  "RealisticRendering-HTML5-Shipping.data",
  "FullSpectrum-HTML5-Shipping.html",
  "FullSpectrum-HTML5-Shipping.js",
  "FullSpectrum-HTML5-Shipping.wasm",
  "FullSpectrum-HTML5-Shipping.data",
  "ShooterGame-HTML5-Shipping.html",
  "ShooterGame-HTML5-Shipping.js",
  "ShooterGame-HTML5-Shipping.wasm",
  "ShooterGame-HTML5-Shipping.data"
];
sw.addEventListener("install", (event) => {
  console.log("[Service Worker] Installing version:", CACHE_VERSION);
  event.waitUntil(
    caches.open(STATIC_CACHE_NAME).then((cache) => {
      console.log("[Service Worker] Pre-caching static assets");
      return Promise.allSettled(
        STATIC_ASSETS.map((url) => {
          return cache.add(url).catch((err) => {
            console.warn(`[Service Worker] Failed to cache asset: ${url}`, err);
            if (url === OFFLINE_URL) {
              throw new Error("Critical asset (offline.html) failed to cache");
            }
            throw err;
          });
        })
      ).then((results) => {
        const offlineIdx = STATIC_ASSETS.indexOf(OFFLINE_URL);
        const offlineResult = results[offlineIdx];
        if (offlineResult && offlineResult.status === "rejected") {
          throw new Error("Critical asset (offline.html) failed to cache");
        }
      });
    }).then(() => sw.skipWaiting())
  );
});
sw.addEventListener("activate", (event) => {
  console.log("[Service Worker] Activating and cleaning old caches...");
  const cacheAllowlist = [STATIC_CACHE_NAME, DYNAMIC_CACHE_NAME];
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames.map((cacheName) => {
          if (!cacheAllowlist.includes(cacheName)) {
            console.log("[Service Worker] Deleting obsolete cache:", cacheName);
            return caches.delete(cacheName);
          }
        })
      );
    }).then(() => sw.clients.claim())
  );
});
sw.addEventListener("fetch", (event) => {
  if (event.request.method !== "GET") {
    return;
  }
  const requestUrl = new URL(event.request.url);
  const rootBrmFiles = [
    "/Brm-HTML5-Shipping.wasm",
    "/Brm-HTML5-Shipping.data",
    "/Brm-HTML5-Shipping.js",
    "/Brm-HTML5-Shipping.html"
  ];
  const isRootBrmFile = rootBrmFiles.includes(requestUrl.pathname);
  const processedUrl = isRootBrmFile ? `${requestUrl.origin}/manufactured${requestUrl.pathname}` : event.request.url;
  if (requestUrl.pathname.startsWith("/manufactured/") || isRootBrmFile) {
    const fetchRequest = isRootBrmFile ? new Request(processedUrl, {
      method: event.request.method,
      headers: event.request.headers,
      mode: "cors",
      credentials: event.request.credentials
    }) : event.request;
    event.respondWith(
      fetch(fetchRequest).then((networkResponse) => {
        if (networkResponse && networkResponse.status === 200) {
          const responseToCache = networkResponse.clone();
          caches.open(DYNAMIC_CACHE_NAME).then((cache) => {
            cache.put(processedUrl, responseToCache);
          }).catch((err) => {
            console.warn("[Service Worker] Failed to cache manufactured asset:", err);
          });
        }
        return networkResponse;
      }).catch((error) => {
        console.warn("[Service Worker] Fetch failed for manufactured asset, falling back to cache:", error);
        return caches.match(processedUrl).then((cachedResponse) => {
          return cachedResponse || Response.error();
        });
      })
    );
    return;
  }
  if (event.request.mode === "navigate") {
    event.respondWith(
      fetch(event.request).then((networkResponse) => {
        return caches.open(DYNAMIC_CACHE_NAME).then((cache) => {
          return cache.put(event.request, networkResponse.clone()).then(() => networkResponse).catch((err) => {
            console.warn("[Service Worker] Failed to update dynamic cache:", err);
            return networkResponse;
          });
        });
      }).catch((error) => {
        console.error("[Service Worker] Navigation fetch failed; checking cache or returning offline page:", error);
        return caches.match(event.request).then((cachedResponse) => {
          if (cachedResponse) {
            return cachedResponse;
          }
          return caches.match(OFFLINE_URL).then((offlineResponse) => {
            return offlineResponse || Response.error();
          });
        });
      })
    );
    return;
  }
  event.respondWith(
    caches.match(event.request).then((cachedResponse) => {
      if (cachedResponse) {
        return cachedResponse;
      }
      return fetch(event.request).then((networkResponse) => {
        if (!networkResponse || networkResponse.status !== 200 || networkResponse.type !== "basic") {
          return networkResponse;
        }
        const requestUrl2 = new URL(event.request.url);
        if (requestUrl2.protocol === "http:" || requestUrl2.protocol === "https:") {
          const responseToCache = networkResponse.clone();
          caches.open(DYNAMIC_CACHE_NAME).then((cache) => {
            return cache.put(event.request, responseToCache);
          }).catch((err) => {
            console.warn("[Service Worker] Failed to update dynamic cache:", err);
          });
        }
        return networkResponse;
      }).catch((err) => {
        console.warn(`[Service Worker] Fetch failed for ${event.request.url}:`, err);
        return Response.error();
      });
    })
  );
});
