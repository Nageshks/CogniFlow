<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { Search } from 'lucide-vue-next'
import { useSearchStore } from '@/stores/search'

const emit = defineEmits<{
  (e: 'focus'): void
  (e: 'blur'): void
}>()

const searchStore = useSearchStore()
const inputRef = ref<HTMLInputElement | null>(null)

// Debounced search
let debounceTimer: ReturnType<typeof setTimeout> | null = null
function handleInput() {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => {
    searchStore.search()
  }, 300)
}

function handleKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
    e.preventDefault()
    inputRef.value?.focus()
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div class="search-container flex items-center w-full">
    <!-- Search: max-w-md width, 40px height, bg #F3F4F6, border 1px #E5E7EB, radius 10px -->
    <div class="flex items-center bg-[#F3F4F6] border border-[#E5E7EB] rounded-[10px] px-3.5 h-10 w-full max-w-md focus-within:border-gray-400 transition-colors">
      <Search :size="16" class="text-[#6B7280] mr-2 flex-shrink-0" />
      <input
        ref="inputRef"
        v-model="searchStore.query"
        type="text"
        placeholder="Type to search prompts, tags, categories..."
        class="w-full text-[13px] bg-transparent border-none outline-none text-[#111827] placeholder-[#9CA3AF] h-full"
        @input="handleInput"
        @focus="emit('focus')"
        @blur="emit('blur')"
      />
    </div>
  </div>
</template>

<style scoped>
.search-container {
  position: relative;
}
</style>
