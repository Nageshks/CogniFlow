import { ref } from 'vue'
import { defineStore } from 'pinia'
import { toast } from 'vue-sonner'
import type { PromptWithMeta, PromptVersion, CreatePromptRequest, UpdatePromptRequest } from '@/types'
import * as api from '@/lib/tauri-api'

export const usePromptsStore = defineStore('prompts', () => {
  const prompts = ref<PromptWithMeta[]>([])
  const currentPrompt = ref<PromptWithMeta | null>(null)
  const versions = ref<PromptVersion[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchPrompts(folderId?: string | null, archived?: boolean) {
    loading.value = true
    error.value = null
    try {
      prompts.value = await api.listPrompts(folderId, archived)
    } catch (e) {
      error.value = String(e)
      toast.error('Failed to load prompts')
    } finally {
      loading.value = false
    }
  }

  async function fetchPrompt(id: string) {
    loading.value = true
    error.value = null
    try {
      currentPrompt.value = await api.getPrompt(id)
    } catch (e) {
      error.value = String(e)
      toast.error('Failed to load prompt')
    } finally {
      loading.value = false
    }
  }

  async function createPrompt(request: CreatePromptRequest) {
    loading.value = true
    error.value = null
    try {
      const created = await api.createPrompt(request)
      toast.success('Prompt created')
      await fetchPrompts()
      return created
    } catch (e) {
      error.value = String(e)
      toast.error('Failed to create prompt')
    } finally {
      loading.value = false
    }
  }

  async function updatePrompt(request: UpdatePromptRequest) {
    loading.value = true
    error.value = null
    try {
      const updated = await api.updatePrompt(request)
      toast.success('Prompt updated')
      if (currentPrompt.value?.id === request.id) {
        await fetchPrompt(request.id)
      }
      return updated
    } catch (e) {
      error.value = String(e)
      toast.error('Failed to update prompt')
    } finally {
      loading.value = false
    }
  }

  async function deletePrompt(id: string) {
    loading.value = true
    error.value = null
    try {
      await api.deletePrompt(id)
      prompts.value = prompts.value.filter((p) => p.id !== id)
      if (currentPrompt.value?.id === id) {
        currentPrompt.value = null
      }
      toast.success('Prompt deleted')
    } catch (e) {
      error.value = String(e)
      toast.error('Failed to delete prompt')
    } finally {
      loading.value = false
    }
  }

  async function archivePrompt(id: string, archive: boolean) {
    loading.value = true
    error.value = null
    try {
      await api.archivePrompt(id, archive)
      toast.success(archive ? 'Prompt archived' : 'Prompt restored')
      await fetchPrompts()
    } catch (e) {
      error.value = String(e)
      toast.error('Failed to archive prompt')
    } finally {
      loading.value = false
    }
  }

  async function fetchVersions(promptId: string) {
    loading.value = true
    error.value = null
    try {
      versions.value = await api.getPromptVersions(promptId)
    } catch (e) {
      error.value = String(e)
      toast.error('Failed to load versions')
    } finally {
      loading.value = false
    }
  }

  async function restoreVersion(promptId: string, versionNumber: number) {
    loading.value = true
    error.value = null
    try {
      await api.restorePromptVersion(promptId, versionNumber)
      toast.success(`Restored to v${versionNumber}`)
      await fetchPrompt(promptId)
      await fetchVersions(promptId)
    } catch (e) {
      error.value = String(e)
      toast.error('Failed to restore version')
    } finally {
      loading.value = false
    }
  }

  return {
    prompts,
    currentPrompt,
    versions,
    loading,
    error,
    fetchPrompts,
    fetchPrompt,
    createPrompt,
    updatePrompt,
    deletePrompt,
    archivePrompt,
    fetchVersions,
    restoreVersion,
  }
})
