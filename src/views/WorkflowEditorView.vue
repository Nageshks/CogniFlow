<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue'
import { useRouter } from 'vue-router'
import {
  ArrowLeft,
  Plus,
  Save,
  Check,
  Play,
  Copy,
  Download,
  Trash2,
  ChevronUp,
  ChevronDown,
  Search,
  FileText,
  Sparkles,
  Link as LinkIcon,
  X,
  Clock,
  FolderOpen,
  AlertTriangle
} from 'lucide-vue-next'
import { useWorkflowsStore } from '@/stores/workflows'
import { usePromptsStore } from '@/stores/prompts'
import { useFoldersStore } from '@/stores/folders'
import { useAutoSave } from '@/composables/useAutoSave'
import { toast } from 'vue-sonner'
import type { WorkflowStep, PromptWithMeta } from '@/types'
import { formatDate } from '@/lib/utils'

// Shadcn UI components
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'

const props = defineProps<{ id: string }>()
const router = useRouter()
const store = useWorkflowsStore()
const promptsStore = usePromptsStore()
const foldersStore = useFoldersStore()

// State
const title = ref('')
const folderId = ref<string | null>(null)

// Step actions state
const showAddStepDialog = ref(false)
const showLinkPromptDialog = ref(false)
const showCreatePromptDialog = ref(false)

// Step creation form
const stepTitleInput = ref('')
const selectedPromptId = ref<string | null>(null)
const inlinePromptTitle = ref('')
const inlinePromptContent = ref('')

// Step replacement state
const replacingStepId = ref<string | null>(null)

// Variable resolution modal
const showUseWorkflowDialog = ref(false)
const variableValues = ref<Record<string, string>>({})
const showResultsView = ref(false)

// Search prompt filtering
const searchPromptQuery = ref('')

// Delete workflow state
const showDeleteConfirm = ref(false)
const deleting = ref(false)

async function handleDeleteWorkflow() {
  deleting.value = true
  try {
    await store.deleteWorkflow(props.id)
    router.push('/workflows')
  } catch (e) {
    toast.error('Failed to delete workflow')
  } finally {
    deleting.value = false
    showDeleteConfirm.value = false
  }
}

onMounted(async () => {
  await Promise.all([
    store.fetchWorkflow(props.id),
    promptsStore.fetchPrompts(),
    foldersStore.fetchFolders()
  ])
  if (store.currentWorkflow) {
    title.value = store.currentWorkflow.title
    folderId.value = store.currentWorkflow.folder_id
  }
})

// Auto-save logic
const { isDirty, lastSaved, trigger: triggerSave } = useAutoSave(async () => {
  await store.updateWorkflow({
    id: props.id,
    title: title.value || undefined,
    folder_id: folderId.value || null,
  })
}, 30000)

watch([title, folderId], () => {
  triggerSave()
})

// Navigation
function goBack() {
  router.push('/workflows')
}

// Reordering Logic
async function moveStep(index: number, direction: 'up' | 'down') {
  if (!store.currentWorkflow) return
  const targetIndex = direction === 'up' ? index - 1 : index + 1
  const steps = store.currentWorkflow.steps
  if (targetIndex < 0 || targetIndex >= steps.length) return

  const reordered = [...steps]
  const temp = reordered[index]
  reordered[index] = reordered[targetIndex]
  reordered[targetIndex] = temp

  const orderPayload = reordered.map((step, idx) => [step.id, idx] as [string, number])
  
  try {
    await store.reorderSteps(props.id, orderPayload)
    toast.success('Steps reordered')
  } catch (e) {
    toast.error('Reordering failed')
  }
}

// Edit Step Title
async function updateStepTitle(step: WorkflowStep, newTitle: string) {
  if (!newTitle.trim()) return
  try {
    await store.updateStep({
      id: step.id,
      title: newTitle.trim(),
    })
    toast.success('Step title updated')
  } catch (e) {
    toast.error('Failed to update step title')
  }
}

// Remove Step
async function removeStep(id: string) {
  try {
    await store.deleteStep(id)
    toast.success('Step removed')
  } catch (e) {
    toast.error('Failed to remove step')
  }
}

// Replace Prompt Modal Trigger
function triggerReplacePrompt(stepId: string) {
  replacingStepId.value = stepId
  searchPromptQuery.value = ''
  selectedPromptId.value = null
  showLinkPromptDialog.value = true
}

// Filtered prompts list for linking
const filteredPrompts = computed(() => {
  const q = searchPromptQuery.value.trim().toLowerCase()
  if (!q) return promptsStore.prompts
  return promptsStore.prompts.filter(
    p => p.title.toLowerCase().includes(q) || p.content.toLowerCase().includes(q)
  )
})

// Select Prompt for Link
function selectPromptForLink(prompt: PromptWithMeta) {
  selectedPromptId.value = prompt.id
  if (!stepTitleInput.value.trim()) {
    stepTitleInput.value = prompt.title
  }
}

// Execute Linking Prompt (Creates a new step or replaces linked prompt in existing step)
async function confirmLinkPrompt() {
  if (!selectedPromptId.value) return
  
  try {
    if (replacingStepId.value) {
      // Replacing existing step's prompt
      await store.linkPromptToStep(replacingStepId.value, selectedPromptId.value)
      toast.success('Step prompt replaced')
      replacingStepId.value = null
    } else {
      // Adding new step with linked prompt
      const idx = store.currentWorkflow?.steps.length || 0
      await store.addStep({
        workflow_id: props.id,
        order_index: idx,
        title: stepTitleInput.value.trim() || 'New Step',
        instruction: '',
        prompt_id: selectedPromptId.value,
      })
      toast.success('Step added successfully')
    }
    showLinkPromptDialog.value = false
    selectedPromptId.value = null
    stepTitleInput.value = ''
  } catch (e) {
    toast.error('Failed to link prompt')
  }
}

// Inline Prompt Creation and linking as a Step
async function handleCreateInlinePrompt() {
  if (!inlinePromptTitle.value.trim() || !inlinePromptContent.value.trim()) return
  
  try {
    // Create new prompt
    const createdPrompt = await promptsStore.createPrompt({
      title: inlinePromptTitle.value.trim(),
      content: inlinePromptContent.value.trim(),
      description: '',
      tags: [],
      category: null
    })

    if (createdPrompt) {
      // Add workflow step linking to newly created prompt
      const idx = store.currentWorkflow?.steps.length || 0
      await store.addStep({
        workflow_id: props.id,
        order_index: idx,
        title: inlinePromptTitle.value.trim(),
        instruction: '',
        prompt_id: createdPrompt.id,
      })
      toast.success('Inline prompt created and step added')
    }
    
    // Reset inline creation state
    showCreatePromptDialog.value = false
    inlinePromptTitle.value = ''
    inlinePromptContent.value = ''
  } catch (e) {
    toast.error('Failed to create inline prompt')
  }
}

// Variable Extraction logic for all prompts in the current workflow
const uniqueVariables = computed(() => {
  const vars = new Set<string>()
  if (!store.currentWorkflow) return []
  
  for (const step of store.currentWorkflow.steps) {
    const prompt = promptsStore.prompts.find(p => p.id === step.prompt_id)
    if (prompt) {
      const matches = prompt.content.matchAll(/\{\{([^}]+)\}\}/g)
      for (const match of matches) {
        vars.add(match[1].trim())
      }
    }
  }
  return Array.from(vars)
})

// Trigger "Use Workflow" variables dialog
function triggerUseWorkflow() {
  if (!store.currentWorkflow || store.currentWorkflow.steps.length === 0) {
    toast.error('Add at least one step to use the workflow')
    return
  }
  
  // Set default empty values for variables
  const vals: Record<string, string> = {}
  uniqueVariables.value.forEach(v => {
    vals[v] = ''
  })
  variableValues.value = vals
  showResultsView.value = false
  showUseWorkflowDialog.value = true
}

// Resolve variables
const resolvedSteps = computed(() => {
  if (!store.currentWorkflow) return []
  
  return store.currentWorkflow.steps.map(step => {
    const prompt = promptsStore.prompts.find(p => p.id === step.prompt_id)
    let resolved = prompt ? prompt.content : ''
    
    // Replace variable keys with user values
    resolved = resolved.replace(/\{\{([^}]+)\}\}/g, (_, name) => {
      const key = name.trim()
      return variableValues.value[key] !== undefined && variableValues.value[key] !== ''
        ? variableValues.value[key]
        : `{{${key}}}`
    })
    
    return {
      step_id: step.id,
      step_title: step.title,
      prompt_title: prompt ? prompt.title : 'No Prompt Linked',
      resolved_content: resolved
    }
  })
})

// Trigger Results view
function showResolvedResults() {
  showResultsView.value = true
}

// Helper to copy text to clipboard
async function copyToClipboard(text: string) {
  try {
    await navigator.clipboard.writeText(text)
    toast.success('Copied to clipboard')
  } catch (e) {
    toast.error('Failed to copy')
  }
}

// Copy single step resolved output
function copyStepOutput(content: string) {
  copyToClipboard(content)
}

// Copy all step outputs joined together
function copyAllOutputs() {
  const output = resolvedSteps.value.map(s => `[Step: ${s.step_title}]\n\n${s.resolved_content}`).join('\n\n---\n\n')
  copyToClipboard(output)
}

// Export resolved output to Markdown (.md)
function exportWorkflowResults() {
  const content = resolvedSteps.value.map(s => {
    return `## Step: ${s.step_title}\n**Linked Prompt:** ${s.prompt_title}\n\n${s.resolved_content}`
  }).join('\n\n---\n\n')
  
  const blob = new Blob([content], { type: 'text/markdown;charset=utf-8;' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.setAttribute('download', `${store.currentWorkflow?.title.replace(/\s+/g, '_') || 'workflow'}_resolved.md`)
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
  toast.success('Results exported as markdown')
}

// Helper to look up linked prompt metadata
function getStepPrompt(promptId: string | null) {
  if (!promptId) return null
  return promptsStore.prompts.find(p => p.id === promptId) ?? null
}
</script>

<template>
  <div class="workflow-editor-view animate-fade-in flex flex-col h-full w-full py-1">
    
    <!-- Top Nav Action Bar -->
    <div class="flex items-center gap-3 mb-6 flex-shrink-0">
      <button
        id="wfe-back"
        class="p-2 rounded-xl transition-colors cursor-pointer hover:bg-gray-100 dark:hover:bg-zinc-900 border border-gray-100 dark:border-zinc-850"
        style="color: var(--color-text-muted); background: var(--color-bg-tertiary);"
        @click="goBack"
      >
        <ArrowLeft :size="16" />
      </button>

      <h1
        class="text-xl font-bold flex-1 tracking-tight"
        style="background: linear-gradient(135deg, var(--color-accent-blue), var(--color-accent-violet)); -webkit-background-clip: text; -webkit-text-fill-color: transparent;"
      >
        {{ title || 'Workflow Editor' }}
      </h1>

      <div class="flex items-center gap-3">
        <!-- Save Status Indicator -->
        <div class="flex items-center gap-2 text-xs mr-2 font-medium" style="color: var(--color-text-muted);">
          <template v-if="isDirty">
            <Save :size="14" class="animate-pulse" />
            <span>Saving...</span>
          </template>
          <template v-else-if="lastSaved">
            <Check :size="14" style="color: var(--color-accent-emerald);" />
            <span>Saved</span>
          </template>
        </div>

        <!-- Primary Execution Button -->
        <Button
          id="wfe-use-btn"
          class="flex items-center gap-1.5 px-5 h-9 rounded-xl bg-gray-900 hover:bg-indigo-650 text-white font-semibold transition-all cursor-pointer text-xs dark:bg-zinc-100 dark:text-zinc-950 dark:hover:bg-indigo-600 dark:hover:text-white"
          :disabled="!store.currentWorkflow?.steps.length"
          @click="triggerUseWorkflow"
        >
          <Play :size="13" class="fill-white" />
          <span>Use Workflow</span>
        </Button>
      </div>
    </div>

    <!-- Main Workspace Container -->
    <div v-if="store.currentWorkflow" class="flex-1 flex gap-6 min-h-0 overflow-hidden">
      
      <!-- Left side panel: Metadata controls -->
      <div class="w-80 flex-shrink-0 flex flex-col gap-4 overflow-y-auto pt-1">
        <div class="glass-card p-5 border border-gray-150 rounded-2xl bg-white dark:bg-zinc-950 dark:border-zinc-850 flex flex-col gap-5">
          <h3 class="text-[11px] font-bold uppercase tracking-widest text-gray-400 dark:text-zinc-500">
            Workflow Details
          </h3>

          <div class="flex flex-col gap-4">
            <!-- Title -->
            <div class="flex flex-col gap-1.5">
              <label for="wfe-title-input" class="text-xs font-semibold text-gray-650 dark:text-zinc-400">Title</label>
              <input
                id="wfe-title-input"
                v-model="title"
                type="text"
                placeholder="Workflow name..."
                class="w-full px-3.5 py-2.5 rounded-xl text-sm bg-gray-50 border border-gray-200 text-gray-900 outline-none focus-visible:ring-indigo-500 dark:bg-zinc-900 dark:border-zinc-800 dark:text-gray-100"
              />
            </div>

            <!-- Folder Selector -->
            <div class="flex flex-col gap-1.5">
              <label for="wfe-folder-select" class="text-xs font-semibold text-gray-650 dark:text-zinc-400">Folder</label>
              <select
                id="wfe-folder-select"
                v-model="folderId"
                class="w-full px-3.5 py-2.5 rounded-xl text-sm bg-gray-50 border border-gray-200 text-gray-900 outline-none cursor-pointer dark:bg-zinc-900 dark:border-zinc-800 dark:text-gray-100"
              >
                <option :value="null">No Folder</option>
                <option v-for="f in foldersStore.folders" :key="f.id" :value="f.id">
                  {{ f.name }}
                </option>
              </select>
            </div>
          </div>

          <!-- Metadata Properties -->
          <div class="border-t border-gray-100 pt-4 flex flex-col gap-2.5 text-xs text-gray-450 dark:border-zinc-850 dark:text-zinc-500">
            <div class="flex items-center justify-between">
              <span>Steps Count:</span>
              <span class="font-bold text-gray-700 dark:text-zinc-300">{{ store.currentWorkflow.steps.length }} steps</span>
            </div>
            <div class="flex items-center justify-between">
              <span>Created:</span>
              <span class="font-mono text-[11px]">{{ formatDate(store.currentWorkflow.created_at) }}</span>
            </div>
            <div class="flex items-center justify-between">
              <span>Last Modified:</span>
              <span class="font-mono text-[11px]">{{ formatDate(store.currentWorkflow.updated_at) }}</span>
            </div>
          </div>

          <!-- Delete Workflow -->
          <div class="border-t border-gray-100 pt-4 dark:border-zinc-850">
            <button
              class="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl text-[13px] font-semibold border border-red-200 bg-red-50/50 hover:bg-red-100 text-red-650 hover:border-red-300 transition-all cursor-pointer dark:bg-red-950/20 dark:border-red-900/50 dark:text-red-400 dark:hover:bg-red-950/40"
              @click="showDeleteConfirm = true"
            >
              <Trash2 :size="14" />
              <span>Delete Workflow</span>
            </button>
          </div>
        </div>
      </div>

      <!-- Center: Workflow Step Sequence List -->
      <div class="flex-1 flex flex-col gap-4 overflow-y-auto pr-2">
        <h3 class="text-[11px] font-bold uppercase tracking-widest text-gray-400 dark:text-zinc-500 mb-1 pl-1">
          Steps Sequence List
        </h3>

        <!-- Empty state inside editor -->
        <div v-if="store.currentWorkflow.steps.length === 0" class="border-2 border-dashed border-gray-200 rounded-2xl bg-gray-50/20 p-12 text-center dark:border-zinc-850 flex flex-col items-center justify-center min-h-[300px]">
          <FileText :size="36" class="text-gray-300 dark:text-zinc-700 mb-3" />
          <h4 class="font-bold text-gray-800 text-[14px] dark:text-zinc-300">No steps defined yet</h4>
          <p class="text-xs text-gray-450 mt-1 max-w-xs leading-relaxed dark:text-zinc-550">
            Link existing templates or build custom inline prompts to construct your process sequence.
          </p>
        </div>

        <!-- Render Step Cards -->
        <div v-else class="flex flex-col gap-3.5">
          <div
            v-for="(step, index) in store.currentWorkflow.steps"
            :key="step.id"
            class="step-card group border border-gray-200/80 rounded-2xl bg-white shadow-sm flex flex-col overflow-hidden dark:bg-zinc-950 dark:border-zinc-850 hover:shadow-md hover:border-indigo-200 dark:hover:border-indigo-900/40 transition-all duration-200"
          >
            <!-- Step Card Top Bar -->
            <div class="px-5 py-4 flex items-center gap-3 border-b border-gray-100 bg-gray-50/40 dark:bg-zinc-900/10 dark:border-zinc-850">
              <span class="text-[11px] font-mono font-bold px-2.5 py-0.5 rounded-lg bg-indigo-50 text-indigo-650 border border-indigo-100 dark:bg-indigo-950/30 dark:text-indigo-400 dark:border-indigo-900/20 select-none">
                Step {{ index + 1 }}
              </span>

              <!-- Editable Step Title -->
              <input
                type="text"
                :value="step.title"
                placeholder="Enter step title..."
                class="flex-1 font-bold text-sm bg-transparent border-0 outline-none text-gray-900 focus:ring-0 dark:text-gray-100 px-1 py-0.5 rounded-md hover:bg-gray-100/50 focus:bg-white dark:hover:bg-zinc-900 focus:dark:bg-zinc-900 transition-colors"
                @change="updateStepTitle(step, ($event.target as HTMLInputElement).value)"
              />

              <!-- Order / Action controls -->
              <div class="flex items-center gap-1 flex-shrink-0">
                <!-- Move Up -->
                <button
                  class="p-1.5 rounded-lg text-gray-400 hover:text-gray-700 hover:bg-gray-100 dark:hover:bg-zinc-900 cursor-pointer disabled:opacity-30 disabled:pointer-events-none transition-colors"
                  :disabled="index === 0"
                  title="Move Up"
                  @click="moveStep(index, 'up')"
                >
                  <ChevronUp :size="15" />
                </button>
                <!-- Move Down -->
                <button
                  class="p-1.5 rounded-lg text-gray-400 hover:text-gray-700 hover:bg-gray-100 dark:hover:bg-zinc-900 cursor-pointer disabled:opacity-30 disabled:pointer-events-none transition-colors"
                  :disabled="index === store.currentWorkflow.steps.length - 1"
                  title="Move Down"
                  @click="moveStep(index, 'down')"
                >
                  <ChevronDown :size="15" />
                </button>
                
                <div class="w-px h-5 bg-gray-200 dark:bg-zinc-800 mx-1"></div>

                <!-- Replace Prompt -->
                <button
                  class="flex items-center gap-1 px-2.5 py-1.5 rounded-lg text-[11px] font-bold text-indigo-600 bg-indigo-50 border border-indigo-100 hover:bg-indigo-100/60 transition-colors cursor-pointer dark:bg-indigo-950/20 dark:text-indigo-400 dark:border-indigo-900/30"
                  title="Replace Linked Prompt"
                  @click="triggerReplacePrompt(step.id)"
                >
                  <LinkIcon :size="11" />
                  <span>Replace Prompt</span>
                </button>
                
                <!-- Remove Step -->
                <button
                  class="p-1.5 rounded-lg text-gray-400 hover:text-red-500 hover:bg-red-50 transition-all cursor-pointer dark:hover:bg-red-950/30 dark:hover:text-red-400"
                  title="Remove Step"
                  @click="removeStep(step.id)"
                >
                  <Trash2 :size="15" />
                </button>
              </div>
            </div>

            <!-- Step Card Content (Linked Prompt view only) -->
            <div class="px-6 py-4 bg-white dark:bg-zinc-950 flex flex-col gap-3">
              <div v-if="getStepPrompt(step.prompt_id)" class="flex flex-col gap-2">
                <div class="flex items-center gap-1.5 text-xs text-gray-400 font-medium">
                  <FileText :size="13" />
                  <span>Linked Prompt:</span>
                  <strong class="text-gray-750 dark:text-zinc-300 font-bold">{{ getStepPrompt(step.prompt_id)?.title }}</strong>
                </div>
                <!-- Prompt preview content -->
                <pre class="bg-gray-50 border border-gray-100 dark:bg-zinc-900/40 dark:border-zinc-850 rounded-xl p-4 font-mono text-[12px] whitespace-pre-wrap leading-relaxed text-gray-550 max-h-[160px] overflow-y-auto">{{ getStepPrompt(step.prompt_id)?.content }}</pre>
              </div>
              <div v-else class="text-xs text-gray-400 italic">No prompt linked. Use "Replace Prompt" to connect a template.</div>
            </div>
          </div>
        </div>

        <!-- Add Step buttons -->
        <div class="mt-4 flex items-center justify-stretch gap-4">
          <button
            class="flex-1 flex items-center justify-center gap-2 py-3 border-2 border-dashed border-gray-200 hover:border-indigo-400 hover:bg-indigo-50/20 text-gray-450 hover:text-indigo-600 rounded-2xl text-xs font-bold transition-all cursor-pointer dark:border-zinc-850 dark:hover:border-indigo-900/50 dark:hover:text-indigo-400"
            @click="stepTitleInput = ''; selectedPromptId = null; searchPromptQuery = ''; showLinkPromptDialog = true; replacingStepId = null"
          >
            <LinkIcon :size="14" />
            <span>Link Existing Prompt</span>
          </button>
          
          <button
            class="flex-1 flex items-center justify-center gap-2 py-3 border-2 border-dashed border-gray-200 hover:border-indigo-400 hover:bg-indigo-50/20 text-gray-450 hover:text-indigo-600 rounded-2xl text-xs font-bold transition-all cursor-pointer dark:border-zinc-850 dark:hover:border-indigo-900/50 dark:hover:text-indigo-400"
            @click="inlinePromptTitle = ''; inlinePromptContent = ''; showCreatePromptDialog = true"
          >
            <Plus :size="14" />
            <span>Create New Prompt Inline</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Loading Skeleton State -->
    <div v-else-if="store.loading" class="flex-1 flex items-center justify-center">
      <div class="w-8 h-8 rounded-full border-2 border-indigo-200 border-t-indigo-650 animate-spin" />
    </div>

    <!-- ─── Modals ─── -->

    <!-- Modal: Link Existing Prompt -->
    <Dialog :open="showLinkPromptDialog" @update:open="showLinkPromptDialog = $event">
      <DialogContent class="sm:max-w-xl bg-white border border-gray-200 rounded-2xl shadow-xl dark:bg-zinc-950 dark:border-zinc-850">
        <DialogHeader>
          <DialogTitle class="font-bold text-gray-900 text-lg dark:text-gray-100">
            {{ replacingStepId ? 'Replace Step Prompt' : 'Link Existing Prompt' }}
          </DialogTitle>
          <DialogDescription class="text-xs text-gray-400">
            Select a prompt template from your library to link to this step.
          </DialogDescription>
        </DialogHeader>

        <div class="flex flex-col gap-4 py-2 px-2 min-h-[300px] max-h-[440px] overflow-y-auto pr-3">
          <!-- Search box -->
          <div class="relative">
            <Search :size="14" class="absolute top-3.5 left-3.5 text-gray-400" />
            <input
              v-model="searchPromptQuery"
              type="text"
              placeholder="Search prompts in library..."
              class="w-full h-11 pl-10 pr-4 py-3 rounded-xl text-xs bg-gray-50 border border-gray-200 text-gray-900 outline-none focus-visible:ring-indigo-500 dark:bg-zinc-900 dark:border-zinc-800 dark:text-gray-100"
            />
          </div>

          <!-- Prompts list -->
          <div class="flex flex-col gap-2 flex-1 overflow-y-auto pr-1">
            <div
              v-for="p in filteredPrompts"
              :key="p.id"
              class="flex flex-col gap-1 p-3 border rounded-xl cursor-pointer transition-all text-left"
              :class="selectedPromptId === p.id 
                ? 'border-indigo-500 bg-indigo-50/20 dark:bg-indigo-950/20' 
                : 'border-gray-100 hover:border-gray-250 hover:bg-gray-50 dark:border-zinc-850 dark:hover:bg-zinc-900'"
              @click="selectPromptForLink(p)"
            >
              <span class="font-bold text-sm text-gray-900 dark:text-gray-100">{{ p.title }}</span>
              <span class="text-xs text-gray-450 line-clamp-1 dark:text-zinc-500 mt-0.5">{{ p.content }}</span>
            </div>
            <div v-if="filteredPrompts.length === 0" class="text-xs text-gray-450 italic text-center p-8">No prompts match your query</div>
          </div>

          <!-- Step Title input (Rendered only when creating a new step) -->
          <div v-if="selectedPromptId && !replacingStepId" class="flex flex-col gap-1.5 pt-2 border-t border-gray-100 dark:border-zinc-850">
            <label for="step-title-input" class="text-xs font-semibold text-gray-600 dark:text-zinc-400">Step Title for Workflow</label>
            <input
              id="step-title-input"
              v-model="stepTitleInput"
              type="text"
              placeholder="e.g. Prepare Study Material"
              class="w-full h-10 px-3.5 py-2.5 rounded-xl text-xs bg-gray-50 border border-gray-200 text-gray-900 outline-none dark:bg-zinc-900 dark:border-zinc-800 dark:text-gray-100"
            />
          </div>
        </div>

        <DialogFooter>
          <Button variant="ghost" class="h-9 px-4 rounded-lg text-xs" @click="showLinkPromptDialog = false">Cancel</Button>
          <Button
            class="h-9 px-5 rounded-lg text-xs font-bold text-white bg-gray-900 hover:bg-indigo-600 disabled:opacity-40 shadow-sm dark:bg-zinc-100 dark:text-zinc-950 dark:hover:bg-indigo-500 dark:hover:text-white"
            :disabled="!selectedPromptId || (!replacingStepId && !stepTitleInput.trim())"
            @click="confirmLinkPrompt"
          >
            {{ replacingStepId ? 'Replace Prompt' : 'Add Step' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Modal: Create New Prompt Inline -->
    <Dialog :open="showCreatePromptDialog" @update:open="showCreatePromptDialog = $event">
      <DialogContent class="sm:max-w-3xl bg-white border border-gray-200 rounded-2xl shadow-xl dark:bg-zinc-950 dark:border-zinc-850">
        <DialogHeader>
          <DialogTitle class="font-bold text-gray-900 text-lg dark:text-gray-100">Create New Prompt Inline</DialogTitle>
          <DialogDescription class="text-xs text-gray-400">
            Add a new prompt template directly to your library and link it to this step sequence.
          </DialogDescription>
        </DialogHeader>

        <div class="flex flex-col gap-4 py-2 text-left">
          <!-- Prompt Title -->
          <div class="flex flex-col gap-1.5">
            <label for="inline-title" class="text-xs font-semibold text-gray-650 dark:text-zinc-400">Prompt Title</label>
            <input
              id="inline-title"
              v-model="inlinePromptTitle"
              type="text"
              placeholder="e.g. Generate MCQs"
              class="w-full h-10 px-3.5 py-2.5 rounded-xl text-xs bg-gray-50 border border-gray-200 text-gray-900 outline-none dark:bg-zinc-900 dark:border-zinc-800 dark:text-gray-100"
            />
          </div>

          <!-- Prompt Content -->
          <div class="flex flex-col gap-1.5">
            <div class="flex items-center justify-between">
              <label for="inline-content" class="text-xs font-semibold text-gray-650 dark:text-zinc-400">Prompt Content</label>
              <span class="text-[10px] text-gray-400">Use <code v-pre class="bg-indigo-50 px-1 py-0.5 rounded text-indigo-500 dark:bg-indigo-950/40 dark:text-indigo-400">{{variable}}</code> for variables</span>
            </div>
            <textarea
              id="inline-content"
              v-model="inlinePromptContent"
              placeholder="Write or paste prompt instructions here..."
              rows="12"
              class="w-full px-4 py-3 rounded-xl text-xs resize-none font-mono bg-gray-50 border border-gray-200 text-gray-900 outline-none dark:bg-zinc-900 dark:border-zinc-800 dark:text-gray-100"
            />
          </div>
        </div>

        <DialogFooter>
          <Button variant="ghost" class="h-9 px-4 rounded-lg text-xs" @click="showCreatePromptDialog = false">Cancel</Button>
          <Button
            class="h-9 px-5 rounded-lg text-xs font-bold text-white bg-gray-900 hover:bg-indigo-600 disabled:opacity-40 shadow-sm dark:bg-zinc-100 dark:text-zinc-950 dark:hover:bg-indigo-500 dark:hover:text-white"
            :disabled="!inlinePromptTitle.trim() || !inlinePromptContent.trim()"
            @click="handleCreateInlinePrompt"
          >
            Create & Link
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Modal: Use Workflow (Variables Collection & Output Results) -->
    <Dialog :open="showUseWorkflowDialog" @update:open="showUseWorkflowDialog = $event">
      <DialogContent class="sm:max-w-5xl bg-white border border-gray-200 rounded-2xl shadow-xl dark:bg-zinc-950 dark:border-zinc-850" :showCloseButton="true">
        <DialogHeader>
          <div class="flex items-center gap-2.5 mb-1">
            <div class="w-8 h-8 rounded-lg bg-gradient-to-br from-indigo-500 to-violet-500 flex items-center justify-center text-white flex-shrink-0 shadow-sm">
              <Play :size="14" class="fill-white" />
            </div>
            <div class="flex flex-col gap-0.5">
              <span class="text-[11px] font-bold uppercase tracking-widest text-indigo-500 dark:text-indigo-400">
                {{ showResultsView ? 'Resolved Output' : 'Run Workflow' }}
              </span>
              <DialogTitle class="font-bold text-gray-900 text-lg leading-tight dark:text-gray-100">
                {{ store.currentWorkflow?.title || 'Workflow' }}
              </DialogTitle>
            </div>
          </div>
          <DialogDescription class="text-xs text-gray-400 mt-1">
            {{ showResultsView ? 'Copy or export the templates with inputs resolved.' : 'Enter values for unique variables found across linked prompts.' }}
          </DialogDescription>
        </DialogHeader>

        <!-- Dynamic Inputs State -->
        <div v-if="!showResultsView" class="flex flex-col gap-5 py-5 px-2 max-h-[60vh] overflow-y-auto text-left pr-3">
          <div v-if="uniqueVariables.length > 0" class="flex flex-col gap-5">
            <div
              v-for="(v, idx) in uniqueVariables"
              :key="v"
              class="flex flex-col gap-2 pb-5"
              :class="idx < uniqueVariables.length - 1 ? 'border-b border-gray-100 dark:border-zinc-850' : ''"
            >
              <label :for="`var-input-${v}`" class="text-[13px] font-bold capitalize text-gray-700 dark:text-zinc-300 flex items-center gap-2">
                <span class="w-1.5 h-1.5 rounded-full bg-indigo-400 dark:bg-indigo-500 flex-shrink-0"></span>
                {{ v }}
              </label>
              <input
                :id="`var-input-${v}`"
                v-model="variableValues[v]"
                type="text"
                :placeholder="`Enter value for ${v}...`"
                class="w-full h-11 px-4 py-3 rounded-xl text-sm bg-gray-50 border border-gray-200 text-gray-900 outline-none focus:border-indigo-400 focus:ring-1 focus:ring-indigo-400/30 ring-offset-0 transition-all dark:bg-zinc-900 dark:border-zinc-800 dark:text-gray-100 dark:focus:border-indigo-500 dark:focus:ring-indigo-500/20"
              />
            </div>
          </div>
          <!-- Zero variables empty state -->
          <div v-else class="text-center p-10 flex flex-col items-center justify-center">
            <div class="w-14 h-14 rounded-2xl bg-gradient-to-br from-indigo-50 to-violet-50 dark:from-indigo-950/30 dark:to-violet-950/30 flex items-center justify-center mb-3">
              <Sparkles :size="24" class="text-indigo-400 animate-pulse" />
            </div>
            <h4 class="font-bold text-gray-800 dark:text-zinc-300">No prompt variables detected</h4>
            <p class="text-xs text-gray-455 mt-1.5 max-w-xs leading-relaxed dark:text-zinc-550">
              Your templates don't contain any variable tags (like <code v-pre>{{topic}}</code>). You can generate outputs immediately.
            </p>
          </div>
        </div>

        <!-- Output Resolved Results State -->
        <div v-else class="flex flex-col gap-5 py-4 px-2 max-h-[70vh] overflow-y-auto pr-3">
          <div class="flex flex-col gap-4 text-left">
            <div
              v-for="(rStep, index) in resolvedSteps"
              :key="rStep.step_id"
              class="flex flex-col border border-gray-150 rounded-xl overflow-hidden bg-white dark:bg-zinc-950 dark:border-zinc-850"
            >
              <!-- Result header -->
              <div class="px-4 py-2.5 bg-gray-50/80 border-b border-gray-150 flex items-center justify-between dark:bg-zinc-900/20 dark:border-zinc-850">
                <span class="text-[12px] font-bold text-gray-800 dark:text-zinc-300">
                  Step {{ index + 1 }}: {{ rStep.step_title }}
                </span>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7 text-gray-400 hover:text-indigo-600 transition-colors cursor-pointer"
                  title="Copy this step template"
                  @click="copyStepOutput(rStep.resolved_content)"
                >
                  <Copy :size="13" />
                </Button>
              </div>
              <!-- Result code block -->
              <pre class="p-4 font-mono text-[12px] whitespace-pre-wrap leading-relaxed text-gray-650 bg-gray-50/20 dark:bg-zinc-900/10">{{ rStep.resolved_content }}</pre>
            </div>
          </div>
        </div>

        <DialogFooter>
          <!-- Back/Cancel Actions -->
          <Button
            v-if="showResultsView && uniqueVariables.length > 0"
            variant="ghost"
            class="h-9 px-4 rounded-lg text-xs"
            @click="showResultsView = false"
          >
            Back to variables
          </Button>
          <Button
            v-else
            variant="ghost"
            class="h-9 px-4 rounded-lg text-xs"
            @click="showUseWorkflowDialog = false"
          >
            Close
          </Button>

          <!-- Action execution triggers -->
          <Button
            v-if="!showResultsView"
            class="h-9 px-5 rounded-lg text-xs font-bold text-white bg-gray-900 hover:bg-indigo-600 shadow-sm dark:bg-zinc-100 dark:text-zinc-950 dark:hover:bg-indigo-500 dark:hover:text-white"
            @click="showResolvedResults"
          >
            Resolve Prompts
          </Button>
          <div v-else class="flex items-center gap-2">
            <Button
              variant="outline"
              class="h-9 px-4 rounded-lg text-xs font-semibold gap-1.5"
              @click="copyAllOutputs"
            >
              <Copy :size="12" />
              <span>Copy All</span>
            </Button>
            <Button
              class="h-9 px-4 rounded-lg text-xs font-bold text-white bg-gray-900 hover:bg-indigo-600 gap-1.5 shadow-sm dark:bg-zinc-100 dark:text-zinc-950 dark:hover:bg-indigo-500 dark:hover:text-white"
              @click="exportWorkflowResults"
            >
              <Download :size="12" />
              <span>Export as Markdown</span>
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>

    <!-- Delete Workflow Confirmation Modal -->
    <Teleport to="body">
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
                  Delete Workflow
                </h3>
                <p class="text-sm text-gray-500 mt-3 leading-relaxed dark:text-zinc-400">
                  Are you sure you want to delete <strong class="text-gray-800 dark:text-gray-200">"{{ store.currentWorkflow?.title }}"</strong>? This will permanently remove the workflow and all its steps. This action cannot be undone.
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
                @click="handleDeleteWorkflow"
              >
                <Trash2 v-if="!deleting" :size="14" />
                <div v-else class="w-3.5 h-3.5 rounded-full border-2 border-white/30 border-t-white animate-spin" />
                <span>{{ deleting ? 'Deleting...' : 'Delete Workflow' }}</span>
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
</template>

<style scoped>
.line-clamp-1 {
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

@keyframes scale-in {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}
.animate-scale-in {
  animation: scale-in 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
