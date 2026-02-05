import { createPinia } from 'pinia'
import { createApp } from 'vue'
import App from './App.vue'
import './assets/main.css'

// Set dark mode by default
document.documentElement.classList.add('dark')

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.mount('#app')
