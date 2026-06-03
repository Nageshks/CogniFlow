<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { FileText, GitBranch, Download, Tag, Share2, Plus, Search } from 'lucide-vue-next'
import { usePromptsStore } from '@/stores/prompts'
import type { DataStats } from '@/types'
import * as api from '@/lib/tauri-api'
import { formatDate } from '@/lib/utils'

const router = useRouter()
const promptsStore = usePromptsStore()
const stats = ref<DataStats | null>(null)

onMounted(async () => {
  try {
    stats.value = await api.getStats()
  } catch {
    // Backend not ready yet
  }
  await promptsStore.fetchPrompts()
})

function navigateNewPrompt() {
  window.dispatchEvent(new CustomEvent('open-new-prompt'))
  router.push('/library')
}

function navigateNewWorkflow() {
  window.dispatchEvent(new CustomEvent('open-new-workflow'))
  router.push('/workflows')
}

function navigateImport() {
  router.push('/settings')
}

function navigateToPrompt(id: string) {
  router.push(`/library`)
  void id
}
</script>

<template>
  <div class="home-view animate-fade-in flex flex-col gap-6 w-full py-1">
    <!-- Premium bold page heading -->
    <div class="flex flex-col gap-1.5 select-none">
      <h1 class="text-3xl font-extrabold tracking-tight" style="font-family: var(--font-display); background: linear-gradient(135deg, var(--color-accent-blue), var(--color-accent-violet)); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;">
        Home
      </h1>
      <p class="text-xs font-medium" style="color: var(--color-text-secondary);">
        Discover, manage, and run your local private cognitive assets.
      </p>
    </div>

    <!-- Spotlight Search Portal Widget -->
    <section>
      <div 
        class="spotlight-search-portal relative flex items-center cursor-pointer" 
        @click="router.push('/library')" 
        role="button" 
        tabindex="0"
      >
        <Search :size="16" style="color: var(--color-text-muted); flex-shrink: 0;" class="mr-3" />
        <span class="text-text-secondary text-xs font-semibold">Search prompts, workflows, relations...</span>
        
        <div class="ml-auto flex items-center gap-1.5 pointer-events-none select-none">
          <kbd class="px-2 py-0.5 bg-bg-elevated border border-border-default rounded text-[9px] font-sans text-text-muted">Ctrl</kbd>
          <span class="text-text-muted text-[9px]">+</span>
          <kbd class="px-2 py-0.5 bg-bg-elevated border border-border-default rounded text-[9px] font-sans text-text-muted">K</kbd>
        </div>
      </div>
    </section>

    <!-- Hero Portal Card -->
    <section class="glass-card p-6 flex items-center justify-between border border-border-default shadow-sm rounded-xl relative overflow-hidden" style="background: linear-gradient(135deg, var(--color-bg-glass), color-mix(in srgb, var(--color-bg-tertiary) 40%, transparent));">
      <div class="absolute -right-16 -top-16 w-48 h-48 rounded-full blur-3xl opacity-10" style="background: var(--color-accent-blue);"></div>
      <div class="absolute -right-8 -bottom-8 w-32 h-32 rounded-full blur-3xl opacity-10" style="background: var(--color-accent-violet);"></div>

      <div class="flex items-center gap-6 max-w-3xl relative z-10">
        <div class="flex-shrink-0 w-14 h-14 rounded-xl flex items-center justify-center border animate-pulse-glow" style="background: var(--color-bg-elevated); border-color: var(--color-border-default);">
          <svg width="34" height="34" viewBox="0 0 28 28" fill="none">
            <defs>
              <linearGradient id="logo-grad-home" x1="0" y1="0" x2="28" y2="28">
                <stop offset="0%" stop-color="var(--color-accent-blue)" />
                <stop offset="100%" stop-color="var(--color-accent-violet)" />
              </linearGradient>
            </defs>
            <circle cx="14" cy="14" r="13" stroke="url(#logo-grad-home)" stroke-width="2" fill="none" />
            <path d="M9 14 C9 10, 14 7, 14 14 C14 7, 19 10, 19 14 C19 18, 14 21, 14 14 C14 21, 9 18, 9 14Z" fill="url(#logo-grad-home)" />
          </svg>
        </div>
        <div>
          <h2 class="text-base font-bold mb-1.5" style="font-family: var(--font-display); background: linear-gradient(135deg, var(--color-accent-blue), var(--color-accent-violet)); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;">
            CogniFlow Laboratory
          </h2>
          <p style="color: var(--color-text-secondary);" class="text-xs leading-relaxed max-w-2xl">
            Welcome to your local cognitive memory center. Construct prompt chains, organize reusable templates, and explore automatically discovered relations within your secure knowledge index.
          </p>
        </div>
      </div>
    </section>

    <!-- Stats Cards Deck - Premium Glow and Hover lifting -->
    <section v-if="stats" class="animate-fade-in">
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
        <!-- Prompts Stats -->
        <div 
          class="stat-card-premium flex items-center gap-5 p-5 hover:scale-[1.02] transition-all cursor-pointer" 
          style="--card-glow-color: var(--color-accent-blue); --card-glow-shadow: var(--shadow-glow-blue);"
          @click="router.push('/library')"
        >
          <div 
            class="w-12 h-12 rounded-xl flex items-center justify-center flex-shrink-0 border"
            style="background: color-mix(in srgb, var(--color-accent-blue) 10%, transparent); color: var(--color-accent-blue); border-color: color-mix(in srgb, var(--color-accent-blue) 20%, transparent);"
          >
            <FileText :size="20" />
          </div>
          <div>
            <div class="stat-value text-2xl font-extrabold tracking-tight" style="font-family: var(--font-display); color: var(--color-text-primary);">
              {{ promptsStore.prompts.length || stats.total_prompts }}
            </div>
            <div class="text-[9px] font-bold mt-0.5 tracking-wider uppercase" style="color: var(--color-text-muted);">
              Prompts Index
            </div>
          </div>
        </div>

        <!-- Workflows Stats -->
        <div 
          class="stat-card-premium flex items-center gap-5 p-5 hover:scale-[1.02] transition-all cursor-pointer" 
          style="--card-glow-color: var(--color-accent-violet); --card-glow-shadow: var(--shadow-glow-violet);"
          @click="router.push('/workflows')"
        >
          <div 
            class="w-12 h-12 rounded-xl flex items-center justify-center flex-shrink-0 border"
            style="background: color-mix(in srgb, var(--color-accent-violet) 10%, transparent); color: var(--color-accent-violet); border-color: color-mix(in srgb, var(--color-accent-violet) 20%, transparent);"
          >
            <GitBranch :size="20" />
          </div>
          <div>
            <div class="stat-value text-2xl font-extrabold tracking-tight" style="font-family: var(--font-display); color: var(--color-text-primary);">
              {{ stats.total_workflows }}
            </div>
            <div class="text-[9px] font-bold mt-0.5 tracking-wider uppercase" style="color: var(--color-text-muted);">
              Workchains
            </div>
          </div>
        </div>

        <!-- Tags Stats -->
        <div 
          class="stat-card-premium flex items-center gap-5 p-5 hover:scale-[1.02] transition-all cursor-pointer" 
          style="--card-glow-color: var(--color-accent-emerald); --card-glow-shadow: 0 0 20px hsl(160 84% 39% / 0.1);"
          @click="router.push('/library')"
        >
          <div 
            class="w-12 h-12 rounded-xl flex items-center justify-center flex-shrink-0 border"
            style="background: color-mix(in srgb, var(--color-accent-emerald) 10%, transparent); color: var(--color-accent-emerald); border-color: color-mix(in srgb, var(--color-accent-emerald) 20%, transparent);"
          >
            <Tag :size="20" />
          </div>
          <div>
            <div class="stat-value text-2xl font-extrabold tracking-tight" style="font-family: var(--font-display); color: var(--color-text-primary);">
              {{ stats.total_tags }}
            </div>
            <div class="text-[9px] font-bold mt-0.5 tracking-wider uppercase" style="color: var(--color-text-muted);">
              Tags Catalog
            </div>
          </div>
        </div>

        <!-- Relationships Stats -->
        <div 
          class="stat-card-premium flex items-center gap-5 p-5 hover:scale-[1.02] transition-all cursor-pointer" 
          style="--card-glow-color: var(--color-accent-amber); --card-glow-shadow: 0 0 20px hsl(38 92% 50% / 0.1);"
          @click="router.push('/graph')"
        >
          <div 
            class="w-12 h-12 rounded-xl flex items-center justify-center flex-shrink-0 border"
            style="background: color-mix(in srgb, var(--color-accent-amber) 10%, transparent); color: var(--color-accent-amber); border-color: color-mix(in srgb, var(--color-accent-amber) 20%, transparent);"
          >
            <Share2 :size="20" />
          </div>
          <div>
            <div class="stat-value text-2xl font-extrabold tracking-tight" style="font-family: var(--font-display); color: var(--color-text-primary);">
              {{ stats.total_relationships }}
            </div>
            <div class="text-[9px] font-bold mt-0.5 tracking-wider uppercase" style="color: var(--color-text-muted);">
              Graph Nodes
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Quick Actions Grid -->
    <section>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
        <button id="qa-new-prompt" class="quick-action p-5 hover:scale-[1.02] transition-all relative overflow-hidden" @click="navigateNewPrompt">
          <div class="action-icon blue flex-shrink-0 w-10 h-10 rounded-xl flex items-center justify-center border border-transparent shadow-sm">
            <FileText :size="18" />
          </div>
          <div class="text-left ml-2">
            <div class="font-bold text-sm" style="color: var(--color-text-primary);">New Prompt</div>
            <div class="text-xs mt-0.5" style="color: var(--color-text-muted);">Construct cognitive prompt asset</div>
          </div>
        </button>

        <button id="qa-new-workflow" class="quick-action p-5 hover:scale-[1.02] transition-all relative overflow-hidden" @click="navigateNewWorkflow">
          <div class="action-icon violet flex-shrink-0 w-10 h-10 rounded-xl flex items-center justify-center border border-transparent shadow-sm">
            <GitBranch :size="18" />
          </div>
          <div class="text-left ml-2">
            <div class="font-bold text-sm" style="color: var(--color-text-primary);">New Workflow</div>
            <div class="text-xs mt-0.5" style="color: var(--color-text-muted);">Chain prompts sequence</div>
          </div>
        </button>

        <button id="qa-import" class="quick-action p-5 hover:scale-[1.02] transition-all relative overflow-hidden" @click="navigateImport">
          <div class="action-icon emerald flex-shrink-0 w-10 h-10 rounded-xl flex items-center justify-center border border-transparent shadow-sm">
            <Download :size="18" />
          </div>
          <div class="text-left ml-2">
            <div class="font-bold text-sm" style="color: var(--color-text-primary);">Import Data</div>
            <div class="text-xs mt-0.5" style="color: var(--color-text-muted);">Load JSON, Markdown, TXT</div>
          </div>
        </button>
      </div>
    </section>

    <!-- Recent Items Section -->
    <section class="mt-2">
      <div class="section-header mb-4 select-none">
        <h2 class="section-title text-sm font-bold tracking-tight">Recent Prompts</h2>
        <button class="section-action text-xs font-semibold hover:underline" @click="router.push('/library')">View catalog →</button>
      </div>

      <!-- Prompts Grid -->
      <div v-if="promptsStore.prompts.length > 0" class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div
          v-for="prompt in promptsStore.prompts.slice(0, 4)"
          :key="prompt.id"
          :id="`recent-${prompt.id}`"
          class="prompt-card p-5"
          @click="navigateToPrompt(prompt.id)"
        >
          <div class="card-title">{{ prompt.title }}</div>
          <div class="card-description mt-1.5 leading-relaxed">{{ prompt.description || prompt.content }}</div>
          <div class="card-meta mt-4">
            <span v-for="tag in prompt.tags.slice(0, 3)" :key="tag" class="tag">{{ tag }}</span>
            <span class="ml-auto text-[10px] font-medium" style="color: var(--color-text-muted);">
              {{ formatDate(prompt.updated_at) }}
            </span>
          </div>
        </div>
      </div>

      <!-- Centered Empty State -->
      <div v-else class="empty-state border border-border-default border-dashed rounded-xl bg-bg-glass backdrop-blur-sm p-12 flex flex-col items-center justify-center text-center" style="min-height: 260px;">
        <FileText :size="40" class="empty-icon text-text-muted opacity-30 mb-3 animate-pulse-glow" />
        <div class="empty-title text-sm font-bold" style="color: var(--color-text-primary);">No prompts cataloged yet</div>
        <div class="empty-description text-xs max-w-sm mb-4 leading-relaxed" style="color: var(--color-text-muted);">
          Create your first cognitive prompt to initiate your private memory indexing system.
        </div>
        <button
          id="home-create-first"
          class="flex items-center gap-1.5 px-4.5 py-2.5 rounded-lg text-xs font-semibold transition-all cursor-pointer shadow-sm hover:scale-[1.02] border border-transparent"
          style="background: var(--color-accent-blue); color: white;"
          @click="navigateNewPrompt"
        >
          <Plus :size="14" />
          <span>Create First Prompt</span>
        </button>
      </div>
    </section>
  </div>
</template>
