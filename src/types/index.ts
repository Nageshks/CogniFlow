// ===== Prompt Models =====

export interface Prompt {
  id: string
  title: string
  content: string
  description: string
  folder_id: string | null
  created_at: string
  updated_at: string
  archived: boolean
  current_version: number
}

export interface PromptVersion {
  id: string
  prompt_id: string
  version_number: number
  content: string
  change_summary: string
  created_at: string
}

/** Flattened: Prompt fields + tags/categories (mirrors serde flatten) */
export interface PromptWithMeta {
  id: string
  title: string
  content: string
  description: string
  folder_id: string | null
  created_at: string
  updated_at: string
  archived: boolean
  current_version: number
  tags: string[]
  categories: string[]
}

export interface CreatePromptRequest {
  title: string
  content: string
  description: string
  folder_id?: string | null
  tags: string[]
  category?: string | null
}

export interface UpdatePromptRequest {
  id: string
  title?: string | null
  content?: string | null
  description?: string | null
  folder_id?: string | null
  change_summary: string
}

// ===== Workflow Models =====

export interface Workflow {
  id: string
  title: string
  description?: string
  intent?: string
  folder_id: string | null
  created_at: string
  updated_at: string
}

export interface WorkflowStep {
  id: string
  workflow_id: string
  order_index: number
  title: string
  instruction: string
  prompt_id: string | null
}

/** Flattened: Workflow fields + steps (mirrors serde flatten) */
export interface WorkflowWithSteps extends Workflow {
  steps: WorkflowStep[]
}

export interface CreateWorkflowRequest {
  title: string
  description?: string | null
  intent?: string | null
  folder_id?: string | null
}

export interface UpdateWorkflowRequest {
  id: string
  title?: string | null
  description?: string | null
  intent?: string | null
  folder_id?: string | null
}

export interface CreateWorkflowStepRequest {
  workflow_id: string
  order_index: number
  title: string
  instruction: string
  prompt_id?: string | null
}

export interface UpdateWorkflowStepRequest {
  id: string
  title?: string | null
  instruction?: string | null
  prompt_id?: string | null
}

export interface ReorderStepsRequest {
  workflow_id: string
  /** Array of [step_id, new_order_index] tuples */
  order: [string, number][]
}

// ===== Folder Models =====

export interface Folder {
  id: string
  name: string
  parent_id: string | null
  depth: number
}

export interface FolderTreeNode {
  id: string
  name: string
  parent_id: string | null
  depth: number
  children: FolderTreeNode[]
  prompt_count: number
}

// ===== Taxonomy Models =====

export interface Tag {
  id: string
  name: string
}

export interface Category {
  id: string
  name: string
}

export interface Intent {
  id: string
  name: string
}

export interface IntentClassification {
  intent: string
  confidence: number
}

export interface AiSuggestions {
  tags: string[]
  category: string | null
  intent: IntentClassification | null
}

// ===== Relationship / Graph Models =====

export type RelationshipType = 'SIMILAR' | 'DERIVED_FROM' | 'USED_IN' | 'RELATED_TO'

export interface Relationship {
  id: string
  source_id: string
  target_id: string
  relationship_type: RelationshipType
  weight: number
}

export interface GraphNode {
  id: string
  label: string
  node_type: string
  metadata: Record<string, unknown>
}

export interface GraphEdge {
  id: string
  source: string
  target: string
  edge_type: string
  weight: number
}

export interface GraphData {
  nodes: GraphNode[]
  edges: GraphEdge[]
}

// ===== Search Models =====

export type SearchMode = 'keyword' | 'semantic' | 'hybrid'

export interface SearchResult {
  id: string
  entity_type: string
  title: string
  snippet: string
  score: number
}

export interface HybridSearchConfig {
  semantic_weight: number
  keyword_weight: number
}

export interface SearchQuery {
  query: string
  mode: SearchMode
  limit: number
  config?: HybridSearchConfig | null
}

// ===== Import / Export Models =====

export type ExportFormat = 'json' | 'markdown'
export type ImportFormat = 'json' | 'markdown' | 'txt'

export interface ExportRequest {
  format: ExportFormat
  path: string
  expanded: boolean
}

export interface ImportRequest {
  format: ImportFormat
  path: string
}

// ===== Stats =====

export interface DataStats {
  total_prompts: number
  total_workflows: number
  total_tags: number
  total_folders: number
  total_relationships: number
  archived_prompts: number
}
