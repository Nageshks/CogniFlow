<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'

const props = defineProps<{
  open: boolean
  title: string
  placeholder?: string
  initialValue?: string
  confirmText?: string
}>()

const emit = defineEmits<{
  (e: 'update:open', val: boolean): void
  (e: 'confirm', val: string): void
  (e: 'cancel'): void
}>()

const inputValue = ref('')
const inputRef = ref<HTMLInputElement | null>(null)

watch(() => props.open, (newVal) => {
  if (newVal) {
    inputValue.value = props.initialValue || ''
    nextTick(() => {
      inputRef.value?.focus()
      inputRef.value?.select()
    })
  }
})

function onConfirm() {
  if (!inputValue.value.trim()) return
  emit('confirm', inputValue.value.trim())
  emit('update:open', false)
}

function onCancel() {
  emit('cancel')
  emit('update:open', false)
}
</script>

<template>
  <Dialog :open="open" @update:open="onCancel">
    <DialogContent class="bg-white rounded-2xl shadow-xl border border-gray-100 sm:max-w-[400px] w-full p-0 overflow-hidden flex flex-col dark:bg-zinc-950 dark:border-zinc-850" :showCloseButton="false">
      <DialogHeader class="px-6 py-5 border-b border-gray-100 dark:border-zinc-850">
        <DialogTitle class="font-bold text-gray-900 text-base tracking-tight dark:text-gray-100 text-left">{{ title }}</DialogTitle>
      </DialogHeader>
      
      <div class="p-6 flex flex-col gap-4">
        <input
          ref="inputRef"
          v-model="inputValue"
          type="text"
          :placeholder="placeholder"
          class="w-full h-10 px-4 py-3 rounded-xl bg-white border border-gray-200 text-gray-900 focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 outline-none transition-all shadow-sm text-sm dark:bg-zinc-900 dark:border-zinc-800 dark:text-gray-100"
          @keyup.enter="onConfirm"
          @keyup.escape="onCancel"
        />
      </div>

      <DialogFooter class="px-6 py-4 bg-gray-50/80 border-t border-gray-100 flex items-center justify-end gap-2.5 dark:bg-zinc-900/50 dark:border-zinc-850">
        <Button variant="ghost" class="px-4 py-2 h-9 rounded-xl text-sm font-semibold text-gray-500 hover:text-gray-900 hover:bg-gray-200 transition-colors cursor-pointer dark:hover:text-zinc-200" @click="onCancel">
          Cancel
        </Button>
        <Button class="px-5 py-2 h-9 rounded-xl bg-gray-900 hover:bg-gray-800 text-white text-sm font-semibold shadow-sm transition-colors cursor-pointer disabled:opacity-50 dark:bg-zinc-100 dark:text-zinc-950 dark:hover:bg-zinc-200" :disabled="!inputValue.trim()" @click="onConfirm">
          {{ confirmText || 'Confirm' }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
