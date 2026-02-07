<script setup lang="ts">
import { ref, shallowRef, watch, onUnmounted, computed, nextTick } from 'vue'
import * as pdfjsLib from 'pdfjs-dist'
import { useThrottleFn } from '@vueuse/core'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { useSelectionStore, useDocumentsStore } from '@/stores'
import { usePdfPageObserver } from '@/composables'

pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
  'pdfjs-dist/build/pdf.worker.mjs',
  import.meta.url,
).toString()

const props = defineProps<{
  pdfUrl: string
}>()

const selectionStore = useSelectionStore()
const documentsStore = useDocumentsStore()

const totalPages = ref(0)
const currentPage = ref(1)
const pageInputValue = ref('1')
const isLoading = ref(false)
const error = ref<string | null>(null)

// Aspect ratio for placeholder sizing (width / height from page 1)
const pageAspectRatio = ref<number | null>(null)

const pdfDocRef = shallowRef<pdfjsLib.PDFDocumentProxy | null>(null)

// Scroll container refs
const mainScrollRef = ref<HTMLElement | null>(null)
const thumbScrollRef = ref<HTMLElement | null>(null)

// Canvas refs for each page (keyed by index)
const mainCanvasRefs = ref<Map<number, HTMLCanvasElement>>(new Map())
const thumbCanvasRefs = ref<Map<number, HTMLCanvasElement>>(new Map())

// Page container refs for scrolling
const pageContainerRefs = ref<Map<number, HTMLElement>>(new Map())
const thumbContainerRefs = ref<Map<number, HTMLElement>>(new Map())

// Observers
const mainObserver = usePdfPageObserver(pdfDocRef, 1.5, '400px 0px')
const thumbObserver = usePdfPageObserver(pdfDocRef, 0.2, '200px 0px')

// --- Helpers ---

function getPageData(pageNum: number) {
  return (
    documentsStore.currentPages.find((pw) => pw.page.page_num === pageNum) ??
    null
  )
}

const currentPageData = computed(() => getPageData(currentPage.value))

const isCurrentPageInEvidence = computed(() => {
  if (!currentPageData.value) return false
  return selectionStore.isInEvidence(currentPageData.value.page.id)
})

function isPageInEvidence(pageNum: number): boolean {
  const pd = getPageData(pageNum)
  return pd ? selectionStore.isInEvidence(pd.page.id) : false
}

function toggleCurrentPageEvidence() {
  if (currentPageData.value) {
    selectionStore.toggleEvidence(currentPageData.value.page.id)
  }
}

function handlePageClick(pageNum: number, event: MouseEvent) {
  const pd = getPageData(pageNum)
  if (!pd) return

  if (event.metaKey || event.ctrlKey) {
    selectionStore.toggleEvidence(pd.page.id)
  } else {
    selectionStore.focusPage(pd.page.id)
  }
}

// --- Page input / jump ---

function jumpToPage() {
  const num = parseInt(pageInputValue.value, 10)
  if (isNaN(num)) {
    pageInputValue.value = String(currentPage.value)
    return
  }
  const clamped = Math.max(1, Math.min(num, totalPages.value))
  scrollToPage(clamped)
}

function resetPageInput() {
  pageInputValue.value = String(currentPage.value)
}

function scrollToPage(pageNum: number) {
  const container = pageContainerRefs.value.get(pageNum)
  if (container) {
    container.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }
}

// --- Scroll tracking ---

const updateCurrentPageFromScroll = useThrottleFn(() => {
  const scrollEl = mainScrollRef.value
  if (!scrollEl) return

  const scrollMid = scrollEl.scrollTop + scrollEl.clientHeight / 2
  let closestPage = 1
  let closestDist = Infinity

  for (const [pageNum, el] of pageContainerRefs.value) {
    const elMid = el.offsetTop + el.offsetHeight / 2
    const dist = Math.abs(elMid - scrollMid)
    if (dist < closestDist) {
      closestDist = dist
      closestPage = pageNum
    }
  }

  if (closestPage !== currentPage.value) {
    currentPage.value = closestPage
    pageInputValue.value = String(closestPage)
    scrollThumbIntoView(closestPage)
  }
}, 100)

function scrollThumbIntoView(pageNum: number) {
  const thumbEl = thumbContainerRefs.value.get(pageNum)
  if (thumbEl) {
    thumbEl.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
  }
}

// --- PDF loading ---

async function loadPdf(url: string) {
  isLoading.value = true
  error.value = null
  mainObserver.reset()
  thumbObserver.reset()
  mainCanvasRefs.value.clear()
  thumbCanvasRefs.value.clear()
  pageContainerRefs.value.clear()
  thumbContainerRefs.value.clear()

  try {
    if (pdfDocRef.value) {
      pdfDocRef.value.destroy()
      pdfDocRef.value = null
    }

    const loadingTask = pdfjsLib.getDocument(url)
    const doc = await loadingTask.promise
    pdfDocRef.value = doc
    totalPages.value = doc.numPages
    currentPage.value = 1
    pageInputValue.value = '1'

    // Get aspect ratio from page 1
    const firstPage = await doc.getPage(1)
    const vp = firstPage.getViewport({ scale: 1 })
    pageAspectRatio.value = vp.width / vp.height

    // Wait for DOM to render, then set up observers
    await nextTick()
    setupObservers()
  } catch (err) {
    console.error('Failed to load PDF:', err)
    error.value = err instanceof Error ? err.message : String(err)
  } finally {
    isLoading.value = false
  }
}

function setupObservers() {
  if (mainScrollRef.value) {
    mainObserver.createObserver(mainScrollRef.value)
  }
  if (thumbScrollRef.value) {
    thumbObserver.createObserver(thumbScrollRef.value)
  }

  // Observe all registered canvases
  for (const [pageNum, canvas] of mainCanvasRefs.value) {
    mainObserver.observe(canvas, pageNum)
  }
  for (const [pageNum, canvas] of thumbCanvasRefs.value) {
    thumbObserver.observe(canvas, pageNum)
  }
}

// --- Template ref callbacks ---

function setMainCanvas(el: HTMLCanvasElement | null, pageNum: number) {
  if (el) {
    mainCanvasRefs.value.set(pageNum, el)
    mainObserver.observe(el, pageNum)
  }
}

function setThumbCanvas(el: HTMLCanvasElement | null, pageNum: number) {
  if (el) {
    thumbCanvasRefs.value.set(pageNum, el)
    thumbObserver.observe(el, pageNum)
  }
}

function setPageContainer(el: HTMLElement | null, pageNum: number) {
  if (el) pageContainerRefs.value.set(pageNum, el)
}

function setThumbContainer(el: HTMLElement | null, pageNum: number) {
  if (el) thumbContainerRefs.value.set(pageNum, el)
}

// --- Lifecycle ---

watch(
  () => props.pdfUrl,
  (url) => {
    if (url) loadPdf(url)
  },
  { immediate: true },
)

onUnmounted(() => {
  if (pdfDocRef.value) {
    pdfDocRef.value.destroy()
    pdfDocRef.value = null
  }
})

// Generate array of page numbers for v-for
const pageNumbers = computed(() =>
  Array.from({ length: totalPages.value }, (_, i) => i + 1),
)
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

        <!-- Page input -->
        <div class="flex items-center gap-1">
          <input
            v-model="pageInputValue"
            class="w-12 rounded bg-gray-800 px-2 py-1 text-center text-sm text-gray-200 outline-none ring-1 ring-gray-600 focus:ring-blue-500"
            @keydown.enter="jumpToPage"
            @blur="resetPageInput"
          />
          <span class="text-sm text-gray-400">/ {{ totalPages }}</span>
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

    <!-- Main content: thumbnail sidebar + scrollable pages -->
    <div class="flex min-h-0 flex-1">
      <!-- Thumbnail sidebar -->
      <div
        ref="thumbScrollRef"
        class="w-24 shrink-0 overflow-y-auto border-r border-gray-700 bg-gray-800/50 p-2"
      >
        <div
          v-for="pageNum in pageNumbers"
          :key="'thumb-' + pageNum"
          :ref="(el) => setThumbContainer(el as HTMLElement, pageNum)"
          class="mb-2 cursor-pointer rounded p-1 transition-all"
          :class="[
            pageNum === currentPage ? 'ring-2 ring-blue-500' : '',
            isPageInEvidence(pageNum) ? 'ring-2 ring-amber-500' : '',
            pageNum === currentPage && isPageInEvidence(pageNum) ? 'ring-2 ring-amber-500' : '',
          ]"
          @click="scrollToPage(pageNum)"
        >
          <div
            class="relative w-full overflow-hidden rounded bg-gray-700"
            :style="pageAspectRatio ? { aspectRatio: String(pageAspectRatio) } : {}"
          >
            <canvas
              :ref="(el) => setThumbCanvas(el as HTMLCanvasElement, pageNum)"
              class="h-full w-full object-contain"
            />
            <!-- Amber dot for evidence pages -->
            <div
              v-if="isPageInEvidence(pageNum)"
              class="absolute right-1 top-1 h-2 w-2 rounded-full bg-amber-500"
            />
          </div>
          <p class="mt-1 text-center text-[10px] text-gray-400">{{ pageNum }}</p>
        </div>
      </div>

      <!-- Main scrollable view -->
      <div
        ref="mainScrollRef"
        class="flex-1 overflow-y-auto"
        @scroll="updateCurrentPageFromScroll"
      >
        <!-- Loading skeleton -->
        <div v-if="isLoading && totalPages === 0" class="flex items-center justify-center p-8">
          <Skeleton class="h-[80vh] w-[60vh] bg-gray-700" />
        </div>

        <!-- Error -->
        <div v-else-if="error" class="flex flex-col items-center justify-center p-8 text-gray-500">
          <span class="i-mdi-file-alert-outline mb-4 text-6xl" />
          <p class="text-sm">Failed to load PDF</p>
          <p class="mt-1 text-xs">{{ error }}</p>
        </div>

        <!-- Pages -->
        <div v-else class="flex flex-col items-center gap-4 p-4">
          <div
            v-for="pageNum in pageNumbers"
            :key="'page-' + pageNum"
            :ref="(el) => setPageContainer(el as HTMLElement, pageNum)"
            class="w-full max-w-4xl"
          >
            <div
              class="relative overflow-hidden rounded shadow-lg"
              :style="pageAspectRatio ? { aspectRatio: String(pageAspectRatio) } : {}"
              :class="[
                isPageInEvidence(pageNum) ? 'ring-2 ring-amber-500' : '',
                'cursor-pointer',
              ]"
              @click="handlePageClick(pageNum, $event)"
            >
              <canvas
                :ref="(el) => setMainCanvas(el as HTMLCanvasElement, pageNum)"
                class="h-full w-full object-contain"
              />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Footer hint -->
    <div class="border-t border-gray-700 px-4 py-2 text-xs text-gray-500">
      Click to focus page, Cmd+click to toggle evidence
    </div>
  </div>
</template>
