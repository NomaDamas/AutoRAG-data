<script setup lang="ts">
import { computed } from 'vue'
import { Separator } from '@/components/ui/separator'
import { useConnectionStore, useDocumentsStore, useSelectionStore, useUiStore } from '@/stores'

const connectionStore = useConnectionStore()
const documentsStore = useDocumentsStore()
const selectionStore = useSelectionStore()
const uiStore = useUiStore()

const connectionStatus = computed(() => {
  if (connectionStore.isConnecting) return 'connecting'
  if (connectionStore.isConnected) return 'connected'
  return 'disconnected'
})

const statusColor = computed(() => {
  switch (connectionStatus.value) {
    case 'connected':
      return 'bg-green-500'
    case 'connecting':
      return 'bg-yellow-500'
    default:
      return 'bg-red-500'
  }
})

const documentTitle = computed(() => {
  const doc = documentsStore.currentDocumentInfo
  if (!doc) return null
  return doc.title || doc.filename || 'Untitled'
})
</script>

<template>
  <div class="flex h-8 items-center justify-between border-t border-gray-700 bg-gray-800 px-4 text-xs text-gray-400">
    <div class="flex items-center gap-4">
      <!-- Connection Status -->
      <button
        class="flex items-center gap-2 hover:text-gray-200 transition-colors"
        @click="uiStore.openConnectionDialog()"
      >
        <span :class="['h-2 w-2 rounded-full', statusColor]" />
        <span class="capitalize">{{ connectionStatus }}</span>
      </button>

      <Separator orientation="vertical" class="h-4" />

      <!-- Document Info -->
      <span v-if="documentsStore.currentDocumentInfo">
        {{ documentTitle }}
        ({{ documentsStore.pageCount }} pages)
      </span>
      <span v-else class="text-gray-500">No document selected</span>
    </div>

    <div class="flex items-center gap-4">
      <!-- Selection Info -->
      <span v-if="selectionStore.hasSelection">
        {{ selectionStore.selectedCount }} page{{ selectionStore.selectedCount > 1 ? 's' : '' }} selected
      </span>

      <Separator v-if="selectionStore.hasSelection" orientation="vertical" class="h-4" />

      <!-- View Controls -->
      <div class="flex items-center gap-1">
        <button
          v-for="size in ['small', 'medium', 'large'] as const"
          :key="size"
          class="px-2 py-0.5 rounded text-xs transition-colors"
          :class="uiStore.thumbnailSize === size ? 'bg-gray-600 text-white' : 'hover:bg-gray-700'"
          @click="uiStore.setThumbnailSize(size)"
        >
          {{ size.charAt(0).toUpperCase() }}
        </button>
      </div>
    </div>
  </div>
</template>
