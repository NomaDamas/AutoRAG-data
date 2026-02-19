import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { acceptHMRUpdate, defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Query } from './annotation'

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

export interface DocumentWithPages {
  document: Document
  pages: PageWithChunks[]
}

export interface DocumentDeletionCheck {
  deletable: boolean
  blocking_queries: Query[]
}

interface PageSourceInfo {
  page_id: number
  chunk_ids: number[]
  page_num: number
  source_path: string | null
}

export const useDocumentsStore = defineStore('documents', () => {
  const documents = ref<Document[]>([])
  const currentDocument = ref<DocumentWithPages | null>(null)
  const isLoading = ref(false)
  const isDeleting = ref(false)
  const error = ref<string | null>(null)

  // Source file path for the current document (from file table)
  const sourceFilePath = ref<string | null>(null)

  // Per-page source URLs: pageId → asset:// URL (for image docs) or null
  const pageSourceUrls = ref<Map<number, string>>(new Map())

  const currentPages = computed(() => currentDocument.value?.pages ?? [])
  const currentDocumentInfo = computed(() => currentDocument.value?.document ?? null)
  const pageCount = computed(() => currentDocument.value?.pages.length ?? 0)

  // Whether the current document is a PDF
  const isPdf = computed(() => {
    if (!sourceFilePath.value) return false
    return sourceFilePath.value.toLowerCase().endsWith('.pdf')
  })

  async function loadDocuments() {
    isLoading.value = true
    error.value = null

    try {
      documents.value = await invoke<Document[]>('list_documents')
    } catch (err) {
      console.error('loadDocuments error:', err)
      error.value = err instanceof Error ? err.message : String(err)
      documents.value = []
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
      pageSourceUrls.value.clear()

      // Fetch source file path
      sourceFilePath.value = await invoke<string | null>('get_source_file_url', { documentId })

      // For non-PDF (image) documents, populate page source URLs
      if (!isPdf.value) {
        await loadPageSourceUrls(documentId)
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      currentDocument.value = null
      sourceFilePath.value = null
    } finally {
      isLoading.value = false
    }
  }

  async function loadPageSourceUrls(documentId: number) {
    try {
      const pageInfos = await invoke<PageSourceInfo[]>('get_page_source_urls', { documentId })
      for (const info of pageInfos) {
        if (info.source_path) {
          pageSourceUrls.value.set(info.page_id, convertFileSrc(info.source_path))
        }
      }
    } catch (err) {
      console.error('Failed to load page source URLs:', err)
    }
  }

  function getPageSourceUrl(pageId: number): string | undefined {
    return pageSourceUrls.value.get(pageId)
  }

  async function getChunkDataUrl(chunkId: number): Promise<string | null> {
    try {
      return await invoke<string>('get_chunk_data_url', { chunkId })
    } catch (err) {
      console.error(`Failed to get chunk data URL for chunk ${chunkId}:`, err)
      return null
    }
  }

  async function checkDocumentDeletable(documentId: number): Promise<DocumentDeletionCheck> {
    return await invoke<DocumentDeletionCheck>('check_document_deletable', { documentId })
  }

  async function deleteDocument(documentId: number): Promise<boolean> {
    isDeleting.value = true
    try {
      await invoke<boolean>('delete_document', { documentId })
      documents.value = documents.value.filter((d) => d.id !== documentId)
      if (currentDocument.value?.document.id === documentId) {
        clearCurrentDocument()
      }
      return true
    } catch (err) {
      console.error('deleteDocument error:', err)
      error.value = err instanceof Error ? err.message : String(err)
      return false
    } finally {
      isDeleting.value = false
    }
  }

  function clearCurrentDocument() {
    currentDocument.value = null
    pageSourceUrls.value.clear()
    sourceFilePath.value = null
  }

  return {
    documents,
    currentDocument,
    currentPages,
    currentDocumentInfo,
    pageCount,
    isLoading,
    isDeleting,
    error,
    sourceFilePath,
    isPdf,
    pageSourceUrls,
    loadDocuments,
    selectDocument,
    getPageSourceUrl,
    getChunkDataUrl,
    checkDocumentDeletable,
    deleteDocument,
    clearCurrentDocument,
  }
})

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useDocumentsStore, import.meta.hot))
}
