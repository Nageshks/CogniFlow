import { ref } from 'vue'
import { defineStore } from 'pinia'
import { toast } from 'vue-sonner'
import type { GraphData, GraphNode } from '@/types'
import * as api from '@/lib/tauri-api'

export const useGraphStore = defineStore('graph', () => {
  const graphData = ref<GraphData>({ nodes: [], edges: [] })
  const selectedNode = ref<GraphNode | null>(null)
  const loading = ref(false)

  async function fetchGraphData() {
    loading.value = true
    try {
      graphData.value = await api.getGraphData()
    } catch (e) {
      toast.error('Failed to load graph data: ' + String(e))
    } finally {
      loading.value = false
    }
  }

  function selectNode(node: GraphNode | null) {
    selectedNode.value = node
  }

  async function fetchNodeRelationships(nodeId: string) {
    try {
      return await api.getNodeRelationships(nodeId)
    } catch (e) {
      toast.error('Failed to load relationships: ' + String(e))
      return []
    }
  }

  return {
    graphData,
    selectedNode,
    loading,
    fetchGraphData,
    selectNode,
    fetchNodeRelationships,
  }
})
