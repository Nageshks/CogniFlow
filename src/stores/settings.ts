import { ref } from 'vue'
import { defineStore } from 'pinia'
import type { HybridSearchConfig } from '@/types'

export const useSettingsStore = defineStore('settings', () => {
  const searchConfig = ref<HybridSearchConfig>({
    semantic_weight: 0.7,
    keyword_weight: 0.3,
  })

  const autoSaveInterval = ref(Number(localStorage.getItem('autoSaveInterval')) || 30000)
  const theme = ref<'dark' | 'light'>((localStorage.getItem('theme') as 'dark' | 'light') || 'dark')
  const accent = ref<'blue' | 'purple' | 'cyan' | 'green' | 'orange'>(
    (localStorage.getItem('accent') as 'blue' | 'purple' | 'cyan' | 'green' | 'orange') || 'cyan'
  )

  function updateSearchConfig(config: Partial<HybridSearchConfig>) {
    searchConfig.value = { ...searchConfig.value, ...config }
  }

  function updateAutoSaveInterval(interval: number) {
    autoSaveInterval.value = interval
    localStorage.setItem('autoSaveInterval', String(interval))
  }

  function updateTheme(val: 'dark' | 'light') {
    theme.value = val
    localStorage.setItem('theme', val)
    applyThemeAndAccent()
  }

  function updateAccent(val: 'blue' | 'purple' | 'cyan' | 'green' | 'orange') {
    accent.value = val
    localStorage.setItem('accent', val)
    applyThemeAndAccent()
  }

  function applyThemeAndAccent() {
    if (typeof window === 'undefined') return
    const root = document.documentElement

    // Toggle theme classes
    if (theme.value === 'dark') {
      root.classList.add('dark')
      root.classList.remove('light')
    } else {
      root.classList.add('light')
      root.classList.remove('dark')
    }

    // Curated accent color mapping scales (Light mode gets high-contrast readable colors, Dark gets vibrant neon scales)
    const isDark = theme.value === 'dark'
    const accentMappings = {
      cyan: {
        primary: isDark ? 'hsl(180 100% 50%)' : 'hsl(180 100% 40%)',
        hover: isDark ? 'hsl(180 100% 65%)' : 'hsl(180 100% 30%)',
        violet: isDark ? 'hsl(270 100% 60%)' : 'hsl(270 100% 50%)',
        violet_hover: isDark ? 'hsl(270 100% 75%)' : 'hsl(270 100% 40%)',
      },
      purple: {
        primary: isDark ? 'hsl(270 100% 60%)' : 'hsl(262.1 83.3% 57.8%)',
        hover: isDark ? 'hsl(270 100% 75%)' : 'hsl(262.1 83.3% 47.8%)',
        violet: isDark ? 'hsl(180 100% 50%)' : 'hsl(180 100% 40%)',
        violet_hover: isDark ? 'hsl(180 100% 65%)' : 'hsl(180 100% 30%)',
      },
      blue: {
        primary: isDark ? 'hsl(217 91% 60%)' : 'hsl(221.2 83.2% 53.3%)',
        hover: isDark ? 'hsl(217 91% 70%)' : 'hsl(221.2 83.2% 43.3%)',
        violet: isDark ? 'hsl(270 100% 60%)' : 'hsl(270 100% 50%)',
        violet_hover: isDark ? 'hsl(270 100% 75%)' : 'hsl(270 100% 40%)',
      },
      green: {
        primary: isDark ? 'hsl(160 84% 39%)' : 'hsl(142.1 76.2% 36.3%)',
        hover: isDark ? 'hsl(160 84% 50%)' : 'hsl(142.1 76.2% 26.3%)',
        violet: isDark ? 'hsl(180 100% 50%)' : 'hsl(180 100% 40%)',
        violet_hover: isDark ? 'hsl(180 100% 65%)' : 'hsl(180 100% 30%)',
      },
      orange: {
        primary: isDark ? 'hsl(38 92% 50%)' : 'hsl(37.7 92.1% 50.2%)',
        hover: isDark ? 'hsl(38 92% 60%)' : 'hsl(37.7 92.1% 40.2%)',
        violet: isDark ? 'hsl(270 100% 60%)' : 'hsl(270 100% 50%)',
        violet_hover: isDark ? 'hsl(270 100% 75%)' : 'hsl(270 100% 40%)',
      },
    }

    const selectedAccent = accentMappings[accent.value]
    root.style.setProperty('--color-accent-blue', selectedAccent.primary)
    root.style.setProperty('--color-accent-blue-hover', selectedAccent.hover)
    root.style.setProperty('--color-accent-violet', selectedAccent.violet)
    root.style.setProperty('--color-accent-violet-hover', selectedAccent.violet_hover)
  }

  // Trigger initial paint
  if (typeof window !== 'undefined') {
    applyThemeAndAccent()
  }

  return {
    searchConfig,
    autoSaveInterval,
    theme,
    accent,
    updateSearchConfig,
    updateAutoSaveInterval,
    updateTheme,
    updateAccent,
    applyThemeAndAccent,
  }
})
