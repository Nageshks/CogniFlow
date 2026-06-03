<script setup lang="ts">
import { GitBranch, Clock, Star, Play, FolderOpen, Trash2 } from 'lucide-vue-next'
import { ref, computed } from 'vue'
import type { WorkflowWithSteps } from '@/types'
import { formatDate } from '@/lib/utils'
import { Card, CardHeader, CardTitle, CardContent, CardFooter } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { useFoldersStore } from '@/stores/folders'

const props = defineProps<{
  workflow: WorkflowWithSteps
  compact?: boolean
}>()

const emit = defineEmits<{
  (e: 'click'): void
  (e: 'delete'): void
}>()

// Persist starred state in localStorage (same as prompts)
const isStarred = ref(localStorage.getItem(`starred-${props.workflow.id}`) === 'true')

function toggleStar(event: Event) {
  event.stopPropagation()
  isStarred.value = !isStarred.value
  localStorage.setItem(`starred-${props.workflow.id}`, String(isStarred.value))
}

const foldersStore = useFoldersStore()
const folderName = computed(() => {
  if (!props.workflow.folder_id) return null
  return foldersStore.folders.find(f => f.id === props.workflow.folder_id)?.name ?? null
})

function getTagColor(tag: string) {
  return 'bg-gray-50 text-gray-600 border-gray-100 dark:bg-zinc-900 dark:border-zinc-800 dark:text-zinc-400'
}
</script>

<template>
  <Card
    v-if="!compact"
    class="group h-full min-h-[220px] bg-white border border-gray-200 rounded-2xl hover:border-emerald-250 hover:shadow-lg transition-all duration-200 cursor-pointer flex flex-col justify-between gap-0 ring-0 dark:bg-zinc-950 dark:border-zinc-800 dark:hover:border-emerald-900/80 py-4"
    @click="emit('click')"
  >
    <!-- Top Content Area -->
    <CardHeader class="px-5 py-0 gap-1 grid-cols-1">
      <!-- Title -->
      <CardTitle class="font-bold text-gray-900 leading-snug text-[15px] line-clamp-1 dark:text-gray-100">
        {{ workflow.title }}
      </CardTitle>
    </CardHeader>

    <CardContent class="px-5 py-0 mt-3 flex-1 flex flex-col justify-between">
      <!-- Steps Preview (numbered list of workflow steps instead of description) -->
      <div v-if="workflow.steps && workflow.steps.length > 0" class="flex flex-col gap-2 flex-1 justify-center py-1">
        <div 
          v-for="step in workflow.steps.slice(0, 3)" 
          :key="step.id"
          class="flex items-center gap-2.5 text-[12.5px] text-gray-600 dark:text-zinc-300 font-medium"
        >
          <span class="w-4 h-4 rounded-full bg-emerald-50 text-emerald-600 dark:bg-emerald-950/40 dark:text-emerald-400 flex items-center justify-center text-[10px] font-bold flex-shrink-0">
            {{ step.order_index }}
          </span>
          <span class="truncate">{{ step.title }}</span>
        </div>
        <div v-if="workflow.steps.length > 3" class="text-[11px] text-gray-500 dark:text-zinc-500 font-semibold pl-6.5 mt-0.5">
          + {{ workflow.steps.length - 3 }} more steps
        </div>
      </div>
      <div v-else class="flex flex-col items-center justify-center p-4 border border-dashed border-gray-150 rounded-xl flex-1 text-center dark:border-zinc-800 justify-center py-5">
        <span class="text-[11.5px] text-gray-500 dark:text-zinc-500 font-medium">No steps added yet</span>
      </div>

      <!-- Metadata: Badge + Tags -->
      <div class="flex flex-wrap items-center gap-2 mt-auto pt-3">
        <Badge
          variant="outline"
          class="text-[10px] font-bold uppercase tracking-wider px-2.5 py-1 rounded-full border bg-emerald-50 text-emerald-600 border-emerald-100 flex items-center gap-1.5 dark:bg-emerald-950/30 dark:text-emerald-400 dark:border-emerald-900/50"
        >
          <GitBranch :size="10" />
          Workflow
        </Badge>
        <Badge
          v-if="folderName"
          variant="secondary"
          class="flex items-center gap-1.5 text-[10px] font-medium px-2.5 py-1 rounded-full border bg-gray-50 text-gray-600 border-gray-100 dark:bg-zinc-900 dark:border-zinc-800 dark:text-zinc-400"
        >
          <FolderOpen :size="10" class="flex-shrink-0" />
          <span class="truncate max-w-[90px]">{{ folderName }}</span>
        </Badge>
        <Badge
          v-for="tag in (workflow.tags || []).slice(0, 2)"
          :key="tag"
          variant="outline"
          :class="['text-[11px] font-medium px-3 py-1 rounded-full border', getTagColor(tag)]"
        >
          {{ tag }}
        </Badge>
        <span v-if="(workflow.tags || []).length > 2" class="text-[11px] text-gray-400 font-medium dark:text-zinc-500">
          +{{ workflow.tags.length - 2 }}
        </span>
      </div>
    </CardContent>

    <!-- Actions & Footer -->
    <CardFooter class="px-5 py-0 mt-4 pt-3 border-t border-gray-100 flex flex-col gap-4 dark:border-zinc-800 bg-transparent rounded-none items-stretch">
      <!-- Primary Action -->
      <Button
        class="w-full flex items-center justify-center gap-2 h-10 rounded-xl bg-gray-900 text-white text-[13px] font-semibold hover:bg-emerald-600 active:scale-[0.98] transition-all duration-150 cursor-pointer shadow-sm border-0 dark:bg-zinc-100 dark:text-zinc-950 dark:hover:bg-emerald-500 dark:hover:text-white"
        @click.stop="emit('click')"
      >
        <Play :size="14" class="fill-current" />
        <span>Run Workflow</span>
      </Button>

      <!-- Date and Star row -->
      <div class="flex items-center justify-end gap-2.5 text-gray-400 dark:text-zinc-400 w-full">
        <div class="flex items-center gap-1.5 text-[11.5px]">
          <Clock :size="12" />
          <span>{{ formatDate(workflow.updated_at) }}</span>
        </div>
        <Button
          variant="ghost"
          class="p-1 size-7 hover:bg-gray-100 rounded-full transition-colors cursor-pointer -mr-1 dark:hover:bg-zinc-800"
          :class="isStarred ? 'text-yellow-400 hover:text-yellow-400' : 'text-gray-300 hover:text-yellow-400 dark:text-zinc-500'"
          @click.stop="toggleStar"
          title="Bookmark workflow"
        >
          <Star :size="15" :class="isStarred ? 'fill-yellow-400' : ''" />
        </Button>
        <Button
          variant="ghost"
          class="p-1 size-7 hover:bg-red-50 rounded-full transition-colors cursor-pointer text-gray-300 hover:text-red-500 dark:text-zinc-500 dark:hover:bg-red-950/30 dark:hover:text-red-400"
          @click.stop="emit('delete')"
          title="Delete workflow"
        >
          <Trash2 :size="14" />
        </Button>
      </div>
    </CardFooter>
  </Card>

  <!-- ─── Compact List Row ─── -->
  <Card
    v-else
    class="group flex flex-col md:flex-row md:items-center gap-3 md:gap-0 px-4 py-3 hover:bg-gray-50 transition-colors cursor-pointer border border-gray-100 hover:border-gray-250 shadow-none dark:border-zinc-900 dark:hover:bg-zinc-900/50 dark:hover:border-zinc-800 rounded-xl"
    @click="emit('click')"
  >
    <!-- Asset Name & Preview column -->
    <div class="flex-1 flex items-center gap-3 min-w-0">
      <div class="w-8 h-8 rounded-lg bg-emerald-50 flex items-center justify-center text-emerald-600 flex-shrink-0 dark:bg-emerald-950/30 dark:text-emerald-400">
        <GitBranch :size="15" />
      </div>
      <div class="min-w-0">
        <h3 class="font-bold text-gray-900 truncate text-[14px] dark:text-gray-100">{{ workflow.title }}</h3>
        <p v-if="workflow.steps && workflow.steps.length > 0" class="text-[12px] text-gray-400 line-clamp-1 dark:text-zinc-400 mt-0.5">
          {{ workflow.steps.map(s => s.title).join(' → ') }}
        </p>
        <p v-else class="text-[12px] text-gray-400 line-clamp-1 dark:text-zinc-400 mt-0.5">No steps added yet</p>
      </div>
    </div>

    <!-- Type column -->
    <div class="w-28 flex-shrink-0 hidden md:block">
      <Badge variant="outline" class="text-[10px] font-bold uppercase bg-emerald-50 text-emerald-600 border border-emerald-100 dark:bg-emerald-950/20 dark:text-emerald-400 dark:border-emerald-900/30 flex items-center gap-1 w-max">
        <GitBranch :size="9" />
        Workflow
      </Badge>
    </div>

    <!-- Folder column -->
    <div class="w-36 flex-shrink-0 hidden md:block">
      <div v-if="folderName" class="flex items-center gap-1.5 text-[12px] text-gray-500 dark:text-zinc-400">
        <FolderOpen :size="11" class="text-gray-400" />
        <span class="truncate max-w-[120px]">{{ folderName }}</span>
      </div>
      <span v-else class="text-[12px] text-gray-300 dark:text-zinc-700">—</span>
    </div>

    <!-- Tags column -->
    <div class="w-44 flex-shrink-0 hidden md:block">
      <div v-if="(workflow.tags || []).length > 0" class="flex items-center gap-1.5 flex-wrap">
        <Badge
          v-for="tag in workflow.tags.slice(0, 2)"
          :key="tag"
          variant="outline"
          :class="['text-[10px] font-medium px-2.5 py-0.5 border rounded', getTagColor(tag)]"
        >
          {{ tag }}
        </Badge>
        <span v-if="workflow.tags.length > 2" class="text-[10px] text-gray-450 font-medium">
          +{{ workflow.tags.length - 2 }}
        </span>
      </div>
      <span v-else class="text-[12px] text-gray-300 dark:text-zinc-700">—</span>
    </div>

    <!-- Stats & Updated Date column -->
    <div class="w-32 flex-shrink-0 hidden md:flex flex-col gap-0.5 items-end pr-6">
      <span class="text-[11px] font-bold text-gray-700 dark:text-zinc-300">{{ (workflow.steps || []).length }} steps</span>
      <span class="text-[10px] text-gray-400 dark:text-zinc-400">{{ formatDate(workflow.updated_at) }}</span>
    </div>

    <!-- Actions column -->
    <div class="w-20 flex-shrink-0 flex items-center justify-end gap-1">
      <div class="flex items-center gap-1 opacity-100 md:opacity-0 md:group-hover:opacity-100 transition-opacity duration-150">
        <Button
          variant="ghost"
          size="icon"
          class="h-8 w-8 text-gray-400 hover:text-emerald-650 hover:bg-emerald-50/50 transition-colors cursor-pointer dark:hover:bg-emerald-950/30 dark:hover:text-emerald-400"
          @click.stop="emit('click')"
          title="Run Workflow"
        >
          <Play :size="14" class="fill-current" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          class="h-8 w-8 transition-all cursor-pointer"
          :class="isStarred ? 'text-yellow-400 hover:bg-yellow-50/50 dark:hover:bg-yellow-950/30' : 'text-gray-300 hover:text-yellow-400 dark:text-zinc-500 hover:bg-gray-50 dark:hover:bg-zinc-800'"
          @click.stop="toggleStar"
          title="Bookmark"
        >
          <Star :size="14" :class="isStarred ? 'fill-yellow-400' : ''" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          class="h-8 w-8 text-gray-300 hover:text-red-500 hover:bg-red-50/50 transition-all cursor-pointer dark:text-zinc-500 dark:hover:bg-red-950/30 dark:hover:text-red-400"
          @click.stop="emit('delete')"
          title="Delete workflow"
        >
          <Trash2 :size="14" />
        </Button>
      </div>
    </div>

    <!-- Mobile-only Details row -->
    <div class="flex items-center gap-2 mt-1 md:hidden text-[11px] text-gray-400 dark:text-zinc-500">
      <span v-if="folderName" class="flex items-center gap-1">
        <FolderOpen :size="10" />
        <span>{{ folderName }}</span>
      </span>
      <span v-if="folderName">·</span>
      <span>Workflow</span>
      <span>·</span>
      <span>{{ (workflow.steps || []).length }} steps</span>
      <span>·</span>
      <span>Updated {{ formatDate(workflow.updated_at) }}</span>
    </div>
  </Card>
</template>
