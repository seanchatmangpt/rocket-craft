// Mirror Nuxt auto-imports for unit tests (no Nuxt runtime needed)
import {
  ref, shallowRef, computed, readonly, reactive,
  watch, watchEffect, onMounted, onUnmounted,
} from 'vue'

Object.assign(globalThis, {
  ref, shallowRef, computed, readonly, reactive,
  watch, watchEffect, onMounted, onUnmounted,
})
