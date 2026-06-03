<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { Trash2, Link, FileText, ChevronUp, ChevronDown, ChevronRight, Save } from 'lucide-vue-next'
import { useWorkflowsStore } from '@/stores/workflows'
import { usePromptsStore } from '@/stores/prompts'
import type { WorkflowStep } from '@/types'
import { toast } from 'vue-sonner'

const props = defineProps<{
  steps: WorkflowStep[]
  workflowId: string
}>()

const workflowsStore = useWorkflowsStore()
const promptsStore = usePromptsStore()
const expandedStepId = ref<string | null>(null)

onMounted(() => {
  promptsStore.fetchPrompts()
})

async function deleteStep(id: string) {
  if (confirm('Are you sure you want to delete this step?')) {
    try {
      await workflowsStore.deleteStep(id)
      toast.success('Step deleted')
    } catch (e) {
      toast.error('Failed to delete step')
    }
  }
}

async function moveStep(index: number, direction: 'up' | 'down') {
  const targetIndex = direction === 'up' ? index - 1 : index + 1
  if (targetIndex < 0 || targetIndex >= props.steps.length) return

  const reordered = [...props.steps]
  const temp = reordered[index]
  reordered[index] = reordered[targetIndex]
  reordered[targetIndex] = temp

  const orderPayload = reordered.map((step, idx) => [step.id, idx] as [string, number])
  
  try {
    await workflowsStore.reorderSteps({
      workflow_id: props.workflowId,
      order: orderPayload,
    })
    toast.success('Steps reordered')
  } catch (e) {
    toast.error('Reordering failed')
  }
}

async function saveStepDetails(step: WorkflowStep, newTitle: string, newInstruction: string, promptId: string | null) {
  try {
    await workflowsStore.updateStep({
      id: step.id,
      title: newTitle,
      instruction: newInstruction,
      prompt_id: promptId || null,
    })
    toast.success('Step updated')
  } catch (e) {
    toast.error('Failed to update step')
  }
}
</script>

<template>
  <div class="flex flex-col gap-3">
    <div
      v-for="(step, index) in steps"
      :key="step.id"
      class="glass-card overflow-hidden transition-all"
      :style="expandedStepId === step.id ? 'border-color: var(--color-accent-violet);' : 'border-color: var(--color-border-subtle);'"
    >
      <!-- Step Header -->
      <div
        class="flex items-center gap-3 px-4 py-3 cursor-pointer select-none"
        style="background: var(--color-bg-tertiary);"
        @click="expandedStepId = expandedStepId === step.id ? null : step.id"
      >
        <ChevronDown v-if="expandedStepId === step.id" :size="14" style="color: var(--color-text-muted); flex-shrink: 0;" />
        <ChevronRight v-else :size="14" style="color: var(--color-text-muted); flex-shrink: 0;" />

        <span class="text-xs font-mono font-bold px-2 py-0.5 rounded bg-white/5" style="color: var(--color-text-secondary);">
          Step {{ index + 1 }}
        </span>
        
        <span class="text-sm font-semibold flex-1 truncate" style="color: var(--color-text-primary);">
          {{ step.title }}
        </span>

        <!-- Prompt association badge -->
        <span
          v-if="step.prompt_id"
          class="flex items-center gap-1 text-[10px] px-2 py-0.5 rounded-full"
          style="background: hsl(210 100% 50% / 0.1); color: var(--color-accent-blue); border: 1px solid hsl(210 100% 50% / 0.15);"
        >
          <FileText :size="10" />
          <span>Linked</span>
        </span>

        <!-- Order controls -->
        <div class="flex items-center gap-0.5" @click.stop>
          <button
            class="p-1 rounded cursor-pointer transition-colors"
            style="color: var(--color-text-muted);"
            :disabled="index === 0"
            @click="moveStep(index, 'up')"
          >
            <ChevronUp :size="14" />
          </button>
          <button
            class="p-1 rounded cursor-pointer transition-colors"
            style="color: var(--color-text-muted);"
            :disabled="index === steps.length - 1"
            @click="moveStep(index, 'down')"
          >
            <ChevronDown :size="14" />
          </button>
        </div>

        <button
          class="p-1.5 rounded-lg cursor-pointer transition-colors hover:bg-red-500/10 hover:text-red-400"
          style="color: var(--color-text-muted);"
          @click.stop="deleteStep(step.id)"
        >
          <Trash2 :size="14" />
        </button>
      </div>

      <!-- Expanded edit details panel -->
      <div v-if="expandedStepId === step.id" class="p-4 border-t flex flex-col gap-4 animate-fade-in" style="border-color: var(--color-border-subtle); background: var(--color-bg-primary);">
        <!-- Step Title -->
        <div>
          <label class="text-xs font-semibold mb-1 block" style="color: var(--color-text-secondary);">Step Title</label>
          <input
            :id="`step-title-${step.id}`"
            type="text"
            :value="step.title"
            class="w-full px-3 py-2 rounded-lg text-sm"
            style="background: var(--color-bg-tertiary); border: 1px solid var(--color-border-default); color: var(--color-text-primary); outline: none;"
            @change="saveStepDetails(step, ($event.target as HTMLInputElement).value, step.instruction, step.prompt_id)"
          />
        </div>

        <!-- Linked Prompt -->
        <div>
          <label class="text-xs font-semibold mb-1 block" style="color: var(--color-text-secondary);">Linked Prompt</label>
          <div class="flex gap-2">
            <select
              :id="`step-prompt-${step.id}`"
              :value="step.prompt_id"
              class="flex-1 px-3 py-2 rounded-lg text-sm cursor-pointer"
              style="background: var(--color-bg-tertiary); border: 1px solid var(--color-border-default); color: var(--color-text-primary); outline: none;"
              @change="saveStepDetails(step, step.title, step.instruction, ($event.target as HTMLSelectElement).value || null)"
            >
              <option :value="null">No Prompt Linked</option>
              <option v-for="p in promptsStore.prompts" :key="p.id" :value="p.id">
                {{ p.title }}
              </option>
            </select>
          </div>
        </div>

        <!-- Instructions -->
        <div>
          <label class="text-xs font-semibold mb-1 block" style="color: var(--color-text-secondary);">Instructions / Thinking Pattern</label>
          <textarea
            :id="`step-instruction-${step.id}`"
            :value="step.instruction"
            placeholder="Add structured instructions or questions for this step..."
            rows="4"
            class="w-full px-3 py-2 rounded-lg text-sm resize-none font-mono"
            style="background: var(--color-bg-tertiary); border: 1px solid var(--color-border-default); color: var(--color-text-primary); outline: none;"
            @change="saveStepDetails(step, step.title, ($event.target as HTMLTextAreaElement).value, step.prompt_id)"
          />
        </div>
      </div>
    </div>
  </div>
</template>
