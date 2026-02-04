import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { acceptHMRUpdate, defineStore } from 'pinia'
import { ref, computed } from 'vue'

// Types matching the new AutoRAG-Research schema
export interface File {
  id: number // bigserial
  type: string // "raw", "image", "audio", "video"
  path: string
}

export interface Document {
  id: number // bigserial
  path: number | null // FK to File.id
  filename: string | null
  author: string | null
  title: string | null
  doc_metadata: Record<string, unknown> | null
}

export interface PageInfo {
  id: number // bigserial
  page_num: number
  document_id: number
  mimetype: string | null
  page_metadata: Record<string, unknown> | null
}

export interface ImageChunkInfo {
  id: number // bigserial
  parent_page: number | null // FK to Page.id
  mimetype: string
}

export interface PageWithChunks {
  page: PageInfo
  chunks: ImageChunkInfo[]
}

export interface FileWithDocuments {
  file: File
  documents: Document[]
}

export interface DocumentWithPages {
  document: Document
  pages: PageWithChunks[]
}

export const useDocumentsStore = defineStore('documents', () => {
  const files = ref<FileWithDocuments[]>([])
  const currentDocument = ref<DocumentWithPages | null>(null)
  const currentFile = ref<File | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const thumbnailUrls = ref<Map<number, string>>(new Map())

  const currentPages = computed(() => currentDocument.value?.pages ?? [])
  const currentDocumentInfo = computed(() => currentDocument.value?.document ?? null)
  const pageCount = computed(() => currentDocument.value?.pages.length ?? 0)

  async function loadFiles() {
    isLoading.value = true
    error.value = null

    try {
      const result = await invoke<FileWithDocuments[]>('list_files_with_documents')
      console.log('list_files_with_documents returned:', result)
      files.value = result
    } catch (err) {
      console.error('loadFiles error:', err)
      error.value = err instanceof Error ? err.message : String(err)
      files.value = []
    } finally {
      isLoading.value = false
    }
  }

  async function selectDocument(documentId: number) {
    isLoading.value = true
    error.value = null

    try {
      currentDocument.value = await invoke<DocumentWithPages>('get_document_with_pages', {
        documentId,
      })

      // Find and set current file
      for (const fileWithDocs of files.value) {
        const foundDoc = fileWithDocs.documents.find((doc) => doc.id === documentId)
        if (foundDoc) {
          currentFile.value = fileWithDocs.file
          break
        }
      }

      // Clear old thumbnails and load new ones
      thumbnailUrls.value.clear()
      await loadThumbnails()
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      currentDocument.value = null
    } finally {
      isLoading.value = false
    }
  }

  async function loadThumbnails() {
    if (!currentDocument.value) return

    for (const pageWithChunks of currentDocument.value.pages) {
      const page = pageWithChunks.page
      if (!thumbnailUrls.value.has(page.id)) {
        try {
          const filePath = await invoke<string>('get_thumbnail_url', {
            pageId: page.id,
          })
          const assetUrl = convertFileSrc(filePath)
          thumbnailUrls.value.set(page.id, assetUrl)
        } catch (err) {
          console.error(`Failed to load thumbnail for page ${page.page_num}:`, err)
        }
      }
    }
  }

  async function getPreviewUrl(pageId: number): Promise<string | null> {
    try {
      const filePath = await invoke<string>('get_preview_url', {
        pageId,
      })
      return convertFileSrc(filePath)
    } catch (err) {
      console.error(`Failed to get preview URL for page ${pageId}:`, err)
      return null
    }
  }

  async function getPageImageUrl(pageId: number): Promise<string | null> {
    try {
      return await invoke<string>('get_page_image_url', {
        pageId,
      })
    } catch (err) {
      console.error(`Failed to get page image URL for page ${pageId}:`, err)
      return null
    }
  }

  async function getChunkImageUrl(chunkId: number): Promise<string | null> {
    try {
      return await invoke<string>('get_chunk_image_url', {
        chunkId,
      })
    } catch (err) {
      console.error(`Failed to get chunk image URL for chunk ${chunkId}:`, err)
      return null
    }
  }

  function clearCurrentDocument() {
    currentDocument.value = null
    currentFile.value = null
    thumbnailUrls.value.clear()
  }

  function getThumbnailUrl(pageId: number): string | undefined {
    return thumbnailUrls.value.get(pageId)
  }

  return {
    files,
    currentDocument,
    currentFile,
    currentPages,
    currentDocumentInfo,
    pageCount,
    isLoading,
    error,
    thumbnailUrls,
    loadFiles,
    selectDocument,
    loadThumbnails,
    getPreviewUrl,
    getPageImageUrl,
    getChunkImageUrl,
    clearCurrentDocument,
    getThumbnailUrl,
  }
})

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useDocumentsStore, import.meta.hot))
}
