<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { useDocumentsStore, useSelectionStore, useUiStore } from '@/stores'

const documentsStore = useDocumentsStore()
const selectionStore = useSelectionStore()
const uiStore = useUiStore()

const previewUrl = ref<string | null>(null)
const isLoadingPreview = ref(false)

const previewPage = computed(() => {
  if (uiStore.previewPageId === null) return null
  // Try current document pages first
  const pageWithChunks = documentsStore.currentPages.find(
    (pw) => pw.page.id === uiStore.previewPageId
  )
  if (pageWithChunks) return pageWithChunks
  // Fallback to evidence items (cross-doc)
  const evidenceItem = selectionStore.evidenceItems.get(uiStore.previewPageId)
  if (evidenceItem) return { page: evidenceItem.page, chunks: evidenceItem.chunks }
  return null
})

const isPageInEvidence = computed(() => {
  if (uiStore.previewPageId === null) return false
  return selectionStore.isInEvidence(uiStore.previewPageId)
})

watch(
  () => uiStore.previewPageId,
  async (pageId) => {
    if (pageId === null) {
      previewUrl.value = null
      return
    }

    const pw = previewPage.value
    if (!pw) return

    isLoadingPreview.value = true
    try {
      // Try page source URL first (direct file)
      const sourceUrl = documentsStore.getPageSourceUrl(pageId)
      if (sourceUrl) {
        previewUrl.value = sourceUrl
        return
      }

      // Fall back to BYTEA data URL via first chunk
      const chunkId = pw.chunks[0]?.id
      if (chunkId) {
        previewUrl.value = await documentsStore.getChunkDataUrl(chunkId)
      } else {
        previewUrl.value = null
      }
    } finally {
      isLoadingPreview.value = false
    }
  }
)

function handleClose() {
  uiStore.closePreview()
}

function handleToggleEvidence() {
  if (uiStore.previewPageId !== null) {
    selectionStore.toggleEvidence(uiStore.previewPageId)
  }
}
</script>

<template>
  <Dialog :open="uiStore.isPreviewModalOpen" @update:open="handleClose">
    <DialogContent class="max-w-[95vw] sm:max-w-[95vw] bg-gray-800 border-gray-700 text-gray-100">
      <DialogHeader>
        <DialogTitle v-if="previewPage">
          Page {{ previewPage.page.page_num }}
        </DialogTitle>
      </DialogHeader>

      <div class="relative min-h-[90vh]">
        <!-- Loading State -->
        <div v-if="isLoadingPreview" class="flex h-full items-center justify-center">
          <Skeleton class="h-[90vh] w-full bg-gray-700" />
        </div>

        <!-- Preview Image -->
        <img
          v-else-if="previewUrl"
          :src="previewUrl"
          :alt="`Page ${previewPage?.page.page_num} preview`"
          class="mx-auto max-h-[92vh] rounded-lg object-contain"
        />

        <!-- Error State -->
        <div v-else class="flex h-full flex-col items-center justify-center text-gray-500">
          <span class="i-mdi-image-off text-6xl mb-4" />
          <p>Failed to load preview</p>
        </div>
      </div>

      <div class="flex items-center justify-between text-sm text-gray-400">
        <div class="flex items-center gap-3">
          <span v-if="previewPage">
            Page {{ previewPage.page.page_num }}
          </span>
          <Button
            v-if="uiStore.previewPageId !== null"
            size="sm"
            :variant="isPageInEvidence ? 'secondary' : 'outline'"
            :class="isPageInEvidence
              ? 'bg-amber-600/80 text-white hover:bg-amber-700'
              : 'border-gray-600 hover:bg-amber-600/80 hover:text-white hover:border-amber-600'"
            @click="handleToggleEvidence"
          >
            <span v-if="isPageInEvidence" class="i-mdi-check mr-1" />
            <span v-else class="i-mdi-plus mr-1" />
            {{ isPageInEvidence ? 'In evidence' : 'Add to evidence' }}
          </Button>
        </div>
        <div class="text-xs">Press Enter to preview focused page</div>
      </div>
    </DialogContent>
  </Dialog>
</template>
