/* eslint-disable no-unused-vars */
// In a real application, you would fetch the user's role from your authentication service.
// For this example, we'll simulate it.
function getUserRole() {
  // Simulate a user with an 'admin' role.
  // In a real app, you might get this from a JWT token, a server API call, etc.
  return 'admin';
}

function isUserAdmin() {
  return getUserRole() === 'admin';
}
