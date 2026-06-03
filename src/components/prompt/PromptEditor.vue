<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { X, Sparkles, Check, Info } from 'lucide-vue-next'
import { usePromptsStore } from '@/stores/prompts'
import { useFoldersStore } from '@/stores/folders'
import * as api from '@/lib/tauri-api'
import InputDialog from '@/components/common/InputDialog.vue'

// Import shadcn-vue components
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'

const props = defineProps<{
  promptId?: string | null
  defaultFolderId?: string | null
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const varHint = '{{var}}'

const store = usePromptsStore()
const foldersStore = useFoldersStore()

const title = ref('')
const content = ref('')
const folderId = ref<string | null>(props.defaultFolderId || null)
const tagsList = ref<string[]>([])
const tagInputVal = ref('')
const tagInputRef = ref<HTMLInputElement | null>(null)
const loading = ref(false)
const showFolderDialog = ref(false)

// AI suggestions state
const aiLoading = ref(false)
const aiTags = ref<string[]>([])

// Scan content for variables
const detectedVariables = computed(() => {
  const c = content.value || ''
  const regex = /\{\{([^}]+)\}\}/g
  const matches: string[] = []
  let match
  while ((match = regex.exec(c)) !== null) {
    const varName = match[1].trim()
    if (varName && !matches.includes(varName)) matches.push(varName)
  }
  return matches
})

onMounted(async () => {
  await foldersStore.fetchFolders()
  if (props.promptId) {
    loading.value = true
    try {
      const pm = await api.getPrompt(props.promptId)
      title.value = pm.title
      content.value = pm.content
      folderId.value = pm.folder_id
      tagsList.value = pm.tags
    } catch (e) {
      console.error(e)
    } finally {
      loading.value = false
    }
  } else if (props.defaultFolderId) {
    folderId.value = props.defaultFolderId
  }
})

function handleTagInput(event: KeyboardEvent) {
  if (event.key === ',' || event.key === 'Enter') {
    event.preventDefault()
    addTag()
  }
}

function handleTagBlur() { addTag() }

// Select mapping helper to handle NO_FOLDER and NEW_FOLDER options safely
const selectedFolderVal = computed({
  get() {
    return folderId.value || 'NO_FOLDER'
  },
  set(val) {
    if (val === 'NO_FOLDER') {
      folderId.value = null
    } else if (val === 'NEW_FOLDER') {
      showFolderDialog.value = true
    } else {
      folderId.value = val
    }
  }
})

function addTag() {
  const val = tagInputVal.value.trim().replace(/,/g, '')
  if (val && !tagsList.value.includes(val)) tagsList.value.push(val)
  tagInputVal.value = ''
}

function removeTag(index: number) { tagsList.value.splice(index, 1) }

function focusTagInput() { tagInputRef.value?.focus() }

async function confirmNewFolder(newName: string) {
  try {
    const newFolder = await foldersStore.createFolder(newName, null)
    if (newFolder) folderId.value = newFolder.id
  } catch (err) {
    console.error('Failed to create folder:', err)
    folderId.value = null
  }
}

function cancelNewFolder() { folderId.value = null }

async function savePrompt() {
  if (!title.value.trim() || !content.value.trim()) return
  addTag()
  loading.value = true
  const tags = [...tagsList.value]
  try {
    if (props.promptId) {
      await store.updatePrompt({
        id: props.promptId,
        title: title.value,
        content: content.value,
        description: '',
        folder_id: folderId.value,
        change_summary: 'Edited from editor UI',
      })
    } else {
      await store.createPrompt({
        title: title.value,
        content: content.value,
        description: '',
        folder_id: folderId.value,
        tags,
        category: null,
      })
    }
    emit('close')
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
  }
}

async function runAiAnalysis() {
  if (!content.value.trim()) return
  aiLoading.value = true
  try {
    const suggestions = await api.suggestIntent(content.value)
    aiTags.value = suggestions.tags
  } catch (e) {
    console.error('AI Suggestion failed:', e)
  } finally {
    aiLoading.value = false
  }
}

function applyAiTag(tag: string) {
  if (!tagsList.value.includes(tag)) tagsList.value.push(tag)
}
</script>

<template>
  <Dialog :open="true" @update:open="emit('close')">
    <DialogContent class="sm:max-w-7xl h-[88vh] max-h-[900px] flex flex-col p-0 gap-0 overflow-hidden bg-white border border-gray-200/80 rounded-2xl shadow-2xl outline-none dark:bg-zinc-950 dark:border-zinc-850" :showCloseButton="false">
      <!-- Header -->
      <DialogHeader class="flex flex-row items-center justify-between px-8 py-5 border-b border-gray-100 flex-shrink-0 bg-white dark:bg-zinc-950 dark:border-zinc-850">
        <div class="flex items-center gap-3.5">
          <div class="w-10 h-10 rounded-xl bg-gray-900 flex items-center justify-center text-white flex-shrink-0 dark:bg-zinc-100 dark:text-zinc-950">
            <Sparkles :size="17" />
          </div>
          <div class="text-left">
            <DialogTitle class="font-bold text-gray-900 text-[17px] leading-tight dark:text-gray-100">
              {{ promptId ? 'Edit Prompt' : 'Create New Prompt' }}
            </DialogTitle>
            <DialogDescription class="text-[12px] text-gray-400 mt-0.5 dark:text-zinc-500">
              {{ promptId ? 'Update your reusable template' : 'Add a reusable template to your library' }}
            </DialogDescription>
          </div>
        </div>
        <Button variant="ghost" size="icon-sm" class="w-9 h-9 flex items-center justify-center rounded-xl cursor-pointer hover:bg-gray-100 text-gray-400 hover:text-gray-700 transition-colors dark:hover:bg-zinc-900 dark:hover:text-zinc-200" @click="emit('close')">
          <X :size="18" />
        </Button>
      </DialogHeader>

      <!-- Body: Two-column layout -->
      <div class="flex-1 overflow-hidden flex min-h-0">
        <!-- Left col: title + content textarea -->
        <div class="flex-1 flex flex-col gap-5 p-8 overflow-y-auto border-r border-gray-100 dark:border-zinc-850">
          <!-- Title -->
          <div class="flex flex-col gap-2">
            <Label for="pe-title" class="text-[11px] font-bold text-gray-400 uppercase tracking-widest dark:text-zinc-500">Prompt Title</Label>
            <Input
              id="pe-title"
              v-model="title"
              type="text"
              placeholder="E.g., Senior Rust Engineer System Prompt"
              class="w-full h-11 px-4 py-3 rounded-xl text-[14px] bg-white border border-gray-200 text-gray-900 placeholder-gray-400 outline-none focus-visible:ring-indigo-500 dark:bg-zinc-900 dark:border-zinc-800 dark:text-gray-100"
            />
          </div>

          <!-- Content -->
          <div class="flex-1 flex flex-col gap-2 min-h-0">
            <div class="flex items-center justify-between">
              <Label for="pe-content" class="text-[11px] font-bold text-gray-400 uppercase tracking-widest dark:text-zinc-500">Template Content</Label>
              <span class="text-[11px] text-gray-400 dark:text-zinc-555">Use <code class="bg-indigo-50 px-1.5 py-0.5 rounded-md text-indigo-500 font-mono dark:bg-indigo-950/40 dark:text-indigo-400">{{ varHint }}</code> for variables</span>
            </div>
            <div class="relative flex-1 flex flex-col min-h-[200px]">
              <Textarea
                id="pe-content"
                v-model="content"
                placeholder="Write or paste your template here. Use {{variable_name}} to define prompt variables..."
                class="flex-1 w-full px-5 py-4 rounded-xl text-[13px] leading-relaxed bg-gray-50 border border-gray-200 text-gray-900 placeholder-gray-400 outline-none resize-none font-mono focus-visible:ring-indigo-500 pb-14 dark:bg-zinc-900 dark:border-zinc-800 dark:text-gray-100"
              />
              <Button
                variant="outline"
                class="absolute bottom-3.5 right-3.5 flex items-center gap-1.5 text-xs px-3 py-1.5 h-8 rounded-lg transition-all cursor-pointer bg-white hover:bg-indigo-50 border border-gray-200 hover:border-indigo-200 text-gray-500 hover:text-indigo-600 font-semibold shadow-sm disabled:opacity-40 dark:bg-zinc-950 dark:border-zinc-800 dark:text-zinc-400 dark:hover:bg-indigo-950/30"
                :disabled="!content.trim() || aiLoading"
                @click="runAiAnalysis"
              >
                <Sparkles :size="12" class="text-indigo-400" />
                <span>{{ aiLoading ? 'Analyzing…' : 'AI Analyze' }}</span>
              </Button>
            </div>
          </div>

          <!-- Detected Variables -->
          <div v-if="detectedVariables.length > 0" class="p-4 rounded-xl bg-indigo-50 border border-indigo-100 flex flex-col gap-2 dark:bg-indigo-950/20 dark:border-indigo-900/50">
            <div class="flex items-center gap-1.5 text-[11px] font-bold text-indigo-600 dark:text-indigo-400">
              <Info :size="13" />
              <span>Detected Variables</span>
            </div>
            <div class="flex flex-wrap gap-1.5">
              <span
                v-for="v in detectedVariables"
                :key="v"
                class="text-[12px] font-mono font-bold px-2.5 py-1 rounded-lg bg-white border border-indigo-100 text-indigo-600 dark:bg-zinc-900 dark:border-indigo-900 dark:text-indigo-400"
              >
                {{ v }}
              </span>
            </div>
          </div>
        </div>

        <!-- Right col: settings panel -->
        <div class="w-64 flex-shrink-0 flex flex-col bg-gray-50/50 border-l border-gray-100 dark:bg-zinc-950 dark:border-zinc-850">
          <!-- Settings header -->
          <div class="px-6 py-5 border-b border-gray-100 dark:border-zinc-850">
            <h3 class="text-[11px] font-bold uppercase tracking-widest text-gray-400 dark:text-zinc-500">Settings</h3>
          </div>

          <div class="flex-1 overflow-y-auto px-6 py-5 flex flex-col gap-6">
            <!-- Folder -->
            <div class="flex flex-col gap-2">
              <Label for="pe-folder" class="text-[12px] font-semibold text-gray-600 dark:text-zinc-400">Folder</Label>
              <Select v-model="selectedFolderVal">
                <SelectTrigger id="pe-folder" class="w-full h-10 px-3.5 py-2.5 rounded-xl text-[13px] bg-white border border-gray-200 text-gray-900 dark:bg-zinc-900 dark:border-zinc-800 dark:text-gray-100 justify-between flex">
                  <SelectValue placeholder="No Folder" />
                </SelectTrigger>
                <SelectContent class="dark:bg-zinc-900 dark:border-zinc-800">
                  <SelectGroup>
                    <SelectItem value="NO_FOLDER">No Folder</SelectItem>
                    <SelectItem v-for="f in foldersStore.folders" :key="f.id" :value="f.id">{{ f.name }}</SelectItem>
                    <SelectItem value="NEW_FOLDER" class="font-bold text-indigo-600 dark:text-indigo-400">+ Create New Folder…</SelectItem>
                  </SelectGroup>
                </SelectContent>
              </Select>
            </div>

            <!-- Tags -->
            <div class="flex flex-col gap-2">
              <Label for="pe-tags" class="text-[12px] font-semibold text-gray-600 dark:text-zinc-400">Tags</Label>
              <div
                class="w-full flex flex-wrap gap-1.5 px-3.5 py-2.5 rounded-xl bg-white border border-gray-200 focus-within:border-indigo-450 focus-within:ring-2 focus-within:ring-indigo-100 transition-all cursor-text min-h-[44px] items-start shadow-sm dark:bg-zinc-900 dark:border-zinc-800 dark:focus-within:border-indigo-500"
                @click="focusTagInput"
              >
                <Badge
                  v-for="(tag, index) in tagsList"
                  :key="tag"
                  variant="secondary"
                  class="flex items-center gap-1 text-[11px] font-semibold px-2 py-0.5 rounded-full bg-violet-50 border border-violet-100 text-violet-750 dark:bg-violet-950/30 dark:border-violet-900/50 dark:text-violet-400"
                >
                  <span>{{ tag }}</span>
                  <button type="button" class="w-3.5 h-3.5 flex items-center justify-center rounded-full hover:bg-violet-200 dark:hover:bg-violet-900 cursor-pointer" @click.stop="removeTag(index)">
                    <X :size="9" />
                  </button>
                </Badge>
                <input
                  id="pe-tags"
                  ref="tagInputRef"
                  v-model="tagInputVal"
                  type="text"
                  placeholder="Add tags…"
                  class="flex-1 min-w-[60px] bg-transparent text-[13px] text-gray-900 dark:text-gray-100 outline-none border-none focus:ring-0 placeholder:text-muted-foreground p-0 m-0"
                  @keydown="handleTagInput"
                  @blur="handleTagBlur"
                />
              </div>
            </div>

            <!-- AI Suggestions -->
            <div v-if="aiTags.length > 0" class="p-4 rounded-xl flex flex-col gap-2.5 bg-indigo-50 border border-indigo-100 dark:bg-indigo-950/20 dark:border-indigo-905/30">
              <div class="flex items-center gap-1.5 text-[11px] font-bold text-indigo-600 dark:text-indigo-450">
                <Sparkles :size="13" />
                <span>AI Suggested Tags</span>
              </div>
              <div class="flex flex-wrap gap-1.5">
                <Button
                  v-for="tag in aiTags"
                  :key="tag"
                  variant="outline"
                  class="text-[11px] h-7 flex items-center gap-1 px-2.5 py-1 rounded-full border border-indigo-100 bg-white text-indigo-600 hover:bg-indigo-50 cursor-pointer transition-all font-medium dark:bg-zinc-900 dark:border-indigo-900/50 dark:text-indigo-400 dark:hover:bg-indigo-950/30"
                  @click="applyAiTag(tag)"
                >
                  <span>+</span>
                  <span>{{ tag }}</span>
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <DialogFooter class="px-8 py-4 border-t border-gray-100 bg-white flex items-center justify-end gap-3 flex-shrink-0 dark:bg-zinc-950 dark:border-zinc-850">
        <Button
          variant="ghost"
          class="h-10 px-5 rounded-xl text-[13px] font-semibold hover:bg-gray-100 text-gray-500 hover:text-gray-900 transition-all cursor-pointer dark:hover:bg-zinc-900 dark:hover:text-zinc-200"
          @click="emit('close')"
        >
          Cancel
        </Button>
        <Button
          id="pe-save"
          class="flex items-center justify-center gap-2 h-10 px-6 rounded-xl bg-gray-900 hover:bg-indigo-600 text-white text-[13px] font-bold transition-colors cursor-pointer disabled:opacity-40 shadow-sm dark:bg-zinc-100 dark:text-zinc-950 dark:hover:bg-indigo-500 dark:hover:text-white"
          :disabled="!title.trim() || !content.trim() || loading"
          @click="savePrompt"
        >
          <Check :size="15" />
          <span>{{ loading ? 'Saving…' : (promptId ? 'Update Prompt' : 'Create Prompt') }}</span>
        </Button>
      </DialogFooter>

      <!-- Create Folder Dialog -->
      <InputDialog
        v-model:open="showFolderDialog"
        title="Create New Folder"
        placeholder="e.g. Project Specs"
        confirmText="Create Folder"
        @confirm="confirmNewFolder"
        @cancel="cancelNewFolder"
      />
    </DialogContent>
  </Dialog>
</template>
