import { supabase } from './lib/supabaseClient'

const userEmailElement = document.getElementById('user-email')
const logoutButton = document.getElementById('logout-button')

async function initProfile() {
  const { data: { user }, error } = await supabase.auth.getUser()

  if (error || !user) {
    window.location.href = 'login.html'
    return
  }

  if (userEmailElement) {
    userEmailElement.textContent = user.email || ''
  }
}

initProfile().catch((err) => {
  console.error('Failed to initialize profile:', err)
  window.location.href = 'login.html'
})

if (logoutButton) {
  logoutButton.addEventListener('click', async () => {
    const { error } = await supabase.auth.signOut()

    if (error) {
      alert(error.message)
    } else {
      window.location.href = 'login.html'
    }
  })
}
