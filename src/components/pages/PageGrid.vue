<script setup lang="ts">
import { computed, ref, watch } from 'vue'
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

// Pagination
const PAGES_PER_VIEW = 10
const currentPageIndex = ref(0)

const totalPageCount = computed(() => documentsStore.currentPages.length)
const totalPaginationPages = computed(() => Math.ceil(totalPageCount.value / PAGES_PER_VIEW))

const paginatedPages = computed(() => {
  const start = currentPageIndex.value * PAGES_PER_VIEW
  return documentsStore.currentPages.slice(start, start + PAGES_PER_VIEW)
})

const pageRangeText = computed(() => {
  if (totalPageCount.value === 0) return '0 pages'
  const start = currentPageIndex.value * PAGES_PER_VIEW + 1
  const end = Math.min(start + PAGES_PER_VIEW - 1, totalPageCount.value)
  return `${start}-${end} of ${totalPageCount.value}`
})

function prevPage() {
  if (currentPageIndex.value > 0) currentPageIndex.value--
}

function nextPage() {
  if (currentPageIndex.value < totalPaginationPages.value - 1) currentPageIndex.value++
}

// Reset pagination when document changes
watch(() => documentsStore.currentDocument, () => {
  currentPageIndex.value = 0
})

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
  if (event.metaKey || event.ctrlKey) {
    // Cmd+click: toggle evidence membership
    selectionStore.toggleEvidence(pageId)
  } else {
    // Plain click: focus page
    selectionStore.focusPage(pageId)
  }
}

function handleToggleEvidence(pageId: number) {
  selectionStore.toggleEvidence(pageId)
}

const documentTitle = computed(() => {
  const doc = documentsStore.currentDocumentInfo
  if (!doc) return 'Pages'
  return doc.title || doc.filename || 'Untitled'
})
</script>

<template>
  <div class="flex h-full flex-col overflow-hidden bg-gray-900">
    <!-- Header -->
    <div class="flex items-center justify-between border-b border-gray-700 px-4 py-3">
      <h2 class="text-sm font-semibold text-gray-200">
        {{ documentTitle }}
      </h2>
      <div class="flex items-center gap-4">
        <!-- Evidence Controls -->
        <div v-if="selectionStore.hasSelection" class="flex items-center gap-2">
          <span class="text-xs text-amber-400">
            {{ selectionStore.selectedCount }} in evidence
          </span>
          <button
            class="text-xs text-gray-400 hover:text-gray-300"
            @click="selectionStore.clearEvidence()"
          >
            Clear
          </button>
        </div>

        <!-- Pagination Controls -->
        <div v-if="totalPageCount > 0" class="flex items-center gap-2">
          <span class="text-xs text-gray-400">{{ pageRangeText }}</span>
          <button
            class="rounded px-2 py-1 text-sm text-gray-400 hover:bg-gray-700 hover:text-gray-200 disabled:opacity-30 disabled:hover:bg-transparent disabled:hover:text-gray-400"
            :disabled="currentPageIndex === 0"
            @click="prevPage"
          >
            ←
          </button>
          <button
            class="rounded px-2 py-1 text-sm text-gray-400 hover:bg-gray-700 hover:text-gray-200 disabled:opacity-30 disabled:hover:bg-transparent disabled:hover:text-gray-400"
            :disabled="currentPageIndex >= totalPaginationPages - 1"
            @click="nextPage"
          >
            →
          </button>
        </div>
      </div>
    </div>

    <!-- Content Area (no ScrollArea) -->
    <div class="flex-1 p-4">
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
          v-for="i in 10"
          :key="i"
          class="aspect-[3/4] w-full rounded-lg bg-gray-700"
        />
      </div>

      <!-- Page Grid -->
      <div v-else :class="['grid gap-4', gridCols]">
        <PageThumbnail
          v-for="pageWithChunks in paginatedPages"
          :key="pageWithChunks.page.id"
          :page="pageWithChunks.page"
          :chunks="pageWithChunks.chunks"
          :is-focused="selectionStore.focusedPageId === pageWithChunks.page.id"
          :is-in-evidence="selectionStore.isInEvidence(pageWithChunks.page.id)"
          :show-page-number="uiStore.showPageNumbers"
          :thumbnail-url="documentsStore.getThumbnailUrl(pageWithChunks.page.id)"
          @click="handlePageClick(pageWithChunks.page.id, $event)"
          @toggle-evidence="handleToggleEvidence(pageWithChunks.page.id)"
        />
      </div>
    </div>

    <!-- Quick Preview Indicator -->
    <div
      v-if="isPreviewActive"
      class="absolute bottom-12 left-1/2 -translate-x-1/2 rounded-full bg-gray-800/90 px-4 py-2 text-sm text-gray-300"
    >
      Hold Space to preview
    </div>
  </div>
</template>
