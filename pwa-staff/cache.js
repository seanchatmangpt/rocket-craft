"use strict";
if ("serviceWorker" in navigator) {
  window.addEventListener("load", function() {
    navigator.serviceWorker.register("worker.js").then(function(registration) {
      console.log("ServiceWorker registration successful with scope: ", registration.scope);
    }).catch(function(err) {
      console.log("ServiceWorker registration failed: ", err);
    });
  });
  window.addEventListener("beforeinstallprompt", () => {
    console.log("[PWA] Add to homescreen shown");
  });
  window.addEventListener("appinstalled", () => {
    console.log("[PWA] App installed by user");
  });
  window.addEventListener("load", () => {
    let trackText;
    const nav = navigator;
    if (nav.standalone) {
      trackText = "Launched: Installed (iOS)";
    } else if (window.matchMedia("(display-mode: standalone)").matches) {
      trackText = "Launched: Installed";
    } else {
      trackText = "Launched: Browser Tab";
    }
    console.log("[PWA]", trackText);
  });
}
