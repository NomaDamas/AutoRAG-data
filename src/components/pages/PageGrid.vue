<script setup lang="ts">
import { computed } from 'vue'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Skeleton } from '@/components/ui/skeleton'
import { useConnectionStore, useDocumentsStore, useSelectionStore, useUiStore } from '@/stores'
import PageThumbnail from './PageThumbnail.vue'
import { useQuickPreview } from '@/composables/useQuickPreview'
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts'

const connectionStore = useConnectionStore()
const documentsStore = useDocumentsStore()
const selectionStore = useSelectionStore()
const uiStore = useUiStore()

const { isPreviewActive } = useQuickPreview()
useKeyboardShortcuts()

const gridCols = computed(() => {
  switch (uiStore.thumbnailSize) {
    case 'small':
      return 'grid-cols-6 lg:grid-cols-8'
    case 'large':
      return 'grid-cols-2 lg:grid-cols-3'
    default:
      return 'grid-cols-4 lg:grid-cols-5'
  }
})

function handlePageClick(pageId: number, event: MouseEvent) {
  selectionStore.togglePage(pageId, {
    shiftKey: event.shiftKey,
    metaKey: event.metaKey,
  })
}

const documentTitle = computed(() => {
  const doc = documentsStore.currentDocumentInfo
  if (!doc) return 'Pages'
  return doc.title || doc.filename || 'Untitled'
})
</script>

<template>
  <div class="flex h-full flex-col bg-gray-900">
    <!-- Header -->
    <div class="flex items-center justify-between border-b border-gray-700 px-4 py-3">
      <h2 class="text-sm font-semibold text-gray-200">
        {{ documentTitle }}
      </h2>
      <div v-if="selectionStore.hasSelection" class="flex items-center gap-2">
        <span class="text-xs text-gray-400">
          {{ selectionStore.selectedCount }} selected
        </span>
        <button
          class="text-xs text-blue-400 hover:text-blue-300"
          @click="selectionStore.selectAll()"
        >
          Select all
        </button>
        <button
          class="text-xs text-gray-400 hover:text-gray-300"
          @click="selectionStore.clearSelection()"
        >
          Clear
        </button>
      </div>
    </div>

    <ScrollArea class="flex-1">
      <div class="p-4">
        <!-- Not Connected State -->
        <div
          v-if="!connectionStore.isConnected"
          class="flex flex-col items-center justify-center p-16 text-center text-gray-500"
        >
          <span class="i-mdi-database-off text-6xl mb-4" />
          <p>Connect to a database to view pages</p>
        </div>

        <!-- No Document Selected -->
        <div
          v-else-if="!documentsStore.currentDocument"
          class="flex flex-col items-center justify-center p-16 text-center text-gray-500"
        >
          <span class="i-mdi-file-document-outline text-6xl mb-4" />
          <p>Select a document to view pages</p>
        </div>

        <!-- Loading State -->
        <div v-else-if="documentsStore.isLoading" :class="['grid gap-4', gridCols]">
          <Skeleton
            v-for="i in 12"
            :key="i"
            class="aspect-[3/4] w-full rounded-lg bg-gray-700"
          />
        </div>

        <!-- Page Grid -->
        <div v-else :class="['grid gap-4', gridCols]">
          <PageThumbnail
            v-for="pageWithChunks in documentsStore.currentPages"
            :key="pageWithChunks.page.id"
            :page="pageWithChunks.page"
            :chunks="pageWithChunks.chunks"
            :is-selected="selectionStore.isSelected(pageWithChunks.page.id)"
            :show-page-number="uiStore.showPageNumbers"
            :thumbnail-url="documentsStore.getThumbnailUrl(pageWithChunks.page.id)"
            @click="handlePageClick(pageWithChunks.page.id, $event)"
          />
        </div>
      </div>
    </ScrollArea>

    <!-- Quick Preview Indicator -->
    <div
      v-if="isPreviewActive"
      class="absolute bottom-12 left-1/2 -translate-x-1/2 rounded-full bg-gray-800/90 px-4 py-2 text-sm text-gray-300"
    >
      Hold Space to preview
    </div>
  </div>
</template>
