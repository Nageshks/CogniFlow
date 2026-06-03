import { ref, onUnmounted } from 'vue'

export function useAutoSave(saveFn: () => Promise<void>, defaultDelay = 30000) {
  const isDirty = ref(false)
  const lastSaved = ref<Date | null>(null)
  let timer: ReturnType<typeof setTimeout> | null = null

  function trigger() {
    isDirty.value = true
    if (timer) clearTimeout(timer)
    timer = setTimeout(async () => {
      await save()
    }, defaultDelay)
  }

  async function save() {
    if (timer) clearTimeout(timer)
    try {
      await saveFn()
      lastSaved.value = new Date()
      isDirty.value = false
    } catch (e) {
      console.error('Auto save failed:', e)
    }
  }

  onUnmounted(() => {
    if (timer) clearTimeout(timer)
  })

  return {
    isDirty,
    lastSaved,
    trigger,
    save,
  }
}
