import { createApp } from 'vue'
import App from './App.vue'
import './styles/index.css'
import { DEFAULT_COLOR_SCHEME } from './generated/themeDefaults'

function applyDefaultColorScheme() {
  if (DEFAULT_COLOR_SCHEME === 'dark') {
    document.documentElement.setAttribute('data-theme', 'dark')
    return
  }

  if (DEFAULT_COLOR_SCHEME === 'light') {
    document.documentElement.removeAttribute('data-theme')
    return
  }

  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
  if (prefersDark) document.documentElement.setAttribute('data-theme', 'dark')
  else document.documentElement.removeAttribute('data-theme')
}

applyDefaultColorScheme()

createApp(App).mount('#app')
