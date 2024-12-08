import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import PrimeVue from 'primevue/config'
import { definePreset } from '@primevue/themes'
import Aura from '@primevue/themes/aura'
import 'primeicons/primeicons.css'
import 'primeflex/primeflex.css'

const primeVuePreset = definePreset(Aura, {
  semantic: {
    primary: {
      50: "#fff8f8",
      100: "#ffdbdb",
      200: "#ffbfbf",
      300: "#ffa3a3",
      400: "#ff8787",
      500: "#ff6b6b",
      600: "#d95b5b",
      700: "#b34b4b",
      800: "#8c3b3b",
      900: "#662b2b",
      950: '#401b1b'
    }
  }
})

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(PrimeVue, {
  theme: {
    preset: primeVuePreset,
    options: {
      prefix: 'p',
      cssLayer: false
    }
  }
})

app.mount('#app')
