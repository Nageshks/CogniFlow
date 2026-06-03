import { ref } from 'vue'
import { defineStore } from 'pinia'
import { toast } from 'vue-sonner'
import type { SearchResult, SearchMode, HybridSearchConfig } from '@/types'
import * as api from '@/lib/tauri-api'

export const useSearchStore = defineStore('search', () => {
  const query = ref('')
  const results = ref<SearchResult[]>([])
  const mode = ref<SearchMode>('hybrid')
  const loading = ref(false)
  const config = ref<HybridSearchConfig>({
    semantic_weight: 0.7,
    keyword_weight: 0.3,
  })

  async function search(searchQuery?: string) {
    const q = searchQuery ?? query.value
    if (!q.trim()) {
      results.value = []
      return
    }

    loading.value = true
    try {
      switch (mode.value) {
        case 'keyword':
          results.value = await api.searchFulltext(q, 20)
          break
        case 'semantic':
          results.value = await api.searchSemantic(q, 20)
          break
        case 'hybrid':
          results.value = await api.searchHybrid(q, 20, config.value)
          break
      }
    } catch (e) {
      toast.error('Search failed: ' + String(e))
      results.value = []
    } finally {
      loading.value = false
    }
  }

  function setMode(newMode: SearchMode) {
    mode.value = newMode
    if (query.value.trim()) {
      search()
    }
  }

  function updateConfig(newConfig: Partial<HybridSearchConfig>) {
    config.value = { ...config.value, ...newConfig }
  }

  return {
    query,
    results,
    mode,
    loading,
    config,
    search,
    setMode,
    updateConfig,
  }
})
