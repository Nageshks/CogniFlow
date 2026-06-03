<script setup lang="ts">
import { ref } from 'vue'
import { toast } from 'vue-sonner'
import {
  Download,
  Upload,
  Sun,
  Moon,
  Sliders,
  Clock,
  Database,
  RotateCcw
} from 'lucide-vue-next'
import { useSettingsStore } from '@/stores/settings'
import ImportExportDialog from '@/components/common/ImportExportDialog.vue'

const settings = useSettingsStore()
const showImportExport = ref(false)

const semanticWeight = ref(settings.searchConfig.semantic_weight * 100)
const keywordWeight = ref(settings.searchConfig.keyword_weight * 100)

function updateWeights(semantic: number) {
  const s = Math.min(100, Math.max(0, semantic))
  semanticWeight.value = s
  keywordWeight.value = 100 - s
  settings.updateSearchConfig({
    semantic_weight: s / 100,
    keyword_weight: (100 - s) / 100,
  })
}

function resetWeights() {
  updateWeights(70)
  toast.success('Search weights reset to defaults')
}

const intervalOptions = [
  { label: '10s', value: 10000 },
  { label: '30s', value: 30000 },
  { label: '1m', value: 60000 },
  { label: '5m', value: 300000 },
]

function updateInterval(val: number) {
  settings.updateAutoSaveInterval(val)
  toast.success('Auto-save interval updated')
}

function toggleTheme(theme: 'light' | 'dark') {
  settings.updateTheme(theme)
  toast.success(`Switched to ${theme} mode`)
}

function resetAllSettings() {
  settings.updateTheme('light')
  settings.updateAutoSaveInterval(30000)
  updateWeights(70)
  toast.success('All settings reset to defaults')
}

function handleBackupClick() {
  showImportExport.value = true
}
</script>

<template>
  <div class="sv-root">
    <div class="sv-container">

      <!-- Subtitle -->
      <p class="sv-subtitle">Manage your workspace preferences.</p>

      <!-- Settings rows -->
      <div class="sv-sections">

        <!-- Section 1: Appearance -->
        <div class="sv-section">
          <div class="sv-section-header">
            <span class="sv-section-label">Appearance</span>
          </div>

          <div class="sv-row">
            <div class="sv-row-left">
              <div class="sv-row-icon">
                <Sun :size="16" v-if="settings.theme === 'light'" />
                <Moon :size="16" v-else />
              </div>
              <div class="sv-row-text">
                <span class="sv-row-title">Theme</span>
                <span class="sv-row-desc">Switch between light and dark mode</span>
              </div>
            </div>
            <div class="sv-segmented">
              <button
                id="theme-light"
                type="button"
                class="sv-seg-btn"
                :class="{ active: settings.theme === 'light' }"
                @click="toggleTheme('light')"
              >
                <Sun :size="14" />
                <span>Light</span>
              </button>
              <button
                id="theme-dark"
                type="button"
                class="sv-seg-btn"
                :class="{ active: settings.theme === 'dark' }"
                @click="toggleTheme('dark')"
              >
                <Moon :size="14" />
                <span>Dark</span>
              </button>
            </div>
          </div>
        </div>

        <!-- Section 2: Search -->
        <div class="sv-section">
          <div class="sv-section-header">
            <span class="sv-section-label">Search</span>
          </div>

          <div class="sv-row sv-row-col">
            <div class="sv-row-top">
              <div class="sv-row-left">
                <div class="sv-row-icon">
                  <Sliders :size="16" />
                </div>
                <div class="sv-row-text">
                  <span class="sv-row-title">Search weights</span>
                  <span class="sv-row-desc">Balance between semantic understanding and keyword matching</span>
                </div>
              </div>
            </div>

            <div class="sv-slider-area">
              <!-- Labels -->
              <div class="sv-slider-labels">
                <span class="sv-slider-label sv-label-semantic">
                  <span class="sv-dot sv-dot-semantic"></span>
                  Semantic {{ semanticWeight.toFixed(0) }}%
                </span>
                <span class="sv-slider-label sv-label-keyword">
                  <span class="sv-dot sv-dot-keyword"></span>
                  Keyword {{ keywordWeight.toFixed(0) }}%
                </span>
              </div>

              <!-- Slider -->
              <div class="sv-slider-track-wrap">
                <input
                  id="settings-semantic-weight"
                  type="range"
                  min="0"
                  max="100"
                  :value="semanticWeight"
                  class="sv-slider"
                  :style="{
                    background: `linear-gradient(to right, var(--sv-color-semantic) ${semanticWeight}%, var(--sv-color-keyword) ${semanticWeight}%)`
                  }"
                  @input="updateWeights(Number(($event.target as HTMLInputElement).value))"
                />
              </div>

              <!-- Actions -->
              <div class="sv-slider-actions">
                <button @click="resetWeights" class="sv-link-btn">Reset to defaults</button>
                <router-link to="/library" class="sv-link-btn sv-link-muted">
                  Test search →
                </router-link>
              </div>
            </div>
          </div>
        </div>

        <!-- Section 3: Auto-save -->
        <div class="sv-section">
          <div class="sv-section-header">
            <span class="sv-section-label">Storage</span>
          </div>

          <div class="sv-row">
            <div class="sv-row-left">
              <div class="sv-row-icon">
                <Clock :size="16" />
              </div>
              <div class="sv-row-text">
                <span class="sv-row-title">Auto-save interval</span>
                <span class="sv-row-desc">How often your work is automatically saved</span>
              </div>
            </div>
            <div class="sv-segmented sv-segmented-4">
              <button
                v-for="opt in intervalOptions"
                :key="opt.value"
                type="button"
                class="sv-seg-btn"
                :class="{ active: settings.autoSaveInterval === opt.value }"
                @click="updateInterval(opt.value)"
              >
                {{ opt.label }}
              </button>
            </div>
          </div>

          <div class="sv-divider"></div>

          <!-- Database row -->
          <div class="sv-row">
            <div class="sv-row-left">
              <div class="sv-row-icon">
                <Database :size="16" />
              </div>
              <div class="sv-row-text">
                <span class="sv-row-title">Backup &amp; Restore</span>
                <span class="sv-row-desc">Export or import your catalog database</span>
              </div>
            </div>
            <div class="sv-btn-group">
              <button
                id="settings-export"
                class="sv-btn sv-btn-secondary"
                @click="handleBackupClick"
              >
                <Download :size="14" />
                <span>Export</span>
              </button>
              <button
                id="settings-import"
                class="sv-btn sv-btn-primary"
                @click="handleBackupClick"
              >
                <Upload :size="14" />
                <span>Import</span>
              </button>
            </div>
          </div>
        </div>

        <!-- Section 4: Danger zone -->
        <div class="sv-section sv-section-danger">
          <div class="sv-section-header">
            <span class="sv-section-label sv-label-danger">Danger zone</span>
          </div>

          <div class="sv-row sv-row-danger" @click="resetAllSettings">
            <div class="sv-row-left">
              <div class="sv-row-icon sv-icon-danger">
                <RotateCcw :size="16" />
              </div>
              <div class="sv-row-text">
                <span class="sv-row-title sv-title-danger">Reset all settings</span>
                <span class="sv-row-desc">Restore theme, search weights, and intervals to defaults</span>
              </div>
            </div>
            <span class="sv-row-action-label">Reset now</span>
          </div>
        </div>

      </div>

      <!-- Version -->
      <div class="sv-version">CogniFlow v1.2.0</div>

    </div>

    <!-- Import/Export Dialog -->
    <ImportExportDialog
      :open="showImportExport"
      @close="showImportExport = false"
    />
  </div>
</template>

<style scoped>
/* ── Design tokens local to Settings ── */
.sv-root {
  --sv-color-semantic: #3B82F6;
  --sv-color-keyword: #8B5CF6;
  --sv-bg-page: var(--color-bg-primary);
  --sv-bg-section: var(--color-bg-secondary);
  --sv-bg-row-hover: var(--color-bg-tertiary);
  --sv-bg-segmented: var(--color-bg-tertiary);
  --sv-border: var(--color-border-default);
  --sv-border-subtle: var(--color-border-subtle);
  --sv-text-primary: var(--color-text-primary);
  --sv-text-secondary: var(--color-text-secondary);
  --sv-text-muted: var(--color-text-muted);
  --sv-radius: 14px;
  --sv-radius-inner: 10px;
  --sv-transition: 180ms cubic-bezier(0.4, 0, 0.2, 1);
}

/* ── Layout shell ── */
.sv-root {
  width: 100%;
  min-height: 100%;
  margin: -32px;
  padding: 48px 32px 64px;
  background: var(--sv-bg-page);
  display: flex;
  justify-content: center;
}

.sv-container {
  width: 100%;
  max-width: 620px;
  display: flex;
  flex-direction: column;
  gap: 0;
}

/* ── Subtitle ── */
.sv-subtitle {
  font-size: 13px;
  font-weight: 500;
  color: var(--sv-text-muted);
  margin-bottom: 32px;
  padding-left: 2px;
  letter-spacing: -0.01em;
}

/* ── Sections stack ── */
.sv-sections {
  display: flex;
  flex-direction: column;
  gap: 28px;
}

/* ── Section (a card-like group) ── */
.sv-section {
  background: var(--sv-bg-section);
  border: 1px solid var(--sv-border);
  border-radius: var(--sv-radius);
  overflow: hidden;
}

.sv-section-header {
  padding: 14px 20px 0;
}

.sv-section-label {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--sv-text-muted);
}

/* ── Row ── */
.sv-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 20px;
  gap: 16px;
  transition: background var(--sv-transition);
}

.sv-row:hover {
  background: var(--sv-bg-row-hover);
}

.sv-row-col {
  flex-direction: column;
  align-items: stretch;
}

.sv-row-col:hover {
  background: transparent;
}

.sv-row-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.sv-row-left {
  display: flex;
  align-items: center;
  gap: 14px;
  min-width: 0;
  flex: 1;
}

.sv-row-icon {
  width: 34px;
  height: 34px;
  border-radius: var(--sv-radius-inner);
  background: var(--sv-bg-segmented);
  border: 1px solid var(--sv-border);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: var(--sv-text-secondary);
  transition: all var(--sv-transition);
}

.sv-row:hover .sv-row-icon {
  color: var(--sv-text-primary);
  border-color: var(--color-accent-blue);
}

.sv-row-col:hover .sv-row-icon {
  color: var(--sv-text-secondary);
  border-color: var(--sv-border);
}

.sv-row-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.sv-row-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--sv-text-primary);
  letter-spacing: -0.01em;
}

.sv-row-desc {
  font-size: 12px;
  font-weight: 400;
  color: var(--sv-text-muted);
  line-height: 1.4;
}

/* ── Divider between rows in the same section ── */
.sv-divider {
  height: 1px;
  background: var(--sv-border);
  margin: 0 20px;
}

/* ── Segmented Control ── */
.sv-segmented {
  display: flex;
  align-items: center;
  background: var(--sv-bg-segmented);
  border: 1px solid var(--sv-border);
  border-radius: var(--sv-radius-inner);
  padding: 3px;
  height: 38px;
  width: 180px;
  flex-shrink: 0;
}

.sv-segmented-4 {
  width: 240px;
}

.sv-seg-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 100%;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--sv-text-muted);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--sv-transition);
  font-family: var(--font-sans);
  outline: none;
}

.sv-seg-btn:hover:not(.active) {
  color: var(--sv-text-secondary);
}

.sv-seg-btn.active {
  background: var(--color-bg-primary);
  color: var(--sv-text-primary);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08), 0 1px 2px rgba(0, 0, 0, 0.04);
  font-weight: 600;
}

/* ── Slider area ── */
.sv-slider-area {
  padding: 4px 0 2px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.sv-slider-labels {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.sv-slider-label {
  font-size: 12px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 6px;
  letter-spacing: -0.01em;
}

.sv-label-semantic {
  color: var(--sv-color-semantic);
}

.sv-label-keyword {
  color: var(--sv-color-keyword);
}

.sv-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.sv-dot-semantic {
  background: var(--sv-color-semantic);
}

.sv-dot-keyword {
  background: var(--sv-color-keyword);
}

/* Slider input */
.sv-slider {
  -webkit-appearance: none;
  appearance: none;
  width: 100%;
  height: 6px;
  border-radius: 9999px;
  outline: none;
  cursor: pointer;
  background: var(--sv-border);
}

.sv-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--color-bg-primary);
  border: 2px solid var(--sv-text-primary);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
  cursor: grab;
  transition: transform 120ms cubic-bezier(0.16, 1, 0.3, 1), box-shadow 120ms ease;
}

.sv-slider::-webkit-slider-thumb:active {
  cursor: grabbing;
  transform: scale(1.15);
  box-shadow: 0 3px 10px rgba(0, 0, 0, 0.2);
}

.sv-slider-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.sv-link-btn {
  font-size: 12px;
  font-weight: 600;
  color: var(--sv-color-semantic);
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
  font-family: var(--font-sans);
  text-decoration: none;
  transition: opacity var(--sv-transition);
}

.sv-link-btn:hover {
  opacity: 0.7;
}

.sv-link-muted {
  color: var(--sv-text-muted);
}

.sv-link-muted:hover {
  color: var(--sv-text-primary);
  opacity: 1;
}

/* ── Buttons ── */
.sv-btn-group {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.sv-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 36px;
  padding: 0 16px;
  border-radius: 9px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid transparent;
  font-family: var(--font-sans);
  transition: all var(--sv-transition);
  white-space: nowrap;
}

.sv-btn-secondary {
  background: var(--sv-bg-segmented);
  border-color: var(--sv-border);
  color: var(--sv-text-primary);
}

.sv-btn-secondary:hover {
  background: var(--sv-bg-row-hover);
  border-color: var(--color-accent-blue);
}

.sv-btn-primary {
  background: var(--sv-text-primary);
  color: var(--color-bg-primary);
}

.sv-btn-primary:hover {
  opacity: 0.85;
}

/* ── Danger zone ── */
.sv-section-danger {
  border-color: color-mix(in srgb, var(--color-accent-rose) 25%, var(--sv-border));
  background: color-mix(in srgb, var(--color-accent-rose) 4%, var(--sv-bg-section));
}

.sv-label-danger {
  color: var(--color-accent-rose);
}

.sv-row-danger {
  cursor: pointer;
}

.sv-icon-danger {
  background: color-mix(in srgb, var(--color-accent-rose) 12%, transparent);
  border-color: color-mix(in srgb, var(--color-accent-rose) 25%, transparent);
  color: var(--color-accent-rose);
}

.sv-row-danger:hover .sv-icon-danger {
  color: var(--color-accent-rose);
  border-color: var(--color-accent-rose);
}

.sv-title-danger {
  color: var(--color-accent-rose);
}

.sv-row-action-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--sv-text-muted);
  flex-shrink: 0;
  transition: color var(--sv-transition);
}

.sv-row-danger:hover .sv-row-action-label {
  color: var(--color-accent-rose);
}

/* ── Version footer ── */
.sv-version {
  text-align: center;
  font-size: 11px;
  font-weight: 500;
  color: var(--sv-text-muted);
  margin-top: 48px;
  letter-spacing: 0.02em;
}

/* ── Responsive ── */
@media (max-width: 640px) {
  .sv-root {
    padding: 24px 16px 48px;
  }

  .sv-row {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
  }

  .sv-segmented,
  .sv-segmented-4 {
    width: 100%;
  }

  .sv-btn-group {
    width: 100%;
  }

  .sv-btn {
    flex: 1;
  }
}
</style>
