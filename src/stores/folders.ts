import { ref } from 'vue'
import { defineStore } from 'pinia'
import { toast } from 'vue-sonner'
import type { Folder, FolderTreeNode } from '@/types'
import * as api from '@/lib/tauri-api'

export const useFoldersStore = defineStore('folders', () => {
  const folders = ref<Folder[]>([])
  const folderTree = ref<FolderTreeNode[]>([])
  const loading = ref(false)

  async function fetchFolders() {
    loading.value = true
    try {
      folders.value = await api.listFolders()
    } catch (e) {
      toast.error('Failed to load folders: ' + String(e))
    } finally {
      loading.value = false
    }
  }

  async function fetchTree() {
    loading.value = true
    try {
      folderTree.value = await api.getFolderTree()
    } catch (e) {
      toast.error('Failed to load folder tree: ' + String(e))
    } finally {
      loading.value = false
    }
  }

  async function createFolder(name: string, parentId?: string | null) {
    try {
      const folder = await api.createFolder(name, parentId)
      toast.success('Folder created')
      await fetchTree()
      return folder
    } catch (e) {
      toast.error('Failed to create folder: ' + String(e))
    }
  }

  async function renameFolder(id: string, name: string) {
    try {
      await api.renameFolder(id, name)
      toast.success('Folder renamed')
      await fetchTree()
    } catch (e) {
      toast.error('Failed to rename folder: ' + String(e))
    }
  }

  async function deleteFolder(id: string) {
    try {
      await api.deleteFolder(id)
      toast.success('Folder deleted')
      await fetchTree()
    } catch (e) {
      toast.error('Failed to delete folder: ' + String(e))
    }
  }

  async function movePromptToFolder(promptId: string, folderId: string | null) {
    try {
      await api.moveToFolder(promptId, folderId)
      toast.success('Prompt moved')
    } catch (e) {
      toast.error('Failed to move prompt: ' + String(e))
    }
  }

  return {
    folders,
    folderTree,
    loading,
    fetchFolders,
    fetchTree,
    createFolder,
    renameFolder,
    deleteFolder,
    movePromptToFolder,
  }
})
