import { acceptHMRUpdate, defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export interface IngestionProgress {
  current_page: number
  total_pages: number
  phase: 'Reading' | 'Rendering' | 'Complete' | 'Failed'
  message: string
}

export interface IngestionResult {
  file_id: number
  document_id: number
  page_count: number
  image_chunk_count: number
}

export const useIngestStore = defineStore('ingest', () => {
  const isIngesting = ref(false)
  const progress = ref<IngestionProgress | null>(null)
  const error = ref<string | null>(null)
  const lastResult = ref<IngestionResult | null>(null)

  let unlistenProgress: UnlistenFn | null = null

  const progressPercent = computed(() => {
    if (!progress.value || progress.value.total_pages === 0) return 0
    return Math.round((progress.value.current_page / progress.value.total_pages) * 100)
  })

  const isComplete = computed(() => progress.value?.phase === 'Complete')
  const isFailed = computed(() => progress.value?.phase === 'Failed')

  async function startListening() {
    if (unlistenProgress) {
      unlistenProgress()
    }

    unlistenProgress = await listen<IngestionProgress>('ingestion-progress', (event) => {
      progress.value = event.payload
      if (event.payload.phase === 'Failed') {
        error.value = event.payload.message
      }
    })
  }

  function stopListening() {
    if (unlistenProgress) {
      unlistenProgress()
      unlistenProgress = null
    }
  }

  async function ingestPdf(filePath: string, title?: string, author?: string): Promise<IngestionResult | null> {
    isIngesting.value = true
    progress.value = null
    error.value = null
    lastResult.value = null

    try {
      await startListening()

      const result = await invoke<IngestionResult>('ingest_pdf', {
        filePath,
        title: title || null,
        author: author || null,
      })

      lastResult.value = result
      return result
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      progress.value = {
        current_page: 0,
        total_pages: 0,
        phase: 'Failed',
        message: error.value,
      }
      return null
    } finally {
      isIngesting.value = false
      stopListening()
    }
  }

  async function ingestImages(filePaths: string[], title: string): Promise<IngestionResult | null> {
    isIngesting.value = true
    progress.value = null
    error.value = null
    lastResult.value = null

    try {
      await startListening()

      const result = await invoke<IngestionResult>('ingest_images', {
        filePaths,
        title,
      })

      lastResult.value = result
      return result
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      progress.value = {
        current_page: 0,
        total_pages: 0,
        phase: 'Failed',
        message: error.value,
      }
      return null
    } finally {
      isIngesting.value = false
      stopListening()
    }
  }

  async function getSupportedFormats(): Promise<string[]> {
    try {
      return await invoke<string[]>('get_supported_formats')
    } catch {
      return ['pdf']
    }
  }

  function reset() {
    isIngesting.value = false
    progress.value = null
    error.value = null
    lastResult.value = null
  }

  return {
    isIngesting,
    progress,
    error,
    lastResult,
    progressPercent,
    isComplete,
    isFailed,
    ingestPdf,
    ingestImages,
    getSupportedFormats,
    reset,
  }
})

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useIngestStore, import.meta.hot))
}
