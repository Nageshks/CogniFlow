<script setup lang="ts">
import { useRouter } from 'vue-router'
import { FileText, GitBranch } from 'lucide-vue-next'
import type { SearchResult } from '@/types'

defineProps<{
  results: SearchResult[]
}>()

const router = useRouter()

function navigateToResult(res: SearchResult) {
  if (res.entity_type === 'prompt') {
    // Open in Library
    router.push('/library')
  } else {
    // Open in Workflows
    router.push(`/workflows/${res.id}`)
  }
}
</script>

<template>
  <div class="search-results-panel glass-card p-4 animate-fade-in flex flex-col gap-3">
    <div class="text-xs font-semibold uppercase tracking-wider mb-1" style="color: var(--color-text-muted);">
      Search Results ({{ results.length }})
    </div>

    <div class="flex flex-col gap-2 max-h-96 overflow-y-auto pr-1">
      <div
        v-for="res in results"
        :key="res.id"
        class="result-item p-3 rounded-lg flex gap-3 items-start transition-all cursor-pointer"
        style="border: 1px solid var(--color-border-subtle); background: var(--color-bg-tertiary);"
        @click="navigateToResult(res)"
      >
        <div class="result-icon flex-shrink-0 p-1.5 rounded-md" :class="res.entity_type">
          <FileText v-if="res.entity_type === 'prompt'" :size="16" />
          <GitBranch v-else :size="16" />
        </div>

        <div class="flex-1 min-w-0">
          <div class="flex items-center justify-between gap-2 mb-1">
            <div class="result-title text-sm font-semibold truncate" style="color: var(--color-text-primary);">
              {{ res.title }}
            </div>
            <div class="score-badge text-[10px] px-1.5 py-0.5 rounded-full font-mono">
              Score: {{ (res.score * 100).toFixed(0) }}%
            </div>
          </div>
          <p class="result-snippet text-xs line-clamp-2" style="color: var(--color-text-secondary);">
            {{ res.snippet }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.result-item:hover {
  border-color: var(--color-accent-blue);
  background: var(--color-bg-elevated);
  transform: translateY(-1px);
}

.result-icon.prompt {
  background: hsl(210 100% 50% / 0.1);
  color: var(--color-accent-blue);
}

.result-icon.workflow {
  background: hsl(270 100% 60% / 0.1);
  color: var(--color-accent-violet);
}

.score-badge {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-default);
  color: var(--color-accent-emerald);
}
</style>
