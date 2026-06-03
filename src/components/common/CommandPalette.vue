<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { Search, FileText, X, ArrowRight, GitBranch } from 'lucide-vue-next'
import { usePromptsStore } from '@/stores/prompts'
import { useWorkflowsStore } from '@/stores/workflows'
import { useRouter } from 'vue-router'

const isOpen = ref(false)
const query = ref('')
const selectedIndex = ref(0)
const inputRef = ref<HTMLInputElement | null>(null)

const promptsStore = usePromptsStore()
const workflowsStore = useWorkflowsStore()
const router = useRouter()

// Local results for Prompts
const promptResults = computed(() => {
  if (!query.value.trim()) return promptsStore.prompts.slice(0, 5)
  const q = query.value.toLowerCase()
  return promptsStore.prompts.filter(p => p.title.toLowerCase().includes(q)).slice(0, 5)
})

// Local results for Workflows
const workflowResults = computed(() => {
  if (!query.value.trim()) return workflowsStore.workflows.slice(0, 5)
  const q = query.value.toLowerCase()
  return workflowsStore.workflows.filter(w => w.title.toLowerCase().includes(q)).slice(0, 5)
})

// Combined results
const allResults = computed(() => {
  const prompts = promptResults.value.map(p => ({ ...p, type: 'prompt' as const }))
  const workflows = workflowResults.value.map(w => ({ ...w, type: 'workflow' as const }))
  return [...prompts, ...workflows]
})

function openPalette() {
  isOpen.value = true
  query.value = ''
  selectedIndex.value = 0
}

function closePalette() {
  isOpen.value = false
  query.value = ''
  selectedIndex.value = 0
}

function moveDown() {
  if (allResults.value.length > 0) {
    selectedIndex.value = (selectedIndex.value + 1) % allResults.value.length
  }
}

function moveUp() {
  if (allResults.value.length > 0) {
    selectedIndex.value = (selectedIndex.value - 1 + allResults.value.length) % allResults.value.length
  }
}

function selectResult() {
  if (allResults.value.length > 0 && allResults.value[selectedIndex.value]) {
    const item = allResults.value[selectedIndex.value]
    if (item.type === 'prompt') {
      navigateToPrompt(item.id)
    } else {
      navigateToWorkflow(item.id)
    }
  }
}

function handleKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
    e.preventDefault()
    if (isOpen.value) {
      closePalette()
    } else {
      openPalette()
    }
    return
  }
  if (!isOpen.value) return
  if (e.key === 'Escape') {
    closePalette()
  }
}

function navigateToPrompt(promptId: string) {
  closePalette()
  router.push('/library')
  // Emit event so LibraryView can open the detail sheet
  setTimeout(() => {
    window.dispatchEvent(new CustomEvent('open-prompt-detail', { detail: { promptId } }))
  }, 100)
}

function navigateToWorkflow(workflowId: string) {
  closePalette()
  router.push(`/workflows/${workflowId}`)
}

watch(isOpen, (newVal) => {
  if (newVal) {
    setTimeout(() => {
      inputRef.value?.focus()
    }, 100)
  }
})

watch(query, () => {
  selectedIndex.value = 0
})

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
  promptsStore.fetchPrompts()
  workflowsStore.fetchWorkflows()
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <Teleport to="body">
    <Transition name="palette-fade">
      <div
        v-if="isOpen"
        class="fixed inset-0 z-[200] bg-black/50 backdrop-blur-sm"
        @mousedown.self="closePalette"
      >
        <div
          class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] max-h-[480px] bg-white dark:bg-zinc-950 rounded-2xl shadow-2xl border border-gray-100 dark:border-zinc-850 flex flex-col overflow-hidden"
          @click.stop
        >
          <!-- Search Input Row -->
          <div class="flex items-center gap-3 px-5 py-4 border-b border-gray-100 dark:border-zinc-850">
            <Search :size="18" class="text-gray-400 dark:text-zinc-500 flex-shrink-0" />
            <input
              ref="inputRef"
              v-model="query"
              type="text"
              placeholder="Search prompts, workflows…"
              class="flex-1 text-[15px] text-gray-900 dark:text-zinc-150 bg-transparent border-none outline-none placeholder-gray-400 dark:placeholder-zinc-500"
              @keydown.down.prevent="moveDown"
              @keydown.up.prevent="moveUp"
              @keydown.enter.prevent="selectResult"
              @keydown.esc.prevent="closePalette"
            />
            <button @click="closePalette" class="p-1 rounded-lg hover:bg-gray-100 dark:hover:bg-zinc-900 text-gray-400 dark:text-zinc-550 cursor-pointer transition-colors">
              <X :size="16" />
            </button>
          </div>

          <!-- Results -->
          <div class="flex-1 overflow-y-auto py-2">
            <div v-if="allResults.length === 0" class="py-12 text-center text-gray-400 dark:text-zinc-500 text-sm">
              No results found for "{{ query }}"
            </div>

            <div v-else class="space-y-1">
              <button
                v-for="(item, idx) in allResults"
                :key="item.id"
                class="w-full flex items-center gap-3 px-4 py-3 transition-colors cursor-pointer text-left group border-l-2"
                :class="[
                  selectedIndex === idx 
                    ? 'bg-indigo-50/70 border-indigo-500 text-indigo-900 dark:bg-zinc-900 dark:border-indigo-500 dark:text-zinc-100' 
                    : 'hover:bg-gray-50 border-transparent text-gray-900 dark:hover:bg-zinc-900/50 dark:text-zinc-300'
                ]"
                @click="item.type === 'prompt' ? navigateToPrompt(item.id) : navigateToWorkflow(item.id)"
                @mouseenter="selectedIndex = idx"
              >
                <!-- Icon -->
                <div 
                  class="w-8 h-8 rounded-lg flex items-center justify-center flex-shrink-0"
                  :class="[
                    item.type === 'prompt' 
                      ? 'bg-indigo-50 border border-indigo-100 text-indigo-500 dark:bg-indigo-950/30 dark:border-indigo-900/50 dark:text-indigo-400' 
                      : 'bg-emerald-50 border border-emerald-100 text-emerald-600 dark:bg-emerald-950/30 dark:border-emerald-900/50 dark:text-emerald-450'
                  ]"
                >
                  <FileText v-if="item.type === 'prompt'" :size="15" />
                  <GitBranch v-else :size="15" />
                </div>

                <!-- Info -->
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2">
                    <span 
                      class="text-[9px] font-extrabold uppercase tracking-wider px-1.5 py-0.5 rounded flex items-center gap-1 border"
                      :class="[
                        item.type === 'prompt' 
                          ? 'bg-indigo-50/80 text-indigo-600 border-indigo-100 dark:bg-indigo-950/40 dark:text-indigo-400 dark:border-indigo-900/40' 
                          : 'bg-emerald-50/80 text-emerald-600 border-emerald-100 dark:bg-emerald-950/40 dark:text-emerald-450 dark:border-emerald-900/40'
                      ]"
                    >
                      {{ item.type }}
                    </span>
                    <div 
                      class="text-[14px] font-semibold truncate"
                      :class="selectedIndex === idx ? 'text-indigo-950 dark:text-zinc-100' : 'text-gray-900 dark:text-zinc-300'"
                    >
                      {{ item.title }}
                    </div>
                  </div>
                  <div class="text-[12px] text-gray-400 dark:text-zinc-500 truncate font-mono mt-0.5">
                    {{ item.type === 'prompt' ? item.content?.slice(0, 60) : (item.description || 'No description provided').slice(0, 60) }}...
                  </div>
                </div>

                <!-- Arrow -->
                <ArrowRight 
                  :size="14" 
                  class="flex-shrink-0 transition-colors"
                  :class="selectedIndex === idx ? 'text-indigo-500 dark:text-indigo-400' : 'text-gray-300 dark:text-zinc-700'"
                />
              </button>
            </div>
          </div>

          <!-- Footer hint -->
          <div class="px-5 py-3 border-t border-gray-100 dark:border-zinc-850 flex items-center gap-4 text-[11px] text-gray-400 dark:text-zinc-500 bg-gray-50/50 dark:bg-zinc-950/50">
            <span><kbd class="bg-gray-100 dark:bg-zinc-900 px-1.5 py-0.5 rounded text-[10px] font-mono border border-gray-200 dark:border-zinc-800">↑↓</kbd> to navigate</span>
            <span><kbd class="bg-gray-100 dark:bg-zinc-900 px-1.5 py-0.5 rounded text-[10px] font-mono border border-gray-200 dark:border-zinc-800">↵</kbd> to open</span>
            <span><kbd class="bg-gray-100 dark:bg-zinc-900 px-1.5 py-0.5 rounded text-[10px] font-mono border border-gray-200 dark:border-zinc-800">Esc</kbd> to close</span>
            <span><kbd class="bg-gray-100 dark:bg-zinc-900 px-1.5 py-0.5 rounded text-[10px] font-mono border border-gray-200 dark:border-zinc-800">Ctrl+K</kbd> to toggle</span>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.palette-fade-enter-active,
.palette-fade-leave-active {
  transition: opacity 0.15s ease;
}
.palette-fade-enter-from,
.palette-fade-leave-to {
  opacity: 0;
}
</style>
