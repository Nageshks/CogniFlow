<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Search, Plus, ChevronLeft } from 'lucide-vue-next'
import { getCurrentWindow } from '@tauri-apps/api/window'

const route = useRoute()
const router = useRouter()

// Get window handler safely for non-tauri environments (like web test)
const appWindow = typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__ ? getCurrentWindow() : null

const pageTitle = computed(() => {
  const titles: Record<string, string> = {
    home: 'Home',
    library: 'Library',
    workflows: 'Workflows',
    'workflow-editor': 'Workflow Editor',
    graph: 'Knowledge Graph',
    settings: 'Settings',
  }
  return titles[route.name as string] ?? 'CogniFlow'
})

function handleSearchClick() {
  router.push('/library')
}

function newPrompt() {
  window.dispatchEvent(new CustomEvent('open-new-prompt'))
  router.push('/library')
}

function newWorkflow() {
  window.dispatchEvent(new CustomEvent('open-new-workflow'))
  router.push('/workflows')
}

function minimizeWindow() {
  if (appWindow) {
    appWindow.minimize()
  }
}

async function toggleMaximizeWindow() {
  if (appWindow) {
    const maximized = await appWindow.isMaximized()
    if (maximized) {
      await appWindow.unmaximize()
    } else {
      await appWindow.maximize()
    }
  }
}

function closeWindow() {
  if (appWindow) {
    appWindow.close()
  }
}
</script>

<template>
  <header class="app-header select-none relative" data-tauri-drag-region>
    <!-- Left: Page Title -->
    <h1 class="text-base font-bold min-w-[140px] pointer-events-none" style="font-family: var(--font-display); background: linear-gradient(135deg, var(--color-accent-blue), var(--color-accent-violet)); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;">
      CogniFlow
    </h1>

    <!-- Center: Global Search Bar -->
    <div id="header-search" class="search-bar cursor-pointer" @click="handleSearchClick" role="button" tabindex="0">
      <Search :size="14" style="color: var(--color-text-muted); flex-shrink: 0;" />
      <input
        type="text"
        placeholder="Search prompts, workflows..."
        readonly
        class="cursor-pointer pointer-events-none"
      />
      <div class="shortcut pointer-events-none">
        <kbd class="text-[9px]">Ctrl</kbd>
        <span class="text-[9px]">+</span>
        <kbd class="text-[9px]">K</kbd>
      </div>
    </div>

    <!-- Right: Contextual Primary Action Button + Window Actions -->
    <div class="flex items-center gap-4 ml-auto">
      <!-- Contextual Page Actions -->
      <div class="flex items-center gap-2">
        <!-- Library actions -->
        <button
          v-if="route.name === 'library'"
          id="header-new-prompt"
          class="flex items-center gap-1.5 px-4 py-2 rounded-lg text-xs font-semibold transition-all cursor-pointer shadow-sm hover:scale-[1.02] border border-transparent"
          style="background: var(--color-accent-blue); color: white;"
          @click="newPrompt"
        >
          <Plus :size="14" />
          <span>New Prompt</span>
        </button>

        <!-- Workflows actions -->
        <button
          v-if="route.name === 'workflows'"
          id="header-new-workflow"
          class="flex items-center gap-1.5 px-4 py-2 rounded-lg text-xs font-semibold transition-all cursor-pointer shadow-sm hover:scale-[1.02] border border-transparent"
          style="background: var(--color-accent-violet); color: white;"
          @click="newWorkflow"
        >
          <Plus :size="14" />
          <span>New Workflow</span>
        </button>

        <!-- Workflow Editor actions -->
        <button
          v-if="route.name === 'workflow-editor'"
          id="header-back-workflows"
          class="flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs font-semibold transition-all cursor-pointer border border-border-default hover:bg-bg-elevated"
          style="background: var(--color-bg-tertiary); color: var(--color-text-primary);"
          @click="router.push('/workflows')"
        >
          <ChevronLeft :size="14" />
          <span>Back</span>
        </button>
      </div>

      <!-- Window Actions Control Box -->
      <div class="flex items-center gap-1 border-l pl-3 border-border-default">
        <button
          id="window-minimize"
          class="w-6 h-6 flex items-center justify-center rounded-md hover:bg-bg-elevated transition-colors cursor-pointer"
          style="color: var(--color-text-secondary);"
          title="Minimize"
          @click="minimizeWindow"
        >
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><line x1="5" y1="12" x2="19" y2="12"></line></svg>
        </button>
        <button
          id="window-maximize"
          class="w-6 h-6 flex items-center justify-center rounded-md hover:bg-bg-elevated transition-colors cursor-pointer"
          style="color: var(--color-text-secondary);"
          title="Maximize"
          @click="toggleMaximizeWindow"
        >
          <svg width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect></svg>
        </button>
        <button
          id="window-close"
          class="w-6 h-6 flex items-center justify-center rounded-md hover:bg-red-500/10 hover:text-red-400 transition-colors cursor-pointer"
          style="color: var(--color-text-secondary);"
          title="Close"
          @click="closeWindow"
        >
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
        </button>
      </div>
    </div>
  </header>
</template>
