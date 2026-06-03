import { ref } from 'vue'
import { defineStore } from 'pinia'
import { toast } from 'vue-sonner'
import type {
  Workflow,
  WorkflowWithSteps,
  WorkflowStep,
  CreateWorkflowRequest,
  UpdateWorkflowRequest,
  CreateWorkflowStepRequest,
  UpdateWorkflowStepRequest,
} from '@/types'
import * as api from '@/lib/tauri-api'

export const useWorkflowsStore = defineStore('workflows', () => {
  const workflows = ref<WorkflowWithSteps[]>([])
  const currentWorkflow = ref<WorkflowWithSteps | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchWorkflows() {
    loading.value = true
    error.value = null
    try {
      workflows.value = await api.listWorkflows()
    } catch (e) {
      error.value = String(e)
      toast.error('Failed to load workflows')
    } finally {
      loading.value = false
    }
  }

  async function fetchWorkflow(id: string) {
    loading.value = true
    error.value = null
    try {
      currentWorkflow.value = await api.getWorkflow(id)
    } catch (e) {
      error.value = String(e)
      toast.error('Failed to load workflow')
    } finally {
      loading.value = false
    }
  }

  async function createWorkflow(request: CreateWorkflowRequest) {
    loading.value = true
    error.value = null
    try {
      const created = await api.createWorkflow(request)
      toast.success('Workflow created')
      await fetchWorkflows()
      return created
    } catch (e) {
      error.value = String(e)
      toast.error('Failed to create workflow')
    } finally {
      loading.value = false
    }
  }

  async function updateWorkflow(request: UpdateWorkflowRequest) {
    loading.value = true
    error.value = null
    try {
      const updated = await api.updateWorkflow(request)
      toast.success('Workflow updated')
      return updated
    } catch (e) {
      error.value = String(e)
      toast.error('Failed to update workflow')
    } finally {
      loading.value = false
    }
  }

  async function deleteWorkflow(id: string) {
    loading.value = true
    error.value = null
    try {
      await api.deleteWorkflow(id)
      workflows.value = workflows.value.filter((w) => w.id !== id)
      if (currentWorkflow.value?.id === id) {
        currentWorkflow.value = null
      }
      toast.success('Workflow deleted')
    } catch (e) {
      error.value = String(e)
      toast.error('Failed to delete workflow')
    } finally {
      loading.value = false
    }
  }

  async function addStep(request: CreateWorkflowStepRequest) {
    error.value = null
    try {
      const step = await api.addWorkflowStep(request)
      if (currentWorkflow.value && currentWorkflow.value.id === request.workflow_id) {
        currentWorkflow.value.steps.push(step)
      }
      toast.success('Step added')
      return step
    } catch (e) {
      error.value = String(e)
      toast.error('Failed to add step')
    }
  }

  async function updateStep(request: UpdateWorkflowStepRequest) {
    error.value = null
    try {
      const updated = await api.updateWorkflowStep(request)
      if (currentWorkflow.value) {
        const idx = currentWorkflow.value.steps.findIndex((s) => s.id === request.id)
        if (idx !== -1) {
          currentWorkflow.value.steps[idx] = updated
        }
      }
      return updated
    } catch (e) {
      error.value = String(e)
      toast.error('Failed to update step')
    }
  }

  async function deleteStep(id: string) {
    error.value = null
    try {
      await api.deleteWorkflowStep(id)
      if (currentWorkflow.value) {
        currentWorkflow.value.steps = currentWorkflow.value.steps.filter((s) => s.id !== id)
      }
      toast.success('Step removed')
    } catch (e) {
      error.value = String(e)
      toast.error('Failed to delete step')
    }
  }

  async function reorderSteps(workflowId: string, order: [string, number][]) {
    error.value = null
    try {
      await api.reorderWorkflowSteps({ workflow_id: workflowId, order })
      if (currentWorkflow.value && currentWorkflow.value.id === workflowId) {
        const orderMap = new Map(order)
        currentWorkflow.value.steps.sort(
          (a, b) => (orderMap.get(a.id) ?? a.order_index) - (orderMap.get(b.id) ?? b.order_index)
        )
      }
    } catch (e) {
      error.value = String(e)
      toast.error('Failed to reorder steps')
    }
  }

  async function linkPromptToStep(stepId: string, promptId: string | null) {
    error.value = null
    try {
      await api.linkPromptToStep(stepId, promptId)
      if (currentWorkflow.value) {
        const step = currentWorkflow.value.steps.find((s: WorkflowStep) => s.id === stepId)
        if (step) {
          step.prompt_id = promptId
        }
      }
      toast.success(promptId ? 'Prompt linked' : 'Prompt unlinked')
    } catch (e) {
      error.value = String(e)
      toast.error('Failed to link prompt')
    }
  }

  return {
    workflows,
    currentWorkflow,
    loading,
    error,
    fetchWorkflows,
    fetchWorkflow,
    createWorkflow,
    updateWorkflow,
    deleteWorkflow,
    addStep,
    updateStep,
    deleteStep,
    reorderSteps,
    linkPromptToStep,
  }
})
