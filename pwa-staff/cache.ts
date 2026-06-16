// Define non-standard navigator property for iOS
interface Navigator {
  standalone?: boolean;
}

if ('serviceWorker' in navigator) {
  window.addEventListener('load', function () {
    navigator.serviceWorker
      .register('worker.js')
      .then(function (registration) {
        console.log('ServiceWorker registration successful with scope: ', registration.scope);
      })
      .catch(function (err) {
        console.log('ServiceWorker registration failed: ', err);
      });
  });

  // Tracking logic moved from worker.js (where it didn't belong)
  window.addEventListener('beforeinstallprompt', () => {
    console.log('[PWA] Add to homescreen shown');
  });

  window.addEventListener('appinstalled', () => {
    console.log('[PWA] App installed by user');
  });

  window.addEventListener('load', () => {
    let trackText: string;
    const nav = navigator as Navigator;
    if (nav.standalone) {
      trackText = 'Launched: Installed (iOS)';
    } else if (window.matchMedia('(display-mode: standalone)').matches) {
      trackText = 'Launched: Installed';
    } else {
      trackText = 'Launched: Browser Tab';
    }
    console.log('[PWA]', trackText);
  });
}
