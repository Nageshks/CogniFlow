<script setup lang="ts">
import { ref } from 'vue'
import { X, Folder, Download, Upload } from 'lucide-vue-next'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { toast } from 'vue-sonner'
import * as api from '@/lib/tauri-api'
import type { ImportFormat, ExportFormat } from '@/types'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const activeTab = ref<'import' | 'export'>('import')
const importFormat = ref<ImportFormat>('json')
const importPath = ref('')
const exportFormat = ref<ExportFormat>('json')
const exportPath = ref('')
const exportExpanded = ref(false)
const loading = ref(false)

async function pickImportPath() {
  try {
    const isDir = importFormat.value === 'markdown' || importFormat.value === 'txt'
    const selected = await openDialog({
      directory: isDir,
      multiple: false,
      title: isDir ? 'Select Import Folder' : 'Select Import File',
      filters: isDir ? [] : [
        { name: 'JSON Data', extensions: ['json'] }
      ]
    })
    
    if (selected) {
      // openDialog returns string or array of strings, or null
      importPath.value = Array.isArray(selected) ? selected[0] : selected
    }
  } catch (e) {
    toast.error('Dialog selection failed: ' + String(e))
  }
}

async function pickExportPath() {
  try {
    const isDir = exportFormat.value === 'markdown'
    const selected = await openDialog({
      directory: isDir,
      multiple: false,
      title: isDir ? 'Select Export Folder' : 'Select Export File',
      filters: isDir ? [] : [
        { name: 'JSON Data', extensions: ['json'] }
      ]
    })
    
    if (selected) {
      exportPath.value = Array.isArray(selected) ? selected[0] : selected
    }
  } catch (e) {
    toast.error('Dialog selection failed: ' + String(e))
  }
}

async function runImport() {
  if (!importPath.value) return
  loading.value = true
  try {
    const count = await api.importData({
      format: importFormat.value,
      path: importPath.value
    })
    toast.success(`Successfully imported ${count} assets`)
    emit('close')
  } catch (e) {
    toast.error('Import failed: ' + String(e))
  } finally {
    loading.value = false
  }
}

async function runExport() {
  if (!exportPath.value) return
  loading.value = true
  try {
    await api.exportData({
      format: exportFormat.value,
      path: exportPath.value,
      expanded: exportExpanded.value
    })
    toast.success('Export completed successfully')
    emit('close')
  } catch (e) {
    toast.error('Export failed: ' + String(e))
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="v">
      <div v-if="open" class="fixed inset-0 z-50 flex items-center justify-center" style="background: rgba(0,0,0,0.65);">
        <div class="glass-card p-6 w-full max-w-md animate-fade-in flex flex-col gap-4" @click.stop>
          <!-- Header -->
          <div class="flex items-center justify-between border-b pb-3" style="border-color: var(--color-border-subtle);">
            <h2 class="text-sm font-bold uppercase tracking-wider" style="color: var(--color-text-primary);">
              Data Transfer
            </h2>
            <button class="p-1.5 rounded-lg cursor-pointer" style="color: var(--color-text-muted);" @click="emit('close')">
              <X :size="16" />
            </button>
          </div>

          <!-- Tabs -->
          <div class="flex rounded-lg overflow-hidden p-1" style="background: var(--color-bg-tertiary); border: 1px solid var(--color-border-default);">
            <button
              class="flex-1 py-1.5 text-xs font-semibold rounded-md transition-all cursor-pointer"
              :style="activeTab === 'import' ? 'background: var(--color-bg-elevated); color: var(--color-accent-blue);' : 'color: var(--color-text-muted);'"
              @click="activeTab = 'import'"
            >
              Import
            </button>
            <button
              class="flex-1 py-1.5 text-xs font-semibold rounded-md transition-all cursor-pointer"
              :style="activeTab === 'export' ? 'background: var(--color-bg-elevated); color: var(--color-accent-violet);' : 'color: var(--color-text-muted);'"
              @click="activeTab = 'export'"
            >
              Export
            </button>
          </div>

          <!-- IMPORT TAB -->
          <div v-if="activeTab === 'import'" class="flex flex-col gap-3">
            <div>
              <label class="text-xs font-semibold mb-1 block" style="color: var(--color-text-secondary);">Format</label>
              <select
                id="import-format"
                v-model="importFormat"
                class="w-full px-3 py-2 rounded-lg text-sm cursor-pointer"
                style="background: var(--color-bg-tertiary); border: 1px solid var(--color-border-default); color: var(--color-text-primary); outline: none;"
                @change="importPath = ''"
              >
                <option value="json">JSON Backup File</option>
                <option value="markdown">Markdown Directory</option>
                <option value="txt">Text Directory</option>
              </select>
            </div>

            <div>
              <label class="text-xs font-semibold mb-1 block" style="color: var(--color-text-secondary);">Path Source</label>
              <div class="flex gap-2">
                <input
                  id="import-path"
                  v-model="importPath"
                  type="text"
                  placeholder="Select source path..."
                  readonly
                  class="flex-1 px-3 py-2 rounded-lg text-xs"
                  style="background: var(--color-bg-tertiary); border: 1px solid var(--color-border-default); color: var(--color-text-primary); outline: none;"
                />
                <button
                  id="btn-pick-import"
                  class="p-2 rounded-lg cursor-pointer transition-colors"
                  style="background: var(--color-bg-tertiary); border: 1px solid var(--color-border-default); color: var(--color-text-secondary);"
                  @click="pickImportPath"
                >
                  <Folder :size="16" />
                </button>
              </div>
            </div>

            <div class="flex justify-end gap-2 mt-4">
              <button
                class="px-4 py-2 rounded-lg text-xs font-medium cursor-pointer transition-colors"
                style="color: var(--color-text-secondary);"
                @click="emit('close')"
              >
                Cancel
              </button>
              <button
                id="submit-import"
                class="flex items-center gap-1.5 px-4 py-2 rounded-lg text-xs font-medium cursor-pointer transition-colors"
                style="background: var(--color-accent-blue); color: white;"
                :disabled="!importPath || loading"
                @click="runImport"
              >
                <Upload :size="14" />
                <span>Import Data</span>
              </button>
            </div>
          </div>

          <!-- EXPORT TAB -->
          <div v-else class="flex flex-col gap-3">
            <div>
              <label class="text-xs font-semibold mb-1 block" style="color: var(--color-text-secondary);">Format</label>
              <select
                id="export-format"
                v-model="exportFormat"
                class="w-full px-3 py-2 rounded-lg text-sm cursor-pointer"
                style="background: var(--color-bg-tertiary); border: 1px solid var(--color-border-default); color: var(--color-text-primary); outline: none;"
                @change="exportPath = ''"
              >
                <option value="json">JSON Backup File</option>
                <option value="markdown">Markdown Folder</option>
              </select>
            </div>

            <div>
              <label class="text-xs font-semibold mb-1 block" style="color: var(--color-text-secondary);">Destination Path</label>
              <div class="flex gap-2">
                <input
                  id="export-path"
                  v-model="exportPath"
                  type="text"
                  placeholder="Select destination path..."
                  readonly
                  class="flex-1 px-3 py-2 rounded-lg text-xs"
                  style="background: var(--color-bg-tertiary); border: 1px solid var(--color-border-default); color: var(--color-text-primary); outline: none;"
                />
                <button
                  id="btn-pick-export"
                  class="p-2 rounded-lg cursor-pointer transition-colors"
                  style="background: var(--color-bg-tertiary); border: 1px solid var(--color-border-default); color: var(--color-text-secondary);"
                  @click="pickExportPath"
                >
                  <Folder :size="16" />
                </button>
              </div>
            </div>

            <!-- Markdown options -->
            <div v-if="exportFormat === 'markdown'" class="flex items-center gap-2 mt-1">
              <input
                id="export-expanded"
                v-model="exportExpanded"
                type="checkbox"
                class="cursor-pointer w-4 h-4"
                style="accent-color: var(--color-accent-violet);"
              />
              <label for="export-expanded" class="text-xs font-medium cursor-pointer" style="color: var(--color-text-secondary);">
                Expanded folder workflows (individual step files)
              </label>
            </div>

            <div class="flex justify-end gap-2 mt-4">
              <button
                class="px-4 py-2 rounded-lg text-xs font-medium cursor-pointer transition-colors"
                style="color: var(--color-text-secondary);"
                @click="emit('close')"
              >
                Cancel
              </button>
              <button
                id="submit-export"
                class="flex items-center gap-1.5 px-4 py-2 rounded-lg text-xs font-medium cursor-pointer transition-colors"
                style="background: var(--color-accent-violet); color: white;"
                :disabled="!exportPath || loading"
                @click="runExport"
              >
                <Download :size="14" />
                <span>Export Data</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
