<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { FileText, Star, Clock, Play, FolderOpen, Plus } from 'lucide-vue-next'
import type { PromptWithMeta } from '@/types'
import { formatDate } from '@/lib/utils'
import { useFoldersStore } from '@/stores/folders'
import { Card, CardHeader, CardTitle, CardContent, CardFooter } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'

const props = defineProps<{
  prompt?: PromptWithMeta
  compact?: boolean
  isNewCard?: boolean
}>()

const emit = defineEmits<{
  (e: 'use', prompt: PromptWithMeta): void
  (e: 'create'): void
}>()

const foldersStore = useFoldersStore()
const folderName = computed(() => {
  if (!props.prompt || !props.prompt.folder_id) return null
  return foldersStore.folders.find(f => f.id === props.prompt.folder_id)?.name ?? null
})

// Star state
const isStarred = ref(false)
function toggleStar(event: Event) {
  if (!props.prompt) return
  event.stopPropagation()
  isStarred.value = !isStarred.value
  localStorage.setItem(`starred-${props.prompt.id}`, String(isStarred.value))
}

// Usage count
const localUsageCount = ref(0)

watch(
  () => props.prompt?.id,
  (newId) => {
    if (!newId) {
      isStarred.value = false
      localUsageCount.value = 0
      return
    }
    isStarred.value = localStorage.getItem(`starred-${newId}`) === 'true'
    localUsageCount.value = parseInt(localStorage.getItem(`usage-${newId}`) || '0')
  },
  { immediate: true }
)

function handlePromptUsedEvent(e: Event) {
  const customEvent = e as CustomEvent
  if (props.prompt && customEvent.detail?.id === props.prompt.id) {
    localUsageCount.value = parseInt(localStorage.getItem(`usage-${props.prompt.id}`) || '0')
  }
}
onMounted(() => window.addEventListener('prompt-used', handlePromptUsedEvent))
onUnmounted(() => window.removeEventListener('prompt-used', handlePromptUsedEvent))

// Consistent color hashing for tags (not everything purple)
const tagColorMap: Record<string, string> = {}
const colorPool = [
  'bg-violet-50 text-violet-600 border-violet-100 dark:bg-violet-950/30 dark:text-violet-400 dark:border-violet-900/50',
  'bg-blue-50 text-blue-600 border-blue-100 dark:bg-blue-950/30 dark:text-blue-400 dark:border-blue-900/50',
  'bg-emerald-50 text-emerald-600 border-emerald-100 dark:bg-emerald-950/30 dark:text-emerald-400 dark:border-emerald-900/50',
  'bg-amber-50 text-amber-600 border-amber-100 dark:bg-amber-950/30 dark:text-amber-400 dark:border-amber-900/50',
  'bg-rose-50 text-rose-600 border-rose-100 dark:bg-rose-950/30 dark:text-rose-400 dark:border-rose-900/50',
  'bg-cyan-50 text-cyan-600 border-cyan-100 dark:bg-cyan-950/30 dark:text-cyan-400 dark:border-cyan-900/50',
  'bg-fuchsia-50 text-fuchsia-600 border-fuchsia-100 dark:bg-fuchsia-950/30 dark:text-fuchsia-400 dark:border-fuchsia-900/50',
]
function getTagColor(tag: string) {
  if (!tagColorMap[tag]) {
    let hash = 0
    for (let i = 0; i < tag.length; i++) hash = tag.charCodeAt(i) + ((hash << 5) - hash)
    tagColorMap[tag] = colorPool[Math.abs(hash) % colorPool.length]
  }
  return tagColorMap[tag]
}

// Click handlers
function handleUseClick(event: Event) {
  if (!props.prompt) return
  event.stopPropagation()
  emit('use', props.prompt)
}
function handleCreateClick(event: Event) {
  event.stopPropagation()
  emit('create')
}
</script>

<template>
  <!-- ─── Empty State: Create New Prompt ─── -->
  <Card
    v-if="isNewCard"
    class="group h-full bg-white dark:bg-zinc-950/30 border-2 border-dashed border-gray-200 dark:border-zinc-800 rounded-2xl hover:border-indigo-300 dark:hover:border-indigo-800 hover:bg-indigo-50/20 dark:hover:bg-indigo-950/10 transition-all duration-200 flex flex-col items-center justify-center cursor-pointer gap-3" style="min-height: 220px"
    @click="handleCreateClick"
  >
    <div class="w-12 h-12 rounded-xl bg-indigo-50 border border-indigo-100 flex items-center justify-center text-indigo-500 group-hover:scale-110 group-hover:bg-indigo-100 transition-all duration-200 dark:bg-indigo-950/30 dark:border-indigo-900/50">
      <Plus :size="22" stroke-width="2.5" />
    </div>
    <div class="text-center">
      <h3 class="font-semibold text-gray-800 text-[14px] mb-0.5 dark:text-gray-200">Create New Prompt</h3>
      <p class="text-[12px] text-gray-400 dark:text-gray-550">Start from a blank slate</p>
    </div>
  </Card>

  <!-- ─── Full Card ─── -->
  <Card
    v-else-if="!compact && prompt"
    class="group flex flex-col h-full rounded-2xl hover:border-indigo-200 hover:shadow-lg transition-all duration-200 cursor-pointer overflow-hidden dark:hover:border-indigo-900/80"
  >
    <!-- Top Content Area -->
    <CardHeader class="p-5 pb-3">
      <CardTitle class="font-bold text-gray-900 leading-snug text-[16px] line-clamp-1 dark:text-gray-100">
        {{ prompt.title }}
      </CardTitle>
    </CardHeader>

    <CardContent class="px-5 py-0 flex-1 flex flex-col">
      <!-- Content 2-line Preview -->
      <p class="text-[14px] text-gray-500 line-clamp-2 leading-relaxed dark:text-gray-400 mb-4">
        {{ prompt.content }}
      </p>

      <!-- Metadata: Folder + Tags -->
      <div class="flex flex-wrap items-center gap-2 mt-auto pb-4">
        <Badge
          v-if="folderName"
          variant="secondary"
          class="flex items-center gap-1.5 text-[11px] px-2.5 py-0.5"
        >
          <FolderOpen :size="10" class="flex-shrink-0" />
          <span class="truncate max-w-[90px]">{{ folderName }}</span>
        </Badge>
        <div v-if="prompt.tags?.length > 0" class="flex items-center gap-2 flex-wrap">
          <Badge
            v-for="tag in prompt.tags.slice(0, 2)"
            :key="tag"
            variant="outline"
            :class="['text-[11px] font-medium px-2.5 py-0.5 border', getTagColor(tag)]"
          >
            {{ tag }}
          </Badge>
          <span v-if="prompt.tags.length > 2" class="text-[11px] text-gray-400 font-medium dark:text-gray-500">
            +{{ prompt.tags.length - 2 }}
          </span>
        </div>
      </div>
    </CardContent>

    <!-- Actions & Footer -->
    <CardFooter class="px-5 py-4 border-t border-gray-100 bg-gray-50/50 flex flex-col gap-4 dark:border-zinc-850 dark:bg-zinc-900/20 items-stretch">
      <!-- Primary Action -->
      <Button
        class="w-full flex items-center justify-center gap-2 h-9 rounded-xl shadow-none transition-all duration-150 cursor-pointer"
        @click="handleUseClick"
      >
        <Play :size="13" class="fill-current" />
        <span>Use Prompt</span>
      </Button>

      <!-- Usage count, Date, and Star row -->
      <div class="flex items-center justify-between w-full">
        <Badge
          variant="outline"
          class="text-[11px] font-semibold px-2.5 py-0.5 rounded-full border flex items-center gap-1.5 h-6"
          :class="localUsageCount > 0 ? 'bg-emerald-50 text-emerald-600 border-emerald-100 dark:bg-emerald-950/30 dark:text-emerald-450 dark:border-emerald-900/50' : 'bg-gray-50 text-gray-400 border-gray-200 dark:bg-zinc-900 dark:text-zinc-550 dark:border-zinc-800'"
        >
          <Play :size="8" :class="localUsageCount > 0 ? 'fill-emerald-600 text-emerald-600 dark:fill-emerald-450 dark:text-emerald-450' : 'fill-gray-400 text-gray-400 dark:fill-zinc-500 dark:text-zinc-500'" />
          <span>{{ localUsageCount }} used</span>
        </Badge>

        <div class="flex items-center gap-3 text-gray-400 dark:text-gray-550">
          <div class="flex items-center gap-1 text-[11.5px]">
            <Clock :size="12" />
            <span>{{ formatDate(prompt.updated_at) }}</span>
          </div>
          <Button
            variant="ghost"
            size="icon"
            class="h-7 w-7 rounded-full"
            :class="isStarred ? 'text-yellow-400 hover:text-yellow-400' : 'text-gray-300 hover:text-yellow-400 dark:text-zinc-650'"
            @click="toggleStar"
            title="Bookmark prompt"
          >
            <Star :size="15" :class="isStarred ? 'fill-yellow-400' : ''" />
          </Button>
        </div>
      </div>
    </CardFooter>
  </Card>

  <!-- ─── Compact List Row ─── -->
  <Card
    v-else-if="prompt"
    class="group flex flex-col md:flex-row md:items-center gap-3 md:gap-0 px-4 py-3 hover:bg-gray-50 transition-colors cursor-pointer border border-gray-100 hover:border-gray-250 shadow-none dark:border-zinc-900 dark:hover:bg-zinc-900/50 dark:hover:border-zinc-800 rounded-xl"
  >
    <!-- Asset Name & Preview column -->
    <div class="flex-1 flex items-center gap-3 min-w-0">
      <div class="w-8 h-8 rounded-lg bg-indigo-50 flex items-center justify-center text-indigo-500 flex-shrink-0 dark:bg-indigo-950/30 dark:text-indigo-400">
        <FileText :size="15" />
      </div>
      <div class="min-w-0">
        <h3 class="font-bold text-gray-900 truncate text-[14px] dark:text-gray-100">{{ prompt.title }}</h3>
        <p class="text-[12px] text-gray-400 line-clamp-1 dark:text-zinc-500 mt-0.5">{{ prompt.content }}</p>
      </div>
    </div>

    <!-- Type column -->
    <div class="w-28 flex-shrink-0 hidden md:block">
      <Badge variant="outline" class="text-[10px] font-bold uppercase bg-indigo-50 text-indigo-650 border border-indigo-100 dark:bg-indigo-950/20 dark:text-indigo-400 dark:border-indigo-900/30">
        Prompt
      </Badge>
    </div>

    <!-- Folder column -->
    <div class="w-36 flex-shrink-0 hidden md:block">
      <div v-if="folderName" class="flex items-center gap-1.5 text-[12px] text-gray-500 dark:text-zinc-400">
        <FolderOpen :size="11" class="text-gray-450" />
        <span class="truncate max-w-[120px]">{{ folderName }}</span>
      </div>
      <span v-else class="text-[12px] text-gray-300 dark:text-zinc-700">—</span>
    </div>

    <!-- Tags column -->
    <div class="w-44 flex-shrink-0 hidden md:block">
      <div v-if="prompt.tags?.length > 0" class="flex items-center gap-1.5 flex-wrap">
        <Badge
          v-for="tag in prompt.tags.slice(0, 2)"
          :key="tag"
          variant="outline"
          :class="['text-[10px] font-medium px-2.5 py-0.5 border rounded', getTagColor(tag)]"
        >
          {{ tag }}
        </Badge>
        <span v-if="prompt.tags.length > 2" class="text-[10px] text-gray-450 font-medium">
          +{{ prompt.tags.length - 2 }}
        </span>
      </div>
      <span v-else class="text-[12px] text-gray-300 dark:text-zinc-700">—</span>
    </div>

    <!-- Stats & Updated Date column -->
    <div class="w-32 flex-shrink-0 hidden md:flex flex-col gap-0.5 items-end pr-6">
      <span class="text-[11px] font-bold text-gray-700 dark:text-zinc-300">{{ localUsageCount }} uses</span>
      <span class="text-[10px] text-gray-400 dark:text-zinc-550">{{ formatDate(prompt.updated_at) }}</span>
    </div>

    <!-- Actions column -->
    <div class="w-20 flex-shrink-0 flex items-center justify-end gap-1">
      <div class="flex items-center gap-1 opacity-100 md:opacity-0 md:group-hover:opacity-100 transition-opacity duration-150">
        <Button
          variant="ghost"
          size="icon"
          class="h-8 w-8 text-gray-400 hover:text-indigo-650 hover:bg-indigo-50/50 transition-colors cursor-pointer dark:hover:bg-indigo-950/30 dark:hover:text-indigo-400"
          @click="handleUseClick"
          title="Use Prompt"
        >
          <Play :size="14" class="fill-current" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          class="h-8 w-8 transition-all cursor-pointer"
          :class="isStarred ? 'text-yellow-400 hover:bg-yellow-50/50 dark:hover:bg-yellow-950/30' : 'text-gray-300 hover:text-yellow-400 dark:text-zinc-650 hover:bg-gray-50 dark:hover:bg-zinc-800'"
          @click="toggleStar"
          title="Bookmark"
        >
          <Star :size="14" :class="isStarred ? 'fill-yellow-400' : ''" />
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
      <span>{{ localUsageCount }} uses</span>
      <span>·</span>
      <span>Updated {{ formatDate(prompt.updated_at) }}</span>
    </div>
  </Card>
</template>

<style scoped>
/* Ensure line-clamp works if not using Tailwind plugin */
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
