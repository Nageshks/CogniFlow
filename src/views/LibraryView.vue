<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from 'vue'
import { Plus, Star, Archive, LayoutGrid, List as ListIcon, Folder as FolderIcon, ChevronDown } from 'lucide-vue-next'
import { usePromptsStore } from '@/stores/prompts'
import { useWorkflowsStore } from '@/stores/workflows'
import { useFoldersStore } from '@/stores/folders'
import PromptCard from '@/components/prompt/PromptCard.vue'
import PromptEditor from '@/components/prompt/PromptEditor.vue'
import PromptDetailSheet from '@/components/prompt/PromptDetailSheet.vue'
import UsePromptDialog from '@/components/prompt/UsePromptDialog.vue'
import InputDialog from '@/components/common/InputDialog.vue'
import WorkflowCard from '@/components/workflow/WorkflowCard.vue'
import { toast } from 'vue-sonner'
import { useRouter } from 'vue-router'

// Import shadcn-vue components
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
} from '@/components/ui/dropdown-menu'

const promptsStore = usePromptsStore()
const workflowsStore = useWorkflowsStore()
const foldersStore = useFoldersStore()
const router = useRouter()

const showEditor = ref(false)
const editingPromptId = ref<string | null>(null)
const selectedPromptId = ref<string | null>(null)
const showDetail = ref(false)

const usePromptOpen = ref(false)
const usePromptData = ref<any | null>(null)

const duplicateDialogOpen = ref(false)
const duplicateTarget = ref<any | null>(null)
const duplicateInitialName = ref('')

const createFolderDialogOpen = ref(false)

async function handleCreateFolderConfirm(name: string) {
  if (!name.trim()) return
  try {
    await foldersStore.createFolder(name.trim(), null)
    await foldersStore.fetchFolders()
  } catch (e) {
    toast.error('Failed to create folder: ' + String(e))
  }
}

// Filter states
const assetType = ref('all') // 'all' | 'prompts' | 'workflows'
const showStarred = ref(false)
const sortBy = ref('Recently Used') // 'Recently Used' | 'Mostly Used' | 'Date Created' | 'Date Modified'
const usageUpdateTracker = ref(0)
const viewMode = ref('grid') // 'grid' | 'list'
const selectedFolder = ref<string | null>(null)

// Combined computation
const filteredAssets = computed(() => {
  // Trigger computed update when prompts are used
  const _ = usageUpdateTracker.value

  const prompts = promptsStore.prompts.map(p => ({ ...p, asset_type: 'prompt' as const }))
  const workflows = workflowsStore.workflows.map(w => ({ ...w, asset_type: 'workflow' as const }))
  
  let assets: any[] = []
  if (assetType.value === 'all') assets = [...prompts, ...workflows]
  else if (assetType.value === 'prompts') assets = [...prompts]
  else if (assetType.value === 'workflows') assets = [...workflows]

  if (selectedFolder.value) {
    assets = assets.filter(a => a.folder_id === selectedFolder.value)
  }

  if (showStarred.value) {
    assets = assets.filter(a => localStorage.getItem(`starred-${a.id}`) === 'true')
  }

  assets.sort((a, b) => {
    if (sortBy.value === 'Date Created') {
      return new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
    }
    if (sortBy.value === 'Mostly Used') {
      const aUsage = a.asset_type === 'prompt' ? parseInt(localStorage.getItem(`usage-${a.id}`) || '0') : 0
      const bUsage = b.asset_type === 'prompt' ? parseInt(localStorage.getItem(`usage-${b.id}`) || '0') : 0
      return bUsage - aUsage
    }
    return new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
  })

  return assets
})

const totalAssetCount = computed(() => filteredAssets.value.length)

onMounted(() => {
  promptsStore.fetchPrompts()
  workflowsStore.fetchWorkflows()
  foldersStore.fetchFolders()
  window.addEventListener('open-new-prompt', openNewPrompt)
  window.addEventListener('open-prompt-detail', handleOpenPromptDetail as EventListener)
  window.addEventListener('prompt-used', handlePromptUsed)
})

onUnmounted(() => {
  window.removeEventListener('open-new-prompt', openNewPrompt)
  window.removeEventListener('open-prompt-detail', handleOpenPromptDetail as EventListener)
  window.removeEventListener('prompt-used', handlePromptUsed)
})

function handlePromptUsed() {
  usageUpdateTracker.value++
}

function handleOpenPromptDetail(e: CustomEvent) {
  if (e.detail?.promptId) openPromptDetail(e.detail.promptId)
}

function openNewPrompt() {
  editingPromptId.value = null
  showEditor.value = true
  showDetail.value = false
}

function openPromptDetail(promptId: string) {
  selectedPromptId.value = promptId
  showDetail.value = true
  showEditor.value = false
}

function openPromptEditor(promptId: string) {
  editingPromptId.value = promptId
  showEditor.value = true
  showDetail.value = false
}

// Ensure editor reload propagates properly
function closeEditor() {
  showEditor.value = false
  editingPromptId.value = null
  promptsStore.fetchPrompts()
}

function closeDetail() {
  showDetail.value = false
  selectedPromptId.value = null
  promptsStore.fetchPrompts()
}

function openUsePrompt(prompt: any) {
  usePromptData.value = prompt
  usePromptOpen.value = true
}

function closeUsePrompt() {
  usePromptOpen.value = false
  usePromptData.value = null
}

async function duplicatePrompt(promptData: any) {
  duplicateTarget.value = promptData
  duplicateInitialName.value = `${promptData.title} (Copy)`
  duplicateDialogOpen.value = true
}

async function handleDuplicateConfirm(newName: string) {
  const promptData = duplicateTarget.value
  if (!promptData || !newName.trim()) return

  try {
    await promptsStore.createPrompt({
      title: newName.trim(),
      content: promptData.content,
      description: '',
      folder_id: promptData.folder_id,
      tags: [...promptData.tags],
      category: null,
    })
    toast.success(`Successfully duplicated as "${newName.trim()}"`)
    promptsStore.fetchPrompts()
  } catch (e) {
    toast.error('Failed to duplicate prompt: ' + String(e))
  } finally {
    duplicateTarget.value = null
  }
}

const createWorkflowDialogOpen = ref(false)

function triggerCreateWorkflow() {
  createWorkflowDialogOpen.value = true
}

async function handleCreateWorkflowConfirm(title: string) {
  if (!title.trim()) return
  try {
    const wf = await workflowsStore.createWorkflow({
      title: title.trim(),
      folder_id: selectedFolder.value || null,
    })
    if (wf) {
      toast.success(`Workflow "${title.trim()}" created successfully`)
      workflowsStore.fetchWorkflows()
      router.push(`/workflows/${wf.id}`)
    }
  } catch (e) {
    toast.error('Failed to create workflow: ' + String(e))
  }
}
</script>

<template>
  <div class="library-view animate-fade-in flex flex-col gap-5 w-full py-8 px-8 max-w-[1400px] mx-auto">
    
    <!-- Top Header Title -->
    <div class="flex items-center justify-between gap-3 w-full">
      <div class="flex items-center gap-3">
        <h1 class="text-[28px] font-bold tracking-tight text-gray-900 dark:text-gray-100">Library</h1>
        <Badge variant="outline" class="px-2.5 py-0.5 rounded-lg text-[11px] font-bold bg-blue-50 text-blue-600 border border-blue-100 mt-0.5 dark:bg-blue-950/30 dark:text-blue-400 dark:border-blue-900/50 h-5">
          {{ totalAssetCount }} Assets
        </Badge>
      </div>

      <!-- Actions: Add Prompt and Add Workflow -->
      <div class="flex items-center gap-3">
        <Button
          id="lib-add-prompt"
          class="flex items-center gap-1.5 px-4 h-9 rounded-xl bg-gray-900 text-white font-semibold transition-all cursor-pointer text-xs hover:bg-gray-800 dark:bg-zinc-100 dark:text-zinc-950 dark:hover:bg-zinc-200"
          @click="openNewPrompt"
        >
          <Plus :size="13" />
          <span>Add Prompt</span>
        </Button>
        <Button
          id="lib-add-workflow"
          class="flex items-center gap-1.5 px-4 h-9 rounded-xl border border-gray-200 bg-white text-gray-900 font-semibold transition-all cursor-pointer text-xs hover:bg-gray-50 dark:border-zinc-850 dark:bg-zinc-950 dark:text-zinc-100 dark:hover:bg-zinc-900"
          @click="triggerCreateWorkflow"
        >
          <Plus :size="13" />
          <span>Add Workflow</span>
        </Button>
      </div>
    </div>

    <!-- Filter Bar -->
    <div class="flex flex-col gap-5">
      <!-- Main Filter Row -->
      <div class="bg-white border border-gray-200 rounded-[20px] px-6 py-4 flex items-center justify-between shadow-sm dark:bg-zinc-950 dark:border-zinc-800">
        <!-- Left side: Type tabs + Star -->
        <div class="flex items-center gap-5">
          <!-- Segmented control container -->
          <div class="flex items-center bg-gray-100/80 p-1.5 rounded-xl gap-1 dark:bg-zinc-900">
            <button 
              class="px-5 py-1.5 text-[14px] font-semibold rounded-lg transition-all cursor-pointer dark:text-zinc-400"
              :class="assetType === 'all' ? 'bg-white text-gray-900 shadow-sm dark:bg-zinc-800 dark:text-zinc-100' : 'text-gray-500 hover:text-gray-700 hover:bg-gray-200/50 dark:hover:bg-zinc-800/50'"
              @click="assetType = 'all'"
            >All</button>
            <button 
              class="px-5 py-1.5 text-[14px] font-semibold rounded-lg transition-all cursor-pointer dark:text-zinc-400"
              :class="assetType === 'prompts' ? 'bg-white text-gray-900 shadow-sm dark:bg-zinc-800 dark:text-zinc-100' : 'text-gray-500 hover:text-gray-700 hover:bg-gray-200/50 dark:hover:bg-zinc-800/50'"
              @click="assetType = 'prompts'"
            >Prompts</button>
            <button 
              class="px-5 py-1.5 text-[14px] font-semibold rounded-lg transition-all cursor-pointer dark:text-zinc-400"
              :class="assetType === 'workflows' ? 'bg-white text-gray-900 shadow-sm dark:bg-zinc-800 dark:text-zinc-100' : 'text-gray-500 hover:text-gray-700 hover:bg-gray-200/50 dark:hover:bg-zinc-800/50'"
              @click="assetType = 'workflows'"
            >Workflows</button>
          </div>

          <div class="w-px h-6 bg-gray-200 dark:bg-zinc-800"></div>

          <Button
            variant="ghost"
            size="icon"
            class="h-9 w-9 rounded-xl transition-colors cursor-pointer"
            :class="showStarred ? 'text-yellow-500 bg-yellow-50 hover:bg-yellow-100/50 dark:bg-yellow-950/20' : 'text-gray-400'"
            @click="showStarred = !showStarred"
          >
            <Star :size="20" :class="showStarred ? 'fill-yellow-500' : ''" />
          </Button>
        </div>

        <!-- Right side: Sort + View Mode -->
        <div class="flex items-center gap-4">
          <div class="flex items-center gap-3">
            <span class="text-[12px] font-bold tracking-wider text-gray-400 uppercase">Sort By</span>
            <DropdownMenu>
              <DropdownMenuTrigger as-child>
                <Button variant="outline" class="flex items-center gap-2 h-9 rounded-xl font-medium">
                  <span>{{ sortBy }}</span>
                  <ChevronDown :size="16" class="text-gray-400" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent class="w-48 bg-white border border-gray-200 rounded-xl shadow-lg dark:bg-zinc-900 dark:border-zinc-800">
                <DropdownMenuItem class="text-left px-5 py-2.5 text-[14px] font-medium hover:bg-gray-50 text-gray-700 dark:text-zinc-300 dark:hover:bg-zinc-800 cursor-pointer" @click="sortBy = 'Recently Used'">Recently Used</DropdownMenuItem>
                <DropdownMenuItem class="text-left px-5 py-2.5 text-[14px] font-medium hover:bg-gray-50 text-gray-700 dark:text-zinc-300 dark:hover:bg-zinc-800 cursor-pointer" @click="sortBy = 'Mostly Used'">Mostly Used</DropdownMenuItem>
                <DropdownMenuItem class="text-left px-5 py-2.5 text-[14px] font-medium hover:bg-gray-50 text-gray-700 dark:text-zinc-300 dark:hover:bg-zinc-800 cursor-pointer" @click="sortBy = 'Date Created'">Date Created</DropdownMenuItem>
                <DropdownMenuItem class="text-left px-5 py-2.5 text-[14px] font-medium hover:bg-gray-50 text-gray-700 dark:text-zinc-300 dark:hover:bg-zinc-800 cursor-pointer" @click="sortBy = 'Date Modified'">Date Modified</DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>

          <div class="w-px h-6 bg-gray-200 dark:bg-zinc-800"></div>

          <div class="flex items-center gap-1.5">
            <Button 
              variant="ghost"
              size="icon"
              class="h-9 w-9 rounded-xl transition-all cursor-pointer"
              :class="viewMode === 'grid' ? 'bg-gray-100 text-gray-900 dark:bg-zinc-800 dark:text-zinc-100' : 'text-gray-400'"
              @click="viewMode = 'grid'"
            ><LayoutGrid :size="20" /></Button>
            <Button 
              variant="ghost"
              size="icon"
              class="h-9 w-9 rounded-xl transition-all cursor-pointer"
              :class="viewMode === 'list' ? 'bg-gray-100 text-gray-900 dark:bg-zinc-800 dark:text-zinc-100' : 'text-gray-400'"
              @click="viewMode = 'list'"
            ><ListIcon :size="20" /></Button>
          </div>
        </div>
      </div>

      <!-- Folders Row -->
      <div class="flex items-center justify-between px-2">
        <div class="flex items-center gap-5">
          <span class="text-[12px] font-bold tracking-wider text-gray-500 uppercase dark:text-zinc-550">Folders:</span>
          <div class="flex items-center gap-2.5 flex-wrap">
            <Button
              variant="outline"
              class="h-8 rounded-full transition-all border-0 shadow-none cursor-pointer"
              :class="!selectedFolder ? 'bg-indigo-500 hover:bg-indigo-650 text-white dark:bg-indigo-600 dark:hover:bg-indigo-700' : 'bg-gray-100 text-gray-600 hover:bg-gray-200 dark:bg-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-850'"
              @click="selectedFolder = null"
            >All</Button>
            <Button
              v-for="folder in foldersStore.folders"
              :key="folder.id"
              variant="outline"
              class="h-8 rounded-full transition-all border-0 shadow-none cursor-pointer"
              :class="selectedFolder === folder.id ? 'bg-indigo-500 hover:bg-indigo-650 text-white dark:bg-indigo-600 dark:hover:bg-indigo-700' : 'bg-gray-100 text-gray-600 hover:bg-gray-200 dark:bg-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-850'"
              @click="selectedFolder = folder.id"
            >{{ folder.name }}</Button>
          </div>
        </div>
        <Button 
          variant="ghost" 
          size="icon" 
          class="p-2 rounded-xl text-gray-400 hover:text-gray-900 hover:bg-gray-100 dark:hover:text-zinc-200 dark:hover:bg-zinc-800 transition-colors size-9 cursor-pointer"
          @click="createFolderDialogOpen = true"
          title="Create New Folder"
        >
          <FolderIcon :size="18" />
        </Button>
      </div>
    </div>

    <!-- Asset Grid/List -->
    <div v-if="promptsStore.loading || workflowsStore.loading" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-5 mt-2">
      <div v-for="i in 8" :key="i" class="bg-gray-100 rounded-2xl animate-pulse dark:bg-zinc-900" style="height: 240px;" />
    </div>

    <div v-else-if="filteredAssets.length > 0" class="mt-2 flex flex-col">
      <!-- List View Table Headers -->
      <div v-if="viewMode === 'list'" class="hidden md:flex items-center px-4 py-2.5 text-[11px] font-bold uppercase tracking-wider text-gray-400 border-b border-gray-200/60 mb-2 dark:text-zinc-500 dark:border-zinc-850">
        <span class="flex-1 pl-11">Asset / Details</span>
        <span class="w-28 pl-1">Type</span>
        <span class="w-36">Folder</span>
        <span class="w-44">Tags</span>
        <span class="w-32 text-right pr-6">Stats & Date</span>
        <span class="w-20 text-right pr-2">Actions</span>
      </div>

      <TransitionGroup
        name="v"
        tag="div"
        class="gap-4"
        :class="viewMode === 'grid' ? 'grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 items-stretch' : 'flex flex-col gap-2'"
      >
        <template v-for="asset in filteredAssets" :key="asset.id">
          <!-- Prompt -->
          <PromptCard
            v-if="asset.asset_type === 'prompt'"
            :prompt="asset"
            :compact="viewMode === 'list'"
            @click="openPromptDetail(asset.id)"
            @use="openUsePrompt"
          />
          
          <!-- Workflow -->
          <WorkflowCard
            v-else-if="asset.asset_type === 'workflow'"
            :workflow="asset"
            :compact="viewMode === 'list'"
            @click="router.push(`/workflows/${asset.id}`)"
          />
        </template>

        <!-- Create New Card (Grid Mode Only) -->
        <PromptCard
          v-if="viewMode === 'grid' && !showStarred && (assetType === 'all' || assetType === 'prompts')"
          key="new-prompt-card"
          isNewCard
          @create="openNewPrompt"
        />
      </TransitionGroup>
    </div>

    <!-- Empty State -->
    <div v-else class="mt-2 empty-state border border-gray-200 border-dashed rounded-2xl bg-white p-12 flex flex-col items-center justify-center text-center min-h-[300px] dark:bg-zinc-950 dark:border-zinc-800">
      <Archive :size="40" class="empty-icon text-gray-300 mb-3 dark:text-zinc-700" />
      <div class="empty-title text-sm font-bold text-gray-900 dark:text-zinc-200">
        {{ showStarred ? 'No starred assets yet' : 'No assets found' }}
      </div>
      <div class="empty-description text-xs text-gray-400 max-w-xs mb-5 leading-relaxed mt-1.5 dark:text-zinc-500">
        {{ showStarred ? 'Star any prompt or workflow to bookmark it.' : 'Create your first asset or adjust filters.' }}
      </div>
      <Button
        class="flex items-center gap-1.5 px-5 h-9 bg-gray-900 hover:bg-gray-800 text-white text-sm font-semibold rounded-xl transition-colors cursor-pointer shadow-sm border-0 dark:bg-zinc-100 dark:text-zinc-950 dark:hover:bg-zinc-200"
        @click="openNewPrompt"
      >
        <span>Create New Prompt</span>
      </Button>
    </div>

    <!-- Modals -->
    <PromptEditor v-slot="{ open }" v-if="showEditor" :prompt-id="editingPromptId" :default-folder-id="selectedFolder" @close="closeEditor" />
    <PromptDetailSheet v-if="showDetail && selectedPromptId" :prompt-id="selectedPromptId" @close="closeDetail" @edit="openPromptEditor(selectedPromptId!)" @use="openUsePrompt" @duplicate="duplicatePrompt" />
    <UsePromptDialog :open="usePromptOpen" :prompt="usePromptData" @close="closeUsePrompt" />
    <InputDialog v-model:open="duplicateDialogOpen" title="Duplicate Prompt Template" placeholder="Enter new prompt name" :initial-value="duplicateInitialName" confirmText="Duplicate" @confirm="handleDuplicateConfirm" />
    <InputDialog v-model:open="createFolderDialogOpen" title="Create New Folder" placeholder="Enter folder name" confirmText="Create" @confirm="handleCreateFolderConfirm" />
    <InputDialog v-model:open="createWorkflowDialogOpen" title="Create New Workflow" placeholder="Enter workflow name" confirmText="Create" @confirm="handleCreateWorkflowConfirm" />
  </div>
</template>

<style scoped>
.v-enter-active,
.v-leave-active {
  transition: all 0.3s ease;
}
.v-enter-from,
.v-leave-to {
  opacity: 0;
  transform: translateY(10px);
}
</style>
