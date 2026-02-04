<script setup lang="ts">
import { computed } from 'vue'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { useDocumentsStore, useSelectionStore, useUiStore } from '@/stores'

const documentsStore = useDocumentsStore()
const selectionStore = useSelectionStore()
const uiStore = useUiStore()

const selectedPagesWithInfo = computed(() => {
  return selectionStore.selectedPages.map((pageWithChunks) => ({
    ...pageWithChunks,
    thumbnailUrl: documentsStore.getThumbnailUrl(pageWithChunks.page.id),
  }))
})

function handlePreview(pageId: number) {
  uiStore.openPreview(pageId)
}

function handleRemove(pageId: number) {
  selectionStore.removeFromSelection(pageId)
}

const documentTitle = computed(() => {
  const doc = documentsStore.currentDocumentInfo
  if (!doc) return 'Untitled'
  return doc.title || doc.filename || 'Untitled'
})
</script>

<template>
  <div class="space-y-2">
    <!-- Empty State -->
    <div
      v-if="!selectionStore.hasSelection"
      class="flex flex-col items-center justify-center rounded-lg border border-dashed border-gray-600 p-6 text-center"
    >
      <span class="i-mdi-image-multiple-outline text-3xl text-gray-500 mb-2" />
      <p class="text-sm text-gray-500">
        Click pages in the grid to add them as evidence
      </p>
      <p class="text-xs text-gray-600 mt-1">
        Use Cmd+click for multi-select, Shift+click for range
      </p>
    </div>

    <!-- Evidence Items -->
    <div v-else class="space-y-2">
      <div
        v-for="item in selectedPagesWithInfo"
        :key="item.page.id"
        class="flex items-center gap-3 rounded-lg bg-gray-700/50 p-2"
      >
        <!-- Thumbnail -->
        <button
          class="h-12 w-10 flex-shrink-0 overflow-hidden rounded bg-gray-700"
          @click="handlePreview(item.page.id)"
        >
          <img
            v-if="item.thumbnailUrl"
            :src="item.thumbnailUrl"
            :alt="`Page ${item.page.page_num}`"
            class="h-full w-full object-cover"
          />
          <div v-else class="flex h-full w-full items-center justify-center">
            <span class="i-mdi-file-document-outline text-gray-500" />
          </div>
        </button>

        <!-- Info -->
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <span class="text-sm font-medium text-gray-200">
              Page {{ item.page.page_num }}
            </span>
            <Badge
              v-if="item.chunks.length > 0"
              variant="secondary"
              class="text-xs bg-gray-600"
            >
              {{ item.chunks.length }} chunk{{ item.chunks.length > 1 ? 's' : '' }}
            </Badge>
          </div>
          <p class="text-xs text-gray-500 truncate">
            {{ documentTitle }}
          </p>
        </div>

        <!-- Actions -->
        <Button
          variant="ghost"
          size="sm"
          class="h-8 w-8 p-0 text-gray-400 hover:text-red-400"
          @click="handleRemove(item.page.id)"
        >
          <span class="i-mdi-close" />
        </Button>
      </div>
    </div>

    <!-- Summary -->
    <div
      v-if="selectionStore.hasSelection"
      class="flex items-center justify-between text-xs text-gray-500 pt-2"
    >
      <span>{{ selectionStore.selectedCount }} page{{ selectionStore.selectedCount > 1 ? 's' : '' }} selected</span>
      <button
        class="text-gray-400 hover:text-gray-300"
        @click="selectionStore.clearSelection()"
      >
        Clear all
      </button>
    </div>
  </div>
</template>
