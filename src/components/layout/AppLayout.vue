<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { getCurrentWindow } from '@tauri-apps/api/window'
import AppSidebar from './AppSidebar.vue'

const route = useRoute()
const router = useRouter()

const pageTitle = computed(() => {
  const titles: Record<string, string> = {
    library: 'Library',
    workflows: 'Workflows',
    'workflow-editor': 'Workflow Editor',
    graph: 'Knowledge Graph',
    settings: 'Settings',
  }
  return titles[route.name as string] ?? 'CogniFlow'
})



function minimizeWindow() {
  try {
    const win = getCurrentWindow()
    if (win) {
      win.minimize()
    }
  } catch (e) {
    console.error('Failed to minimize window:', e)
  }
}

async function toggleMaximizeWindow() {
  try {
    const win = getCurrentWindow()
    if (win) {
      const maximized = await win.isMaximized()
      if (maximized) {
        await win.unmaximize()
      } else {
        await win.maximize()
      }
    }
  } catch (e) {
    console.error('Failed to toggle maximize window:', e)
  }
}

function closeWindow() {
  try {
    const win = getCurrentWindow()
    if (win) {
      win.close()
    }
  } catch (e) {
    console.error('Failed to close window:', e)
  }
}
</script>

<template>
  <div class="app-layout relative overflow-hidden bg-bg-primary">
    <AppSidebar />
    <div class="app-main animate-fade-in flex flex-col h-screen relative">
      <!-- Fixed Header/Toolbar at the top spanning full width of main content area -->
      <header class="app-header select-none relative flex items-center justify-between border-b border-border-default bg-bg-glass backdrop-blur-md px-6 h-14" data-tauri-drag-region>
        <!-- Left: Page Title -->
        <h1 class="text-sm font-extrabold select-none cursor-default" style="font-family: var(--font-sans); color: var(--color-text-primary);">
          {{ pageTitle }}
        </h1>

        <!-- Center: Empty for flex-1 spacing -->
        <div class="flex-1"></div>

        <!-- Right: Window Actions -->
        <div class="flex items-center gap-1.5 border-l pl-3 border-border-default ml-auto">
          <button
            id="window-minimize"
            class="w-6 h-6 flex items-center justify-center rounded-lg hover:bg-bg-elevated transition-colors cursor-pointer text-[#6B7280] hover:text-[#111827]"
            title="Minimize"
            @click="minimizeWindow"
          >
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><line x1="5" y1="12" x2="19" y2="12"></line></svg>
          </button>
          <button
            id="window-maximize"
            class="w-6 h-6 flex items-center justify-center rounded-lg hover:bg-bg-elevated transition-colors cursor-pointer text-[#6B7280] hover:text-[#111827]"
            title="Maximize"
            @click="toggleMaximizeWindow"
          >
            <svg width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect></svg>
          </button>
          <button
            id="window-close"
            class="w-6 h-6 flex items-center justify-center rounded-lg hover:bg-[#EF4444]/10 hover:text-[#EF4444] transition-colors cursor-pointer text-[#6B7280]"
            title="Close"
            @click="closeWindow"
          >
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
          </button>
        </div>
      </header>

      <main class="app-content flex-1 overflow-y-auto p-8">
        <router-view />
      </main>
    </div>
  </div>
</template>
