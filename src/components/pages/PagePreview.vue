<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Skeleton } from '@/components/ui/skeleton'
import { useDocumentsStore, useUiStore } from '@/stores'

const documentsStore = useDocumentsStore()
const uiStore = useUiStore()

const previewUrl = ref<string | null>(null)
const isLoadingPreview = ref(false)

const previewPage = computed(() => {
  if (uiStore.previewPageId === null) return null
  const pageWithChunks = documentsStore.currentPages.find(
    (pw) => pw.page.id === uiStore.previewPageId
  )
  return pageWithChunks?.page ?? null
})

watch(
  () => uiStore.previewPageId,
  async (pageId) => {
    if (pageId === null) {
      previewUrl.value = null
      return
    }

    const page = previewPage.value
    if (!page) return

    isLoadingPreview.value = true
    try {
      previewUrl.value = await documentsStore.getPreviewUrl(pageId)
    } finally {
      isLoadingPreview.value = false
    }
  }
)

function handleClose() {
  uiStore.closePreview()
}
</script>

<template>
  <Dialog :open="uiStore.isPreviewModalOpen" @update:open="handleClose">
    <DialogContent class="max-w-[95vw] sm:max-w-[95vw] bg-gray-800 border-gray-700 text-gray-100">
      <DialogHeader>
        <DialogTitle v-if="previewPage">
          Page {{ previewPage.page_num }}
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
          :alt="`Page ${previewPage?.page_num} preview`"
          class="mx-auto max-h-[92vh] rounded-lg object-contain"
        />

        <!-- Error State -->
        <div v-else class="flex h-full flex-col items-center justify-center text-gray-500">
          <span class="i-mdi-image-off text-6xl mb-4" />
          <p>Failed to load preview</p>
        </div>
      </div>

      <div class="flex items-center justify-between text-sm text-gray-400">
        <div v-if="previewPage">
          Page {{ previewPage.page_num }}
        </div>
        <div class="text-xs">Press Space to quick preview selected page</div>
      </div>
    </DialogContent>
  </Dialog>
</template>
