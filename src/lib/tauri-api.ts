import { invoke } from '@tauri-apps/api/core'
import type {
  Prompt,
  PromptWithMeta,
  PromptVersion,
  CreatePromptRequest,
  UpdatePromptRequest,
  Workflow,
  WorkflowWithSteps,
  WorkflowStep,
  CreateWorkflowRequest,
  UpdateWorkflowRequest,
  CreateWorkflowStepRequest,
  UpdateWorkflowStepRequest,
  ReorderStepsRequest,
  SearchResult,
  HybridSearchConfig,
  AiSuggestions,
  GraphData,
  Relationship,
  RelationshipType,
  Folder,
  FolderTreeNode,
  ExportRequest,
  ImportRequest,
  DataStats,
} from '@/types'

// ===== Prompts =====

export async function createPrompt(request: CreatePromptRequest): Promise<Prompt> {
  return invoke<Prompt>('create_prompt', { request })
}

export async function getPrompt(id: string): Promise<PromptWithMeta> {
  return invoke<PromptWithMeta>('get_prompt', { id })
}

export async function listPrompts(
  folderId?: string | null,
  archived?: boolean
): Promise<PromptWithMeta[]> {
  return invoke<PromptWithMeta[]>('list_prompts', { folderId, archived })
}

export async function updatePrompt(request: UpdatePromptRequest): Promise<Prompt> {
  return invoke<Prompt>('update_prompt', { request })
}

export async function deletePrompt(id: string): Promise<void> {
  return invoke<void>('delete_prompt', { id })
}

export async function archivePrompt(id: string, archive: boolean): Promise<void> {
  return invoke<void>('archive_prompt', { id, archive })
}

export async function getPromptVersions(promptId: string): Promise<PromptVersion[]> {
  return invoke<PromptVersion[]>('get_prompt_versions', { promptId })
}

export async function restorePromptVersion(promptId: string, versionNumber: number): Promise<Prompt> {
  return invoke<Prompt>('restore_prompt_version', { promptId, versionNumber })
}

// ===== Workflows =====

export async function createWorkflow(request: CreateWorkflowRequest): Promise<Workflow> {
  return invoke<Workflow>('create_workflow', { request })
}

export async function getWorkflow(id: string): Promise<WorkflowWithSteps> {
  return invoke<WorkflowWithSteps>('get_workflow', { id })
}

export async function listWorkflows(): Promise<WorkflowWithSteps[]> {
  return invoke<WorkflowWithSteps[]>('list_workflows')
}

export async function updateWorkflow(request: UpdateWorkflowRequest): Promise<Workflow> {
  return invoke<Workflow>('update_workflow', { request })
}

export async function deleteWorkflow(id: string): Promise<void> {
  return invoke<void>('delete_workflow', { id })
}

export async function addWorkflowStep(request: CreateWorkflowStepRequest): Promise<WorkflowStep> {
  return invoke<WorkflowStep>('add_workflow_step', { request })
}

export async function updateWorkflowStep(request: UpdateWorkflowStepRequest): Promise<WorkflowStep> {
  return invoke<WorkflowStep>('update_workflow_step', { request })
}

export async function deleteWorkflowStep(id: string): Promise<void> {
  return invoke<void>('delete_workflow_step', { id })
}

export async function reorderWorkflowSteps(request: ReorderStepsRequest): Promise<void> {
  return invoke<void>('reorder_workflow_steps', { request })
}

export async function linkPromptToStep(stepId: string, promptId: string | null): Promise<void> {
  return invoke<void>('link_prompt_to_step', { stepId, promptId })
}

export async function getWorkflowsUsingPrompt(promptId: string): Promise<Workflow[]> {
  return invoke<Workflow[]>('get_workflows_using_prompt', { promptId })
}

// ===== Search =====

export async function searchFulltext(query: string, limit: number): Promise<SearchResult[]> {
  return invoke<SearchResult[]>('search_fulltext', { query, limit })
}

export async function searchSemantic(query: string, limit: number): Promise<SearchResult[]> {
  return invoke<SearchResult[]>('search_semantic', { query, limit })
}

export async function searchHybrid(
  query: string,
  limit: number,
  config?: HybridSearchConfig
): Promise<SearchResult[]> {
  return invoke<SearchResult[]>('search_hybrid', { query, limit, config })
}

// ===== AI =====

export async function suggestTags(content: string): Promise<string[]> {
  return invoke<string[]>('suggest_tags', { content })
}

export async function suggestCategory(content: string): Promise<string | null> {
  return invoke<string | null>('suggest_category', { content })
}

export async function suggestIntent(content: string): Promise<AiSuggestions> {
  return invoke<AiSuggestions>('suggest_intent', { content })
}

export async function getSimilarAssets(id: string, limit: number): Promise<SearchResult[]> {
  return invoke<SearchResult[]>('get_similar_assets', { id, limit })
}

// ===== Graph =====

export async function getGraphData(): Promise<GraphData> {
  return invoke<GraphData>('get_graph_data')
}

export async function getNodeRelationships(nodeId: string): Promise<Relationship[]> {
  return invoke<Relationship[]>('get_node_relationships', { nodeId })
}

export async function addRelationship(
  sourceId: string,
  targetId: string,
  relationshipType: RelationshipType,
  weight: number
): Promise<Relationship> {
  return invoke<Relationship>('add_relationship', {
    sourceId,
    targetId,
    relationshipType,
    weight,
  })
}

export async function removeRelationship(id: string): Promise<void> {
  return invoke<void>('remove_relationship', { id })
}

// ===== Folders =====

export async function createFolder(
  name: string,
  parentId?: string | null
): Promise<Folder> {
  return invoke<Folder>('create_folder', { name, parentId })
}

export async function renameFolder(id: string, name: string): Promise<Folder> {
  return invoke<Folder>('rename_folder', { id, name })
}

export async function deleteFolder(id: string): Promise<void> {
  return invoke<void>('delete_folder_cmd', { id })
}

export async function moveToFolder(promptId: string, folderId: string | null): Promise<void> {
  return invoke<void>('move_to_folder_cmd', { promptId, folderId })
}

export async function listFolders(): Promise<Folder[]> {
  return invoke<Folder[]>('list_folders_cmd')
}

export async function getFolderTree(): Promise<FolderTreeNode[]> {
  return invoke<FolderTreeNode[]>('get_folder_tree_cmd')
}

// ===== Import / Export =====

export async function exportData(request: ExportRequest): Promise<string> {
  return invoke<string>('export_data_cmd', { request })
}

export async function importData(request: ImportRequest): Promise<number> {
  return invoke<number>('import_data_cmd', { request })
}

// ===== Stats =====

export async function getStats(): Promise<DataStats> {
  return invoke<DataStats>('get_stats_cmd')
}

