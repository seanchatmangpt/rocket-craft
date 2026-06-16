// This script should be included in the admin.html page.
// It depends on functions defined in auth.js.
/* global isUserAdmin */

// Check if the user is an admin.
if (!isUserAdmin()) {
  // If not, redirect to the home page.
  window.location.href = '/';
}
