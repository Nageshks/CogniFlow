import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { getCurrentWindow } from '@tauri-apps/api/window'
import router from './router'
import App from './App.vue'
import './style.css'

const app = createApp(App)
app.use(createPinia())
app.use(router)

// Show window once frontend scripts start executing, preventing visual flash
try {
  getCurrentWindow().show()
} catch (e) {
  console.warn('Failed to show window (probably running in a browser):', e)
}

app.mount('#app')
