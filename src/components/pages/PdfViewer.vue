<script setup lang="ts">
import { ref, watch, onUnmounted, computed } from 'vue'
import * as pdfjsLib from 'pdfjs-dist'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { useSelectionStore, useDocumentsStore } from '@/stores'

pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
  'pdfjs-dist/build/pdf.worker.mjs',
  import.meta.url,
).toString()

const props = defineProps<{
  pdfUrl: string
}>()

const selectionStore = useSelectionStore()
const documentsStore = useDocumentsStore()

const canvasRef = ref<HTMLCanvasElement | null>(null)
const currentPage = ref(1)
const totalPages = ref(0)
const isLoading = ref(false)
const error = ref<string | null>(null)
const scale = ref(1.5)

let pdfDoc: pdfjsLib.PDFDocumentProxy | null = null

const canGoBack = computed(() => currentPage.value > 1)
const canGoForward = computed(() => currentPage.value < totalPages.value)

// Resolve the current page_num → PageWithChunks from the store (single lookup)
const currentPageData = computed(() =>
  documentsStore.currentPages.find(
    (pw) => pw.page.page_num === currentPage.value,
  ) ?? null,
)

async function loadPdf(url: string) {
  isLoading.value = true
  error.value = null

  try {
    if (pdfDoc) {
      pdfDoc.destroy()
      pdfDoc = null
    }

    const loadingTask = pdfjsLib.getDocument(url)
    pdfDoc = await loadingTask.promise
    totalPages.value = pdfDoc.numPages
    currentPage.value = 1
    await renderPage(1)
  } catch (err) {
    console.error('Failed to load PDF:', err)
    error.value = err instanceof Error ? err.message : String(err)
  } finally {
    isLoading.value = false
  }
}

async function renderPage(pageNum: number) {
  if (!pdfDoc || !canvasRef.value) return

  isLoading.value = true
  try {
    const page = await pdfDoc.getPage(pageNum)
    const viewport = page.getViewport({ scale: scale.value })

    const canvas = canvasRef.value
    const context = canvas.getContext('2d')
    if (!context) return

    canvas.height = viewport.height
    canvas.width = viewport.width

    await page.render({
      canvasContext: context,
      viewport,
      canvas,
    }).promise
  } catch (err) {
    console.error('Failed to render page:', err)
    error.value = err instanceof Error ? err.message : String(err)
  } finally {
    isLoading.value = false
  }
}

function prevPage() {
  if (canGoBack.value) {
    currentPage.value--
    renderPage(currentPage.value)
  }
}

function nextPage() {
  if (canGoForward.value) {
    currentPage.value++
    renderPage(currentPage.value)
  }
}

function handleCanvasClick(event: MouseEvent) {
  if (!currentPageData.value) return

  if (event.metaKey || event.ctrlKey) {
    selectionStore.toggleEvidence(currentPageData.value.page.id)
  } else {
    selectionStore.focusPage(currentPageData.value.page.id)
  }
}

const isCurrentPageInEvidence = computed(() => {
  if (!currentPageData.value) return false
  return selectionStore.isInEvidence(currentPageData.value.page.id)
})

function toggleCurrentPageEvidence() {
  if (currentPageData.value) {
    selectionStore.toggleEvidence(currentPageData.value.page.id)
  }
}

watch(
  () => props.pdfUrl,
  (url) => {
    if (url) loadPdf(url)
  },
  { immediate: true },
)

onUnmounted(() => {
  if (pdfDoc) {
    pdfDoc.destroy()
    pdfDoc = null
  }
})
</script>

<template>
  <div class="flex h-full flex-col overflow-hidden bg-gray-900">
    <!-- Header / Controls -->
    <div class="flex items-center justify-between border-b border-gray-700 px-4 py-3">
      <div class="flex items-center gap-3">
        <h2 class="text-sm font-semibold text-gray-200">
          {{ documentsStore.currentDocumentInfo?.title || 'PDF Viewer' }}
        </h2>
      </div>

      <div class="flex items-center gap-3">
        <!-- Evidence toggle for current page -->
        <Button
          size="sm"
          :variant="isCurrentPageInEvidence ? 'secondary' : 'outline'"
          :class="isCurrentPageInEvidence
            ? 'bg-amber-600/80 text-white hover:bg-amber-700'
            : 'border-gray-600 hover:bg-amber-600/80 hover:text-white hover:border-amber-600'"
          @click="toggleCurrentPageEvidence"
        >
          <span v-if="isCurrentPageInEvidence" class="i-mdi-check mr-1" />
          <span v-else class="i-mdi-plus mr-1" />
          {{ isCurrentPageInEvidence ? 'In evidence' : 'Add to evidence' }}
        </Button>

        <!-- Page Navigation -->
        <div class="flex items-center gap-2">
          <button
            class="rounded px-2 py-1 text-sm text-gray-400 hover:bg-gray-700 hover:text-gray-200 disabled:opacity-30"
            :disabled="!canGoBack"
            @click="prevPage"
          >
            ←
          </button>
          <span class="text-sm text-gray-400">
            {{ currentPage }} / {{ totalPages }}
          </span>
          <button
            class="rounded px-2 py-1 text-sm text-gray-400 hover:bg-gray-700 hover:text-gray-200 disabled:opacity-30"
            :disabled="!canGoForward"
            @click="nextPage"
          >
            →
          </button>
        </div>

        <!-- Evidence count -->
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
      </div>
    </div>

    <!-- Canvas Area -->
    <div class="flex-1 overflow-auto flex items-start justify-center p-4">
      <!-- Loading -->
      <div v-if="isLoading && !canvasRef?.width" class="flex items-center justify-center h-full">
        <Skeleton class="h-[80vh] w-[60vh] bg-gray-700" />
      </div>

      <!-- Error -->
      <div v-else-if="error" class="flex flex-col items-center justify-center h-full text-gray-500">
        <span class="i-mdi-file-alert-outline text-6xl mb-4" />
        <p class="text-sm">Failed to load PDF</p>
        <p class="text-xs mt-1">{{ error }}</p>
      </div>

      <!-- PDF Canvas -->
      <canvas
        ref="canvasRef"
        class="max-w-full cursor-pointer rounded shadow-lg"
        :class="isCurrentPageInEvidence ? 'ring-2 ring-amber-500' : ''"
        @click="handleCanvasClick"
      />
    </div>

    <!-- Footer hint -->
    <div class="border-t border-gray-700 px-4 py-2 text-xs text-gray-500">
      Click to focus page, Cmd+click to toggle evidence
    </div>
  </div>
</template>
