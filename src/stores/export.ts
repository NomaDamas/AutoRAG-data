import { acceptHMRUpdate, defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export interface ExportConfig {
  output_path: string
  create_zip: boolean
  include_documents: boolean
  include_queries: boolean
  include_relations: boolean
  include_image_chunks: boolean
  include_images: boolean
}

export interface ExportProgress {
  phase:
    | 'Documents'
    | 'Queries'
    | 'Relations'
    | 'ImageChunks'
    | 'Images'
    | 'Zipping'
    | 'Complete'
    | 'Failed'
  current: number
  total: number
  message: string
}

export interface ExportResult {
  output_path: string
  documents_count: number
  queries_count: number
  relations_count: number
  image_chunks_count: number
  images_count: number
}

export interface ExportCounts {
  documents: number
  queries: number
  relations: number
  image_chunks: number
}

export const useExportStore = defineStore('export', () => {
  const isExporting = ref(false)
  const progress = ref<ExportProgress | null>(null)
  const error = ref<string | null>(null)
  const lastResult = ref<ExportResult | null>(null)
  const counts = ref<ExportCounts | null>(null)

  let unlistenProgress: UnlistenFn | null = null

  const progressPercent = computed(() => {
    if (!progress.value || progress.value.total === 0) return 0
    return Math.round((progress.value.current / progress.value.total) * 100)
  })

  const isComplete = computed(() => progress.value?.phase === 'Complete')
  const isFailed = computed(() => progress.value?.phase === 'Failed')

  async function startListening() {
    if (unlistenProgress) {
      unlistenProgress()
    }

    unlistenProgress = await listen<ExportProgress>('export-progress', (event) => {
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

  async function fetchCounts(): Promise<ExportCounts | null> {
    try {
      const result = await invoke<ExportCounts>('get_export_counts')
      counts.value = result
      return result
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      return null
    }
  }

  async function exportData(config: ExportConfig): Promise<ExportResult | null> {
    isExporting.value = true
    progress.value = null
    error.value = null
    lastResult.value = null

    try {
      await startListening()

      const result = await invoke<ExportResult>('export_data', { config })

      lastResult.value = result
      return result
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      progress.value = {
        phase: 'Failed',
        current: 0,
        total: 0,
        message: error.value,
      }
      return null
    } finally {
      isExporting.value = false
      stopListening()
    }
  }

  function reset() {
    isExporting.value = false
    progress.value = null
    error.value = null
    lastResult.value = null
  }

  return {
    isExporting,
    progress,
    error,
    lastResult,
    counts,
    progressPercent,
    isComplete,
    isFailed,
    fetchCounts,
    exportData,
    reset,
  }
})

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useExportStore, import.meta.hot))
}
