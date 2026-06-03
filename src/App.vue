<script setup lang="ts">
import AppLayout from './components/layout/AppLayout.vue'
import CommandPalette from './components/common/CommandPalette.vue'
import { Toaster } from 'vue-sonner'
</script>

<template>
  <AppLayout />
  <CommandPalette />
  <Toaster position="bottom-right" theme="dark" :rich-colors="true" />
</template>
