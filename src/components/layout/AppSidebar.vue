<script setup lang="ts">
import { ref } from 'vue'
import { useRoute } from 'vue-router'
import { Library, GitBranch, Settings } from 'lucide-vue-next'

const route = useRoute()
const collapsed = ref(false)

const navItems = [
  { path: '/library', label: 'Library', icon: Library },
  { path: '/workflows', label: 'Workflows', icon: GitBranch },
  { path: '/settings', label: 'Settings', icon: Settings },
]

function isActive(path: string): boolean {
  return route.path.startsWith(path)
}

function toggleCollapse() {
  collapsed.value = !collapsed.value
}
</script>

<template>
  <aside id="app-sidebar" class="app-sidebar select-none pt-2" :class="{ collapsed }">
    <!-- Logo & Collapser Block -->
    <div class="sidebar-logo flex items-center justify-between px-4 pt-5 pb-3 mb-4" :class="{ 'justify-center': collapsed }">
      <!-- Inner Logo Group has the drag region -->
      <div v-if="!collapsed" class="flex items-center gap-3 cursor-default select-none animate-slide-left" data-tauri-drag-region>
        <!-- 32px black rounded square icon -->
        <div class="logo-icon w-8 h-8 rounded-lg flex items-center justify-center pointer-events-none bg-[#111827]">
          <div class="w-4.5 h-4.5 rounded-full flex items-center justify-center border border-white/90">
            <div class="w-1.5 h-1.5 rounded-full bg-white/90"></div>
          </div>
        </div>
        <!-- "CogniFlow" 15px/600 -->
        <span class="font-semibold text-text-primary pointer-events-none" style="font-size: 15px;">
          CogniFlow
        </span>
      </div>
      
      <!-- Collapsed Logo Icon -->
      <div v-else class="logo-icon w-8 h-8 rounded-lg flex items-center justify-center cursor-pointer transition-transform hover:scale-[1.03] bg-[#111827]" @click="toggleCollapse" title="Expand Sidebar">
        <div class="w-4.5 h-4.5 rounded-full flex items-center justify-center border border-white/90">
          <div class="w-1.5 h-1.5 rounded-full bg-white/90"></div>
        </div>
      </div>

      <!-- Collapser Button in logo block -->
      <button
        v-if="!collapsed"
        id="sidebar-toggle"
        class="p-1 rounded-lg hover:bg-bg-elevated transition-colors cursor-pointer"
        style="color: var(--color-text-muted);"
        @click="toggleCollapse"
        title="Collapse Sidebar"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
          <line x1="9" y1="3" x2="9" y2="21"/>
        </svg>
      </button>
    </div>

    <!-- Navigation -->
    <nav class="flex-1 px-3 py-2 flex flex-col gap-1">
      <router-link
        v-for="item in navItems"
        :key="item.path"
        :id="`nav-${item.label.toLowerCase()}`"
        :to="item.path"
        class="nav-item relative"
        :class="{ active: isActive(item.path) }"
        :title="collapsed ? item.label : undefined"
      >
        <!-- Icon size exactly 18px -->
        <component :is="item.icon" :size="18" />
        <span v-if="!collapsed" class="font-medium" style="font-size: 14px;">{{ item.label }}</span>
      </router-link>
    </nav>
  </aside>
</template>
