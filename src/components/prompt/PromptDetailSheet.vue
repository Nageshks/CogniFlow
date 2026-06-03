<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { X, Copy, Check, Edit2, History, RotateCcw, Play, FileText, Tag, FolderOpen, Clock, Trash2, AlertTriangle, GitBranch } from 'lucide-vue-next'
import * as api from '@/lib/tauri-api'
import type { PromptWithMeta, PromptVersion } from '@/types'
import { formatDate } from '@/lib/utils'
import { toast } from 'vue-sonner'
import { useFoldersStore } from '@/stores/folders'
import { usePromptsStore } from '@/stores/prompts'

const props = defineProps<{
  promptId: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'edit'): void
  (e: 'use', prompt: PromptWithMeta): void
  (e: 'duplicate', prompt: PromptWithMeta): void
}>()

const promptsStore = usePromptsStore()
const showDeleteConfirm = ref(false)
const deleting = ref(false)
const linkedWorkflows = ref<any[]>([])

const promptMeta = ref<PromptWithMeta | null>(null)
const versions = ref<PromptVersion[]>([])
const loading = ref(false)
const copied = ref(false)
const selectedVersionId = ref<string | null>(null)

const selectedVersion = computed(() => {
  if (!selectedVersionId.value) return null
  return versions.value.find(v => v.id === selectedVersionId.value) ?? null
})

// Highlight variables helper: e.g. {{variable}}
const formattedContent = computed(() => {
  if (!promptMeta.value) return ''
  return highlightVariables(promptMeta.value.content)
})

function highlightVariables(text: string): string {
  if (!text) return ''
  const escaped = escapeHtml(text)

  return escaped.replace(
    /\{\{([^}]+)\}\}/g,
    '<span class="inline-block px-1.5 py-0.5 mx-0.5 rounded font-bold text-[11px] bg-indigo-500/20 text-indigo-300 border border-indigo-500/30 animate-pulse-subtle">$1</span>'
  )
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;')
}

// Visual Line Diffing Algorithm (LCS method)
interface DiffLine {
  type: 'added' | 'removed' | 'unchanged'
  text: string
}

function computeLineDiff(oldStr: string, newStr: string): DiffLine[] {
  const oldLines = oldStr.split('\n')
  const newLines = newStr.split('\n')
  
  const dp: number[][] = Array(oldLines.length + 1).fill(null).map(() => Array(newLines.length + 1).fill(0))
  for (let i = 1; i <= oldLines.length; i++) {
    for (let j = 1; j <= newLines.length; j++) {
      if (oldLines[i - 1] === newLines[j - 1]) {
        dp[i][j] = dp[i - 1][j - 1] + 1
      } else {
        dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1])
      }
    }
  }

  const diff: DiffLine[] = []
  let i = oldLines.length
  let j = newLines.length

  while (i > 0 || j > 0) {
    if (i > 0 && j > 0 && oldLines[i - 1] === newLines[j - 1]) {
      diff.unshift({ type: 'unchanged', text: oldLines[i - 1] })
      i--
      j--
    } else if (j > 0 && (i === 0 || dp[i][j - 1] >= dp[i - 1][j])) {
      diff.unshift({ type: 'added', text: newLines[j - 1] })
      j--
    } else {
      diff.unshift({ type: 'removed', text: oldLines[i - 1] })
      i--
    }
  }

  return diff
}

// Word-Level Diffing Algorithm (LCS method)
interface WordDiffToken {
  type: 'added' | 'removed' | 'unchanged'
  text: string
}

function computeWordDiff(oldStr: string, newStr: string): WordDiffToken[] {
  const tokenRegex = /(\s+|[^\s\w]+|\w+)/g
  const oldTokens = oldStr.match(tokenRegex) || []
  const newTokens = newStr.match(tokenRegex) || []

  const dp: number[][] = Array(oldTokens.length + 1).fill(null).map(() => Array(newTokens.length + 1).fill(0))
  for (let i = 1; i <= oldTokens.length; i++) {
    for (let j = 1; j <= newTokens.length; j++) {
      if (oldTokens[i - 1] === newTokens[j - 1]) {
        dp[i][j] = dp[i - 1][j - 1] + 1
      } else {
        dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1])
      }
    }
  }

  const diff: WordDiffToken[] = []
  let i = oldTokens.length
  let j = newTokens.length

  while (i > 0 || j > 0) {
    if (i > 0 && j > 0 && oldTokens[i - 1] === newTokens[j - 1]) {
      diff.unshift({ type: 'unchanged', text: oldTokens[i - 1] })
      i--
      j--
    } else if (j > 0 && (i === 0 || dp[i][j - 1] >= dp[i - 1][j])) {
      diff.unshift({ type: 'added', text: newTokens[j - 1] })
      j--
    } else {
      diff.unshift({ type: 'removed', text: oldTokens[i - 1] })
      i--
    }
  }

  return diff
}

function highlightWordDiff(oldLine: string, newLine: string, type: 'added' | 'removed'): string {
  const diff = computeWordDiff(oldLine, newLine)
  let html = ''
  for (const token of diff) {
    if (token.type === 'unchanged') {
      html += escapeHtml(token.text)
    } else if (token.type === 'removed' && type === 'removed') {
      html += `<span class="bg-rose-500/35 text-rose-100 px-1 py-0.5 rounded font-bold border border-rose-500/20">${escapeHtml(token.text)}</span>`
    } else if (token.type === 'added' && type === 'added') {
      html += `<span class="bg-emerald-500/35 text-emerald-100 px-1 py-0.5 rounded font-bold border border-emerald-500/20">${escapeHtml(token.text)}</span>`
    }
  }
  return html
}

interface ProcessedDiffLine {
  type: 'added' | 'removed' | 'unchanged'
  html: string
}

function getWordLevelDiff(oldStr: string, newStr: string): ProcessedDiffLine[] {
  const lineDiff = computeLineDiff(oldStr, newStr)
  const result: ProcessedDiffLine[] = []

  let idx = 0
  while (idx < lineDiff.length) {
    const current = lineDiff[idx]
    const next = lineDiff[idx + 1]

    if (current.type === 'removed' && next && next.type === 'added') {
      const oldLine = current.text
      const newLine = next.text

      result.push({
        type: 'removed',
        html: highlightWordDiff(oldLine, newLine, 'removed')
      })
      result.push({
        type: 'added',
        html: highlightWordDiff(oldLine, newLine, 'added')
      })
      idx += 2
    } else {
      result.push({
        type: current.type,
        html: current.type === 'unchanged' 
          ? escapeHtml(current.text)
          : current.type === 'added'
            ? `<span class="bg-emerald-500/10 text-emerald-300 px-1 rounded">${escapeHtml(current.text)}</span>`
            : `<span class="bg-rose-500/10 text-rose-300 px-1 rounded line-through">${escapeHtml(current.text)}</span>`
      })
      idx++
    }
  }
  return result
}

const foldersStore = useFoldersStore()
const folderName = computed(() => {
  if (!promptMeta.value || !promptMeta.value.folder_id) return null
  return foldersStore.folders.find(f => f.id === promptMeta.value.folder_id)?.name ?? null
})

// Starred tag color logic (similar to PromptCard)
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

function toggleVersionPreview(versionId: string) {
  selectedVersionId.value = selectedVersionId.value === versionId ? null : versionId
}

async function loadData() {
  loading.value = true
  try {
    await foldersStore.fetchFolders()
    promptMeta.value = await api.getPrompt(props.promptId)
    versions.value = await api.getPromptVersions(props.promptId)
    linkedWorkflows.value = await api.getWorkflowsUsingPrompt(props.promptId)
  } catch (e) {
    toast.error('Failed to load prompt details: ' + String(e))
  } finally {
    loading.value = false
  }
}

onMounted(loadData)

async function copyContent() {
  const textToCopy = selectedVersion.value ? selectedVersion.value.content : (promptMeta.value?.content || '')
  if (!textToCopy) return
  try {
    await navigator.clipboard.writeText(textToCopy)
    copied.value = true
    toast.success('Copied to clipboard')
    
    // Track usage count
    const currentCount = parseInt(localStorage.getItem(`usage-${props.promptId}`) || '0')
    localStorage.setItem(`usage-${props.promptId}`, (currentCount + 1).toString())
    window.dispatchEvent(new CustomEvent('prompt-used', { detail: { id: props.promptId } }))

    setTimeout(() => { copied.value = false }, 2000)
  } catch (e) {
    toast.error('Failed to copy: ' + String(e))
  }
}

async function restoreVersion(versionNo: number) {
  if (confirm(`Restore to version ${versionNo}?`)) {
    try {
      await api.restorePromptVersion(props.promptId, versionNo)
      toast.success(`Restored to v${versionNo}`)
      selectedVersionId.value = null
      await loadData()
    } catch (e) {
      toast.error('Failed to restore: ' + String(e))
    }
  }
}

function handleUsePrompt() {
  if (selectedVersion.value) {
    const runPrompt = {
      ...promptMeta.value,
      content: selectedVersion.value.content,
      current_version: selectedVersion.value.version_number
    } as PromptWithMeta
    emit('use', runPrompt)
  } else if (promptMeta.value) {
    emit('use', promptMeta.value)
  }
}

function handleDuplicatePrompt() {
  if (promptMeta.value) emit('duplicate', promptMeta.value)
}

async function handleDeletePrompt() {
  deleting.value = true
  try {
    await promptsStore.deletePrompt(props.promptId)
    showDeleteConfirm.value = false
    emit('close')
  } catch (e) {
    toast.error('Failed to delete prompt: ' + String(e))
  } finally {
    deleting.value = false
  }
}

function onBackdropClick(e: MouseEvent) {
  if (e.target === e.currentTarget) emit('close')
}
</script>

<template>
  <Teleport to="body">
    <div
      class="fixed inset-0 z-50 flex items-center justify-center p-6 bg-black/50 backdrop-blur-sm"
      @mousedown="onBackdropClick"
    >
      <div
        class="relative bg-white rounded-2xl shadow-2xl border border-gray-200/80 w-full max-w-5xl h-[88vh] max-h-[900px] flex flex-col overflow-hidden animate-scale-in"
        @click.stop
      >
        <!-- Loading state -->
        <div v-if="loading" class="flex-1 flex items-center justify-center p-20">
          <div class="w-8 h-8 rounded-full border-2 border-indigo-200 border-t-indigo-600 animate-spin" />
        </div>

        <template v-else-if="promptMeta">
          <!-- Header -->
          <div class="flex items-center justify-between px-8 py-5 border-b border-gray-100 flex-shrink-0 bg-white">
            <div class="flex items-center gap-4 min-w-0">
              <div class="w-11 h-11 rounded-xl bg-gradient-to-br from-indigo-500 to-violet-600 flex items-center justify-center text-white flex-shrink-0 shadow-sm">
                <FileText :size="19" />
              </div>
              <div class="min-w-0">
                <h2 class="font-bold text-gray-900 text-[18px] leading-tight truncate max-w-[440px]">
                  {{ promptMeta.title }}
                </h2>
                <!-- Metadata row (folder, clock, tags) -->
                <div class="flex flex-wrap items-center gap-3 mt-1.5 text-gray-500">
                  <span class="text-[10px] font-mono font-bold px-2 py-0.5 rounded-full bg-gray-100 text-gray-500 border border-gray-200">
                    v{{ promptMeta.current_version }}
                  </span>
                  <span v-if="folderName" class="flex items-center gap-1.5 text-[11px] font-medium text-gray-400">
                    <FolderOpen :size="11" />
                    <span>{{ folderName }}</span>
                  </span>
                  <span class="flex items-center gap-1.5 text-[11px] font-medium text-gray-400">
                    <Clock :size="11" />
                    <span>Updated {{ formatDate(promptMeta.updated_at) }}</span>
                  </span>
                  <div v-if="promptMeta.tags?.length > 0" class="flex items-center gap-2 flex-wrap">
                    <span
                      v-for="tag in promptMeta.tags"
                      :key="tag"
                      class="inline-flex items-center gap-1.5 text-[11px] font-semibold px-2.5 py-0.5 rounded-md bg-gray-50 border border-gray-150 text-gray-500 dark:bg-zinc-900/30 dark:border-zinc-850 dark:text-zinc-400 select-none shadow-sm"
                    >
                      <Tag :size="10" class="text-indigo-500 dark:text-indigo-400 opacity-70" />
                      {{ tag }}
                    </span>
                  </div>
                </div>
              </div>
            </div>

            <div class="flex items-center gap-2.5 flex-shrink-0">
              <button
                class="flex items-center gap-2 px-4 py-2 rounded-xl text-[13px] font-semibold border border-red-200 bg-red-50/50 hover:bg-red-50 text-red-650 hover:border-red-300 transition-all cursor-pointer shadow-sm dark:bg-red-950/20 dark:border-red-900/50 dark:text-red-400 dark:hover:bg-red-950/30"
                @click="showDeleteConfirm = true"
              >
                <Trash2 :size="14" />
                <span>Delete</span>
              </button>
              <button
                class="flex items-center gap-2 px-4 py-2 rounded-xl text-[13px] font-semibold border border-gray-200 bg-white text-gray-600 hover:bg-gray-50 hover:text-gray-900 hover:border-gray-300 transition-all cursor-pointer shadow-sm"
                @click="handleDuplicatePrompt"
              >
                <Copy :size="14" />
                <span>Duplicate</span>
              </button>
              <button
                class="flex items-center gap-2 px-4 py-2 rounded-xl text-[13px] font-semibold border border-gray-200 bg-white text-gray-600 hover:bg-gray-50 hover:text-gray-900 hover:border-gray-300 transition-all cursor-pointer shadow-sm"
                @click="emit('edit')"
              >
                <Edit2 :size="14" />
                <span>Edit</span>
              </button>
              <button class="w-9 h-9 flex items-center justify-center rounded-xl cursor-pointer hover:bg-gray-100 text-gray-400 hover:text-gray-700 transition-colors" @click="emit('close')">
                <X :size="18" />
              </button>
            </div>
          </div>

          <!-- Body: Full-width panel -->
          <div class="flex-1 overflow-hidden flex flex-col min-h-0">
            <!-- Content preview -->
            <div class="flex-1 flex flex-col gap-5 overflow-y-auto p-8">
              <!-- Template Content box (Sleek code editor style) -->
              <div class="flex-1 relative rounded-xl overflow-hidden border border-gray-200 bg-zinc-950 flex flex-col min-h-[200px]">
                <!-- Banner for Diff Preview -->
                <div v-if="selectedVersion" class="flex items-center justify-between bg-zinc-900 border-b border-zinc-800/80 px-5 py-3 flex-shrink-0 z-10">
                  <div class="flex items-center gap-2 min-w-0">
                    <span class="relative flex h-2 w-2">
                      <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-indigo-400 opacity-75"></span>
                      <span class="relative inline-flex rounded-full h-2 w-2 bg-indigo-500"></span>
                    </span>
                    <span class="text-xs font-semibold text-zinc-300 truncate">
                      Comparing v{{ selectedVersion.version_number }} with Current (v{{ promptMeta.current_version }})
                    </span>
                  </div>
                  <div class="flex items-center gap-2 flex-shrink-0">
                    <!-- Copy Button inside Banner to avoid overlap -->
                    <button
                      class="flex items-center gap-1.5 text-[11px] font-bold text-zinc-300 hover:text-zinc-100 bg-zinc-800 hover:bg-zinc-700 px-2.5 py-1.5 rounded-lg border border-zinc-700 hover:border-zinc-600 transition-all cursor-pointer shadow-sm"
                      @click="copyContent"
                    >
                      <Check v-if="copied" :size="11" class="text-emerald-400" />
                      <Copy v-else :size="11" />
                      <span>{{ copied ? 'Copied!' : `Copy v${selectedVersion.version_number}` }}</span>
                    </button>
                    <!-- Restore Button -->
                    <button
                      class="flex items-center gap-1 text-[11px] font-bold text-indigo-400 hover:text-indigo-300 bg-indigo-500/10 hover:bg-indigo-500/20 px-2.5 py-1.5 rounded-lg border border-indigo-500/20 hover:border-indigo-500/30 transition-all cursor-pointer shadow-sm"
                      @click="restoreVersion(selectedVersion.version_number)"
                    >
                      <RotateCcw :size="11" />
                      <span>Restore v{{ selectedVersion.version_number }}</span>
                    </button>
                    <!-- Exit Preview Button -->
                    <button
                      class="text-[11px] font-bold text-zinc-400 hover:text-zinc-200 bg-zinc-800 hover:bg-zinc-700 px-2.5 py-1.5 rounded-lg border border-zinc-700 hover:border-zinc-600 transition-all cursor-pointer shadow-sm"
                      @click="selectedVersionId = null"
                    >
                      Exit Preview
                    </button>
                  </div>
                </div>

                <!-- Floating Label (if no version selected) -->
                <span v-else class="absolute top-3.5 left-5 text-[10px] font-bold text-zinc-500 uppercase tracking-widest pointer-events-none select-none">Template Content</span>

                <!-- Floating Copy Button (Rendered only when not previewing a past version) -->
                <button
                  v-if="!selectedVersion"
                  class="absolute top-2.5 right-3 z-10 flex items-center gap-1.5 text-xs font-semibold cursor-pointer text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900 bg-transparent border border-zinc-800 hover:border-zinc-700 px-2.5 py-1.5 rounded-lg transition-all"
                  @click="copyContent"
                >
                  <Check v-if="copied" :size="13" class="text-emerald-500" />
                  <Copy v-else :size="13" />
                  <span>{{ copied ? 'Copied!' : 'Copy' }}</span>
                </button>

                <!-- Plain content preview if no version selected, or diff view if version selected -->
                <pre v-if="!selectedVersion" class="flex-1 px-6 pt-12 pb-6 overflow-auto font-mono text-[13px] whitespace-pre-wrap leading-relaxed text-zinc-300" v-html="formattedContent"></pre>
                
                <div v-else class="flex-1 px-6 pt-14 pb-6 overflow-auto font-mono text-[13px] whitespace-pre-wrap leading-relaxed text-zinc-300 flex flex-col gap-0.5">
                  <div 
                    v-for="(line, idx) in getWordLevelDiff(promptMeta.content, selectedVersion.content)" 
                    :key="idx"
                    :class="[
                      'px-2 py-0.5 rounded border-l-2 font-mono text-[13px] leading-relaxed flex items-start gap-1 min-h-[22px]',
                      line.type === 'added' ? 'bg-emerald-950/20 text-emerald-400 border-emerald-500/70' :
                      line.type === 'removed' ? 'bg-rose-950/20 text-rose-400 border-rose-500/70 line-through' :
                      'text-zinc-400 border-transparent pl-5'
                    ]"
                  >
                    <span v-if="line.type !== 'unchanged'" class="select-none text-zinc-650 font-bold inline-block w-2.5 text-left flex-shrink-0">{{ line.type === 'added' ? '+' : '-' }}</span>
                    <span class="flex-1" v-html="line.html"></span>
                  </div>
                </div>
              </div>

              <!-- Used In Workflows Section -->
              <div v-if="linkedWorkflows.length > 0" class="flex flex-col gap-2.5 mt-2 text-left">
                <h4 class="text-xs font-bold text-gray-400 uppercase tracking-widest dark:text-zinc-500">Used In Workflows</h4>
                <div class="flex flex-wrap gap-2">
                  <router-link
                    v-for="wf in linkedWorkflows"
                    :key="wf.id"
                    :to="`/workflows/${wf.id}`"
                    class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl border border-gray-200/80 bg-gray-50 hover:bg-gray-105 text-xs font-medium text-gray-750 hover:text-gray-900 transition-all dark:bg-zinc-900 dark:border-zinc-850 dark:text-zinc-400 dark:hover:bg-zinc-800 dark:hover:text-zinc-200 shadow-sm"
                  >
                    <GitBranch :size="12" class="text-indigo-500 dark:text-indigo-400" />
                    <span>{{ wf.title }}</span>
                  </router-link>
                </div>
              </div>
            </div>

            <!-- Bottom Action & Version Track Bar (Unified) -->
            <div class="px-8 py-4 bg-gray-50/50 border-t border-gray-100 flex items-center justify-between gap-4 flex-shrink-0">
              <div v-if="versions.length > 0" class="flex items-center gap-3 min-w-0 flex-1">
                <span class="text-xs font-bold text-gray-500 flex items-center gap-1.5 flex-shrink-0">
                  <History :size="12" />
                  <span>Version History:</span>
                </span>
                <div class="flex items-center gap-2 overflow-x-auto py-1 pr-4 min-w-0">
                  <button
                    v-for="v in versions"
                    :key="v.id"
                    class="flex items-center gap-2 px-3 py-1.5 rounded-lg border text-xs font-semibold cursor-pointer transition-all flex-shrink-0 shadow-sm bg-white"
                    :class="[
                      selectedVersionId === v.id
                        ? 'border-indigo-500 bg-indigo-50/40 text-indigo-700 dark:bg-indigo-950/20 dark:text-indigo-300 ring-1 ring-indigo-100/50'
                        : v.version_number === promptMeta.current_version
                          ? 'border-indigo-150 bg-indigo-50/10 text-indigo-650 hover:bg-indigo-50/20'
                          : 'border-gray-200 bg-white text-gray-600 hover:border-gray-300 hover:bg-gray-50'
                    ]"
                    @click="toggleVersionPreview(v.id)"
                  >
                    <span>v{{ v.version_number }}</span>
                    <span v-if="v.version_number === promptMeta.current_version" class="text-[9px] font-bold uppercase tracking-wide opacity-80">(Current)</span>
                    <span class="text-[10px] text-gray-400 font-normal">{{ formatDate(v.created_at) }}</span>
                  </button>
                </div>
              </div>
              <div v-else class="flex-1" />

              <button
                class="flex items-center justify-center gap-2 h-10 px-6 bg-gray-900 hover:bg-indigo-600 text-white text-[13px] font-bold rounded-xl transition-all cursor-pointer shadow-sm flex-shrink-0"
                @click="handleUsePrompt"
              >
                <Play :size="13" class="fill-white" />
                <span>{{ selectedVersion ? `Use Version ${selectedVersion.version_number}` : 'Use Prompt' }}</span>
              </button>
            </div>
          </div>
        </template>
      </div>
    </div>

    <!-- Custom Delete Confirmation Modal -->
    <Transition name="fade">
      <div
        v-if="showDeleteConfirm"
        class="fixed inset-0 z-[60] flex items-center justify-center p-6 bg-black/60 backdrop-blur-sm"
        @click="showDeleteConfirm = false"
      >
        <div
          class="relative bg-white rounded-2xl shadow-2xl border border-gray-200/80 w-full max-w-md p-6 flex flex-col gap-4 animate-scale-in dark:bg-zinc-950 dark:border-zinc-850"
          @click.stop
        >
          <div class="flex items-start gap-4">
            <div class="w-12 h-12 rounded-full bg-red-50 flex items-center justify-center text-red-650 flex-shrink-0 dark:bg-red-950/30 dark:text-red-400">
              <AlertTriangle :size="22" />
            </div>
            <div class="flex-1 text-left">
              <h3 class="font-bold text-gray-900 text-lg leading-tight dark:text-gray-100">
                Delete Prompt Template
              </h3>
              
              <div v-if="linkedWorkflows.length > 0" class="mt-3 p-3 bg-amber-50 border border-amber-250 rounded-xl dark:bg-amber-955/20 dark:border-amber-900/50 flex flex-col gap-1.5">
                <span class="text-xs font-bold text-amber-750 dark:text-amber-400">
                  ⚠️ Used In Workflows Warning:
                </span>
                <p class="text-[11px] text-amber-800 dark:text-amber-350 leading-normal">
                  This prompt is currently linked to <strong class="font-bold text-gray-900 dark:text-gray-200">{{ linkedWorkflows.length }} workflow{{ linkedWorkflows.length > 1 ? 's' : '' }}</strong>. Deleting it will leave these workflows with missing steps:
                </p>
                <div class="flex flex-col gap-0.5 mt-1 pl-1 text-[11.5px] text-amber-750 dark:text-amber-400 font-semibold">
                  <span v-for="wf in linkedWorkflows" :key="wf.id">
                    • {{ wf.title }}
                  </span>
                </div>
              </div>

              <p class="text-sm text-gray-500 mt-3 leading-relaxed dark:text-zinc-400">
                Are you sure you want to delete <strong class="text-gray-800 dark:text-gray-200">"{{ promptMeta?.title }}"</strong>? This action will permanently remove it and cannot be undone.
              </p>
            </div>
          </div>

          <div class="flex items-center justify-end gap-3 mt-2">
            <button
              class="h-10 px-4 rounded-xl text-sm font-semibold border border-gray-200 bg-white text-gray-600 hover:bg-gray-50 hover:text-gray-900 transition-all cursor-pointer dark:bg-zinc-900 dark:border-zinc-850 dark:text-zinc-400 dark:hover:bg-zinc-800"
              :disabled="deleting"
              @click="showDeleteConfirm = false"
            >
              Cancel
            </button>
            <button
              class="flex items-center justify-center gap-2 h-10 px-5 rounded-xl bg-red-650 hover:bg-red-700 text-white text-sm font-bold transition-all cursor-pointer disabled:opacity-40 shadow-sm dark:bg-red-650 dark:hover:bg-red-600"
              :disabled="deleting"
              @click="handleDeletePrompt"
            >
              <Trash2 v-if="!deleting" :size="14" />
              <div v-else class="w-3.5 h-3.5 rounded-full border-2 border-white/30 border-t-white animate-spin" />
              <span>{{ deleting ? 'Deleting...' : 'Delete Prompt' }}</span>
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
@keyframes scale-in {
  from { opacity: 0; transform: scale(0.97) translateY(6px); }
  to   { opacity: 1; transform: scale(1) translateY(0); }
}
.animate-scale-in {
  animation: scale-in 0.18s cubic-bezier(0.16, 1, 0.3, 1);
}
.expand-enter-active, .expand-leave-active {
  transition: all 0.2s ease;
  overflow: hidden;
}
.expand-enter-from, .expand-leave-to {
  opacity: 0;
  max-height: 0;
}
.expand-enter-to, .expand-leave-from {
  max-height: 200px;
}
@keyframes pulse-subtle {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.85; }
}
.animate-pulse-subtle {
  animation: pulse-subtle 3s infinite ease-in-out;
}
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}
</style>
