<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { GitBranch, Plus, Trash2, AlertTriangle } from 'lucide-vue-next'
import { useWorkflowsStore } from '@/stores/workflows'
import { useFoldersStore } from '@/stores/folders'
import WorkflowCard from '@/components/workflow/WorkflowCard.vue'

const router = useRouter()
const workflowsStore = useWorkflowsStore()
const foldersStore = useFoldersStore()
const showCreateDialog = ref(false)
const newTitle = ref('')
const newFolderId = ref<string | null>(null)

// Delete workflow state
const showDeleteConfirm = ref(false)
const deletingWorkflowId = ref<string | null>(null)
const deleting = ref(false)

const deletingWorkflow = computed(() => {
  if (!deletingWorkflowId.value) return null
  return workflowsStore.workflows.find(w => w.id === deletingWorkflowId.value) ?? null
})

onMounted(() => {
  workflowsStore.fetchWorkflows()
  foldersStore.fetchFolders()
  window.addEventListener('open-new-workflow', handleOpenCreate)
})

onUnmounted(() => {
  window.removeEventListener('open-new-workflow', handleOpenCreate)
})

function handleOpenCreate() {
  showCreateDialog.value = true
}

async function createWorkflow() {
  if (!newTitle.value.trim()) return
  const wf = await workflowsStore.createWorkflow({
    title: newTitle.value,
    folder_id: newFolderId.value || null,
  })
  if (wf) {
    showCreateDialog.value = false
    newTitle.value = ''
    newFolderId.value = null
    router.push(`/workflows/${wf.id}`)
  }
}

function openWorkflow(id: string) {
  router.push(`/workflows/${id}`)
}

function triggerDelete(id: string) {
  deletingWorkflowId.value = id
  showDeleteConfirm.value = true
}

async function handleDeleteWorkflow() {
  if (!deletingWorkflowId.value) return
  deleting.value = true
  try {
    await workflowsStore.deleteWorkflow(deletingWorkflowId.value)
  } catch (e) {
    // error handled by store
  } finally {
    deleting.value = false
    showDeleteConfirm.value = false
    deletingWorkflowId.value = null
  }
}
</script>

<template>
  <div class="workflows-view animate-fade-in flex flex-col gap-6 w-full py-1">
    <!-- Filters & Indicators Bar -->
    <div class="flex items-center justify-between border-b border-border-subtle pb-3">
      <div class="flex items-center gap-2">
        <GitBranch :size="14" style="color: var(--color-text-muted);" />
        <span class="text-xs font-semibold" style="color: var(--color-text-secondary);">
          Structured Prompt Pipeline Sequences
        </span>
      </div>

      <!-- Action: Add Workflow -->
      <button
        id="wf-add-btn"
        class="flex items-center gap-1.5 px-4 py-2 rounded-xl bg-gray-900 text-white font-semibold transition-all cursor-pointer text-xs hover:bg-gray-800 dark:bg-zinc-100 dark:text-zinc-950 dark:hover:bg-zinc-200 shadow-sm border-0"
        @click="showCreateDialog = true"
      >
        <Plus :size="13" />
        <span>Add Workflow</span>
      </button>
    </div>

    <!-- Loading state -->
    <div v-if="workflowsStore.loading" class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-6">
      <div v-for="i in 8" :key="i" class="skeleton" style="height: 140px;" />
    </div>

    <!-- Workflows Grid -->
    <TransitionGroup
      v-else-if="workflowsStore.workflows.length > 0"
      name="v"
      tag="div"
      class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-6"
    >
      <WorkflowCard
        v-for="wf in workflowsStore.workflows"
        :key="wf.id"
        :workflow="wf"
        @click="openWorkflow(wf.id)"
        @delete="triggerDelete(wf.id)"
      />
    </TransitionGroup>

    <!-- Perfectly Balanced Centered Empty State -->
    <div v-else class="empty-state border border-border-default border-dashed rounded-xl bg-bg-secondary p-12 flex flex-col items-center justify-center text-center" style="min-height: 360px;">
      <GitBranch :size="48" class="empty-icon text-text-muted opacity-30 mb-2" />
      <div class="empty-title text-sm font-bold" style="color: var(--color-text-primary);">No workflow chains created yet</div>
      <div class="empty-description text-xs max-w-sm mb-4" style="color: var(--color-text-muted);">
        Chain prompt templates together into robust, reusable step-by-step sequences.
      </div>
      <button
        class="flex items-center gap-1.5 px-4 py-2.5 rounded-lg text-xs font-semibold transition-all cursor-pointer shadow-sm hover:scale-[1.02] border border-transparent"
        style="background: var(--color-accent-violet); color: white;"
        @click="showCreateDialog = true"
      >
        <Plus :size="14" />
        <span>Create First Workflow</span>
      </button>
    </div>

    <!-- Create Dialog (Overlay modal) -->
    <Teleport to="body">
      <Transition name="v">
        <div v-if="showCreateDialog" class="fixed inset-0 z-50 flex items-center justify-center" style="background: rgba(0,0,0,0.6); backdrop-filter: blur(4px);">
          <div class="glass-card p-6 w-full max-w-md animate-fade-in" style="background: var(--color-bg-secondary); border: 1px solid var(--color-border-default);" @click.stop>
            <h2 class="text-sm font-bold mb-4" style="font-family: var(--font-display); color: var(--color-text-primary);">Create Structured Workflow</h2>

            <div class="flex flex-col gap-3">
              <div>
                <label class="text-xs font-semibold mb-1 block" style="color: var(--color-text-secondary);">Title</label>
                <input
                  id="wf-create-title"
                  v-model="newTitle"
                  type="text"
                  placeholder="e.g. Code Review Assistant"
                  class="w-full px-3 py-2 rounded-lg text-xs"
                  style="background: var(--color-bg-tertiary); border: 1px solid var(--color-border-default); color: var(--color-text-primary); outline: none;"
                />
              </div>

              <div>
                <label class="text-xs font-semibold mb-1 block" style="color: var(--color-text-secondary);">Folder (optional)</label>
                <select
                  id="wf-create-folder"
                  v-model="newFolderId"
                  class="w-full px-3 py-2 rounded-lg text-xs cursor-pointer"
                  style="background: var(--color-bg-tertiary); border: 1px solid var(--color-border-default); color: var(--color-text-primary); outline: none;"
                >
                  <option :value="null">No Folder</option>
                  <option v-for="f in foldersStore.folders" :key="f.id" :value="f.id">
                    {{ f.name }}
                  </option>
                </select>
              </div>
            </div>

            <div class="flex justify-end gap-2 mt-5">
              <button
                class="px-4 py-2 rounded-lg text-xs font-semibold cursor-pointer"
                style="color: var(--color-text-secondary);"
                @click="showCreateDialog = false"
              >
                Cancel
              </button>
              <button
                id="wf-create-submit"
                class="px-4 py-2 rounded-lg text-xs font-semibold cursor-pointer transition-colors"
                style="background: var(--color-accent-violet); color: white;"
                :disabled="!newTitle.trim()"
                @click="createWorkflow"
              >
                Create Sequence
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- Delete Workflow Confirmation Modal -->
    <Teleport to="body">
      <Transition name="fade">
        <div
          v-if="showDeleteConfirm"
          class="fixed inset-0 z-[60] flex items-center justify-center p-6 bg-black/60 backdrop-filter backdrop-blur-sm"
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
                  Are you sure you want to delete <strong class="text-gray-800 dark:text-gray-200">"{{ deletingWorkflow?.title }}"</strong>? This will permanently remove the workflow and all its steps. This action cannot be undone.
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
  </div>
</template>

<style scoped>
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

