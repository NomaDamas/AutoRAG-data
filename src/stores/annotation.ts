import { invoke } from '@tauri-apps/api/core'
import { acceptHMRUpdate, defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useSelectionStore } from './selection'
import type { PageInfo, ImageChunkInfo } from './documents'

// Types matching the new AutoRAG-Research schema
export interface Query {
  id: number
  contents: string
  query_to_llm: string | null
  generation_gt: string[] | null // text[] - multiple valid answers
}

export interface RetrievalRelation {
  query_id: number
  group_index: number
  group_order: number
  chunk_id: number | null
  image_chunk_id: number | null
}

export interface EvidenceItem {
  relation: RetrievalRelation
  chunk: ImageChunkInfo | null
  page: PageInfo | null
}

export interface EvidenceGroup {
  group_index: number
  items: EvidenceItem[]
}

export interface QueryWithEvidence {
  query: Query
  evidence_groups: EvidenceGroup[]
}

export interface CreateQueryRequest {
  contents: string
  query_to_llm: string | null
  generation_gt: string[] | null
  evidence_groups: number[][] // Vec of groups, each group is Vec of image_chunk_ids
}

export interface AddEvidenceRequest {
  query_id: number
  group_index: number
  image_chunk_id: number
}

export const useAnnotationStore = defineStore('annotation', () => {
  const selectionStore = useSelectionStore()

  // Draft query being edited
  const draftContents = ref('')
  const draftQueryToLlm = ref('')
  const draftGenerationGt = ref<string[]>(['']) // Array of answers

  // Currently editing existing query
  const editingQuery = ref<QueryWithEvidence | null>(null)

  // All queries
  const queries = ref<Query[]>([])
  const isLoading = ref(false)
  const isSaving = ref(false)
  const error = ref<string | null>(null)

  const isDirty = computed(() => {
    if (editingQuery.value) {
      const existingGt = editingQuery.value.query.generation_gt ?? ['']
      return (
        draftContents.value !== editingQuery.value.query.contents ||
        draftQueryToLlm.value !== (editingQuery.value.query.query_to_llm ?? '') ||
        JSON.stringify(draftGenerationGt.value) !== JSON.stringify(existingGt)
      )
    }
    return (
      draftContents.value.trim() !== '' ||
      draftQueryToLlm.value.trim() !== '' ||
      draftGenerationGt.value.some((gt) => gt.trim() !== '')
    )
  })

  const canSave = computed(() => {
    return draftContents.value.trim() !== '' && selectionStore.hasSelection
  })

  // Computed to get non-empty generation_gt values
  const validGenerationGt = computed((): string[] | null => {
    const nonEmpty = draftGenerationGt.value.filter((gt) => gt.trim() !== '')
    return nonEmpty.length > 0 ? nonEmpty : null
  })

  async function loadQueries() {
    isLoading.value = true
    error.value = null

    try {
      queries.value = await invoke<Query[]>('list_queries')
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      queries.value = []
    } finally {
      isLoading.value = false
    }
  }

  async function saveQuery(): Promise<QueryWithEvidence | null> {
    if (!canSave.value) return null

    isSaving.value = true
    error.value = null

    try {
      // Build evidence groups from selected chunks
      // For now, put all selected chunks in a single group (group 0)
      const evidenceGroups: number[][] = [selectionStore.selectedChunkIds]

      const request: CreateQueryRequest = {
        contents: draftContents.value.trim(),
        query_to_llm: draftQueryToLlm.value.trim() || null,
        generation_gt: validGenerationGt.value,
        evidence_groups: evidenceGroups,
      }

      const result = await invoke<QueryWithEvidence>('create_query', { request })

      // Add to local list
      queries.value.unshift(result.query)

      // Clear draft
      clearDraft()

      return result
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      return null
    } finally {
      isSaving.value = false
    }
  }

  async function updateQuery(): Promise<Query | null> {
    if (!editingQuery.value) return null

    isSaving.value = true
    error.value = null

    try {
      const result = await invoke<Query>('update_query', {
        request: {
          id: editingQuery.value.query.id,
          contents: draftContents.value.trim() || null,
          query_to_llm: draftQueryToLlm.value.trim() || null,
          generation_gt: validGenerationGt.value,
        },
      })

      // Update in local list
      const index = queries.value.findIndex((query) => query.id === result.id)
      if (index !== -1) {
        queries.value.splice(index, 1, result)
      }

      // Clear editing state
      editingQuery.value = null
      clearDraft()

      return result
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      return null
    } finally {
      isSaving.value = false
    }
  }

  async function deleteQuery(queryId: number): Promise<boolean> {
    isSaving.value = true
    error.value = null

    try {
      await invoke('delete_query', { queryId })

      // Remove from local list
      queries.value = queries.value.filter((query) => query.id !== queryId)

      // Clear if we were editing this one
      if (editingQuery.value?.query.id === queryId) {
        editingQuery.value = null
        clearDraft()
      }

      return true
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      return false
    } finally {
      isSaving.value = false
    }
  }

  async function loadQueryWithEvidence(queryId: number): Promise<QueryWithEvidence | null> {
    try {
      return await invoke<QueryWithEvidence>('get_query_with_evidence', { queryId })
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      return null
    }
  }

  async function startEditingQuery(queryId: number) {
    const queryWithEvidence = await loadQueryWithEvidence(queryId)
    if (queryWithEvidence) {
      editingQuery.value = queryWithEvidence
      draftContents.value = queryWithEvidence.query.contents
      draftQueryToLlm.value = queryWithEvidence.query.query_to_llm ?? ''
      draftGenerationGt.value = queryWithEvidence.query.generation_gt ?? ['']
      // Ensure at least one empty slot for new answers
      if (draftGenerationGt.value.length === 0) {
        draftGenerationGt.value = ['']
      }
    }
  }

  function cancelEditing() {
    editingQuery.value = null
    clearDraft()
  }

  function clearDraft() {
    draftContents.value = ''
    draftQueryToLlm.value = ''
    draftGenerationGt.value = ['']
  }

  // Helper functions for generation_gt array management
  function addGenerationGt() {
    draftGenerationGt.value.push('')
  }

  function removeGenerationGt(index: number) {
    if (draftGenerationGt.value.length > 1) {
      draftGenerationGt.value.splice(index, 1)
    } else {
      // Keep at least one slot, just clear it
      draftGenerationGt.value[0] = ''
    }
  }

  function updateGenerationGt(index: number, value: string) {
    // eslint-disable-next-line security/detect-object-injection -- safe array index access
    draftGenerationGt.value[index] = value
  }

  return {
    draftContents,
    draftQueryToLlm,
    draftGenerationGt,
    editingQuery,
    queries,
    isLoading,
    isSaving,
    error,
    isDirty,
    canSave,
    validGenerationGt,
    loadQueries,
    saveQuery,
    updateQuery,
    deleteQuery,
    loadQueryWithEvidence,
    startEditingQuery,
    cancelEditing,
    clearDraft,
    addGenerationGt,
    removeGenerationGt,
    updateGenerationGt,
  }
})

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useAnnotationStore, import.meta.hot))
}
