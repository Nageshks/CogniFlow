<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { X, Copy, Check, Info } from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import type { PromptWithMeta } from '@/types'

const props = defineProps<{
  open: boolean
  prompt: PromptWithMeta | null
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const variableValues = ref<Record<string, string>>({})
const copied = ref(false)

// Extract unique variables: e.g. {{variable}}
const variables = computed(() => {
  if (!props.prompt) return []
  const content = props.prompt.content || ''
  const regex = /\{\{([^}]+)\}\}/g
  const matches: string[] = []
  let match
  while ((match = regex.exec(content)) !== null) {
    const varName = match[1].trim()
    if (varName && !matches.includes(varName)) {
      matches.push(varName)
    }
  }
  return matches
})

// Initialize inputs whenever the prompt changes
watch(
  () => props.prompt,
  (newPrompt) => {
    if (newPrompt) {
      const vals: Record<string, string> = {}
      variables.value.forEach((v) => {
        vals[v] = ''
      })
      variableValues.value = vals
      copied.value = false
    }
  },
  { immediate: true }
)

// Compile the template dynamically
const compiledContent = computed(() => {
  if (!props.prompt) return ''
  let result = props.prompt.content || ''
  variables.value.forEach((key) => {
    const val = variableValues.value[key] || `{{${key}}}`
    const escapedKey = key.replace(/[-\/\\^$*+?.()|[\]{}]/g, '\\$&')
    const regex = new RegExp(`\\{\\{\\s*${escapedKey}\\s*\\}\\}`, 'g')
    result = result.replace(regex, val)
  })
  return result
})

// Format and highlight variables in output
const formattedCompiledContent = computed(() => {
  if (!props.prompt) return ''
  let result = props.prompt.content || ''
  
  result = result
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;')

  variables.value.forEach((key) => {
    const rawVal = variableValues.value[key]
    let replacement = ''
    if (rawVal && rawVal.trim() !== '') {
      const escapedVal = rawVal
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#039;')
      replacement = `<span class="upd-var-highlight animate-pulse-subtle">${escapedVal}</span>`
    } else {
      replacement = `<span class="upd-var-placeholder animate-pulse-subtle">{{${key}}}</span>`
    }
    const escapedKey = key.replace(/[-\/\\^$*+?.()|[\]{}]/g, '\\$&')
    const regex = new RegExp(`\\{\\{\\s*${escapedKey}\\s*\\}\\}`, 'g')
    result = result.replace(regex, replacement)
  })
  return result
})

const firstInput = ref<HTMLInputElement | null>(null)
function setFirstInputRef(el: any) {
  firstInput.value = el
}

watch(
  () => props.open,
  async (isOpen) => {
    if (isOpen) {
      await nextTick()
      setTimeout(() => {
        firstInput.value?.focus()
      }, 150)
    }
  }
)

async function copyResult() {
  try {
    await navigator.clipboard.writeText(compiledContent.value)
    copied.value = true
    toast.success('Compiled prompt copied to clipboard!')
    
    if (props.prompt) {
      const currentCount = parseInt(localStorage.getItem(`usage-${props.prompt.id}`) || '0')
      localStorage.setItem(`usage-${props.prompt.id}`, (currentCount + 1).toString())
      window.dispatchEvent(new CustomEvent('prompt-used', { detail: { id: props.prompt.id } }))
    }

    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (e) {
    toast.error('Failed to copy: ' + String(e))
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="v">
      <div v-if="open && prompt" class="upd-overlay" @click="emit('close')">
        <div class="upd-dialog" @click.stop>
          
          <!-- Header -->
          <div class="upd-header">
            <h2 class="upd-dialog-title truncate max-w-[800px]">
              {{ prompt.title }}
            </h2>
            <button class="upd-close-btn" @click="emit('close')">
              <X :size="18" />
            </button>
          </div>

          <!-- Body Grid (Swapped: Preview on Left, Form on Right) -->
          <div class="upd-body">
            
            <!-- Left panel: Preview Output (60%) -->
            <div class="upd-preview-panel">
              <div class="upd-preview-box relative">
                <!-- Floating Copy Button -->
                <button
                  class="absolute top-3.5 right-4 z-10 flex items-center gap-2 text-xs font-bold cursor-pointer text-white bg-indigo-600 hover:bg-indigo-700 active:scale-[0.98] px-4 py-2.5 rounded-xl transition-all shadow-md"
                  @click="copyResult"
                >
                  <Check v-if="copied" :size="13" />
                  <Copy v-else :size="13" />
                  <span>{{ copied ? 'Copied!' : 'Copy Result' }}</span>
                </button>
                <pre class="upd-preview-text pt-12" v-html="formattedCompiledContent"></pre>
              </div>
            </div>

            <!-- Right panel: Variable Form (40%) -->
            <div class="upd-form-panel">
              <div class="upd-form-box">
                <h3 class="upd-panel-title">Variables</h3>
                
                <div v-if="variables.length > 0" class="upd-form-fields">
                  <div v-for="(variable, index) in variables" :key="variable" class="upd-field">
                    <label :for="'var-' + variable" class="upd-field-label">{{ variable }}</label>
                    <input
                      :id="'var-' + variable"
                      v-model="variableValues[variable]"
                      type="text"
                      :placeholder="'Enter ' + variable + '...'"
                      class="upd-field-input"
                      :ref="(el) => { if (index === 0) setFirstInputRef(el) }"
                    />
                  </div>
                </div>

                <!-- Empty state: No variables detected -->
                <div v-else class="upd-no-vars animate-fade-in">
                  <Info :size="24" class="text-blue-500" />
                  <p class="upd-no-vars-text">
                    No variables found in this template. The prompt content will copy as-is.
                  </p>
                  <div class="upd-tip">
                    Tip: Insert variables in prompt editor using curly braces like <code class="font-mono bg-gray-100 dark:bg-zinc-800 px-1 rounded">&#123;&#123;name&#125;&#125;</code>
                  </div>
                </div>
              </div>
            </div>

          </div>

        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.upd-overlay {
  position: fixed;
  inset: 0;
  z-index: 60; /* Raised from 50 to stack above PromptDetailSheet (z-50) */
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
}

.upd-dialog {
  background: var(--color-bg-primary);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-xl);
  width: 100%;
  max-width: 1100px;
  height: 85%;
  max-height: 750px;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-elevated);
  overflow: hidden;
  animation: fadeIn 200ms ease-out;
}

/* Header */
.upd-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 24px;
  border-bottom: 1px solid var(--color-border-default);
  background: var(--color-bg-secondary);
}

.upd-dialog-title {
  font-size: 17px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.upd-close-btn {
  background: transparent;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 4px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 150ms ease;
}

.upd-close-btn:hover {
  color: var(--color-text-primary);
  background: var(--color-bg-elevated);
}

/* Body split layout */
.upd-body {
  display: flex;
  flex: 1;
  min-height: 0;
  background: var(--color-bg-secondary);
}

.upd-preview-panel {
  width: 60%;
  display: flex;
  flex-direction: column;
  padding: 20px 10px 20px 20px;
  min-height: 0;
}

.upd-form-panel {
  width: 40%;
  display: flex;
  flex-direction: column;
  padding: 20px 20px 20px 10px;
  min-height: 0;
}

.upd-form-box {
  flex: 1;
  border-radius: var(--radius-lg);
  border: 1px solid var(--color-border-default);
  background: var(--color-bg-primary);
  padding: 20px;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow-y: auto;
}

.upd-panel-title {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--color-text-muted);
  margin-bottom: 16px;
}

/* Form items */
.upd-form-fields {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.upd-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.upd-field-label {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-text-secondary);
}

.upd-field-input {
  width: 100%;
  padding: 8px 12px;
  font-size: 13px;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-default);
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
  outline: none;
  transition: border-color 150ms ease;
}

.upd-field-input:focus {
  border-color: var(--color-text-muted);
}

/* Empty variables view */
.upd-no-vars {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 48px 16px;
  margin: auto 0;
  gap: 12px;
}

.upd-no-vars-text {
  font-size: 13px;
  color: var(--color-text-secondary);
  line-height: 1.5;
  max-width: 240px;
}

.upd-tip {
  font-size: 11px;
  color: var(--color-text-muted);
  margin-top: 16px;
  border-top: 1px solid var(--color-border-default);
  padding-top: 12px;
  width: 100%;
}

/* Preview view */
.upd-preview-box {
  flex: 1;
  border-radius: var(--radius-lg);
  border: 1px solid var(--color-border-default);
  background: var(--color-bg-primary);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  position: relative; /* Added relative positioning for floating copy */
}

.upd-preview-text {
  flex: 1;
  padding: 16px;
  font-family: monospace;
  font-size: 13px;
  line-height: 1.6;
  color: var(--color-text-primary);
  white-space: pre-wrap;
  word-break: break-all;
  overflow-y: auto;
  user-select: text;
}

@keyframes pulse-subtle {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.85; }
}
.animate-pulse-subtle {
  animation: pulse-subtle 3s infinite ease-in-out;
}

.upd-preview-text :deep(.upd-var-highlight) {
  display: inline-block;
  padding: 2px 6px;
  margin: 0 2px;
  border-radius: var(--radius-sm);
  font-size: 12px;
  font-weight: 600;
  background: color-mix(in srgb, var(--color-accent-blue) 10%, transparent);
  color: var(--color-accent-blue);
  border: 1px solid color-mix(in srgb, var(--color-accent-blue) 20%, transparent);
}

.upd-preview-text :deep(.upd-var-placeholder) {
  display: inline-block;
  padding: 2px 6px;
  margin: 0 2px;
  border-radius: var(--radius-sm);
  font-size: 11px;
  font-weight: 600;
  background: color-mix(in srgb, var(--color-accent-violet) 10%, transparent);
  color: var(--color-accent-violet);
  border: 1px dashed color-mix(in srgb, var(--color-accent-violet) 22%, transparent);
}

@media (max-width: 768px) {
  .upd-dialog {
    max-height: 95%;
    height: 95%;
  }
  .upd-body {
    flex-direction: column;
  }
  .upd-preview-panel {
    width: 100%;
    height: 55%;
    padding: 12px 12px 6px 12px;
  }
  .upd-form-panel {
    width: 100%;
    height: 45%;
    padding: 6px 12px 12px 12px;
  }
}
</style>
