import { acceptHMRUpdate, defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useDocumentsStore, type PageInfo, type ImageChunkInfo, type PageWithChunks } from './documents'

export interface EvidenceWithScore {
  chunk_id: number
  score: number
}

// Stores everything needed to display evidence without depending on currentPages
export interface EvidencePageItem {
  page: PageInfo
  chunks: ImageChunkInfo[]
  documentId: number
  documentTitle: string
  thumbnailUrl?: string
}

// Minimal shape for restoring evidence groups (avoids circular import with annotation store)
interface EvidenceGroupLike {
  group_index: number
  items: {
    relation?: { score: number } | null
    chunk?: { id: number } | null
    page?: { id: number; page_num: number; document_id: number; mimetype: string | null } | null
  }[]
}

export const useSelectionStore = defineStore('selection', () => {
  // FOCUS (browsing, transient, current-doc)
  const focusedPageId = ref<number | null>(null)

  // EVIDENCE CART (persistent, cross-doc)
  const evidenceItems = ref<Map<number, EvidencePageItem>>(new Map()) // pageId → item

  // Map from chunk_id to score (default is 1 = somewhat relevant)
  const chunkScores = ref<Map<number, number>>(new Map())

  // Grouping state
  const groupingMode = ref<'and_all' | 'custom'>('and_all')
  const customGroups = ref<Set<number>[]>([]) // array of Sets of pageIds

  const documentsStore = useDocumentsStore()

  // Backward-compatible computeds
  const selectedPageIds = computed(() => new Set(evidenceItems.value.keys()))
  const selectedCount = computed(() => evidenceItems.value.size)
  const hasSelection = computed(() => evidenceItems.value.size > 0)

  const selectedPages = computed((): PageWithChunks[] => {
    return Array.from(evidenceItems.value.values()).map((item) => ({
      page: item.page,
      chunks: item.chunks,
    }))
  })

  // Get all chunk IDs from selected pages
  const selectedChunkIds = computed((): number[] => {
    const chunkIds: number[] = []
    for (const item of evidenceItems.value.values()) {
      for (const chunk of item.chunks) {
        chunkIds.push(chunk.id)
      }
    }
    return chunkIds
  })

  // Get all chunks with their scores for creating queries
  const selectedChunksWithScores = computed((): EvidenceWithScore[] => {
    const result: EvidenceWithScore[] = []
    for (const item of evidenceItems.value.values()) {
      for (const chunk of item.chunks) {
        result.push({
          chunk_id: chunk.id,
          score: chunkScores.value.get(chunk.id) ?? 1,
        })
      }
    }
    return result
  })

  // Evidence groups: each group is an array of EvidenceWithScore
  const evidenceGroups = computed((): EvidenceWithScore[][] => {
    const pages = selectedPages.value
    if (groupingMode.value === 'and_all') {
      return pages.map((pw) =>
        pw.chunks.map((chunk) => ({
          chunk_id: chunk.id,
          score: chunkScores.value.get(chunk.id) ?? 1,
        })),
      )
    }

    // custom mode
    return customGroups.value
      .filter((group) => group.size > 0)
      .map((group) => {
        const chunks: EvidenceWithScore[] = []
        for (const pageId of group) {
          const pw = pages.find((pg) => pg.page.id === pageId)
          if (pw) {
            for (const chunk of pw.chunks) {
              chunks.push({
                chunk_id: chunk.id,
                score: chunkScores.value.get(chunk.id) ?? 1,
              })
            }
          }
        }
        return chunks
      })
      .filter((group) => group.length > 0)
  })

  // Grouped pages for UI rendering
  const groupedPages = computed((): { groupIndex: number; pages: PageWithChunks[] }[] => {
    const pages = selectedPages.value
    if (groupingMode.value === 'and_all') {
      return pages.map((pw, i) => ({
        groupIndex: i,
        pages: [pw],
      }))
    }

    // custom mode
    return customGroups.value
      .map((group, idx) => {
        const groupPages = Array.from(group)
          .map((pageId) => pages.find((pg) => pg.page.id === pageId))
          .filter((pg): pg is PageWithChunks => pg !== undefined)
        return { groupIndex: idx, pages: groupPages }
      })
      .filter((grp) => grp.pages.length > 0)
  })

  // Get all page IDs from selected pages
  const selectedPageIdsList = computed((): number[] => {
    return Array.from(evidenceItems.value.keys())
  })

  function getChunkScore(chunkId: number): number {
    return chunkScores.value.get(chunkId) ?? 1
  }

  function setChunkScore(chunkId: number, score: number) {
    chunkScores.value.set(chunkId, score)
  }

  // --- Focus methods (browsing, transient, current-doc) ---

  function focusPage(pageId: number) {
    focusedPageId.value = pageId
  }

  function clearFocus() {
    focusedPageId.value = null
  }

  // --- Evidence methods (persistent, cross-doc) ---

  function isInEvidence(pageId: number): boolean {
    return evidenceItems.value.has(pageId)
  }

  function addEvidence(pageId: number) {
    if (evidenceItems.value.has(pageId)) return

    // Try to find page data from current document's pages
    const pageWithChunks = documentsStore.currentPages.find((pw) => pw.page.id === pageId)
    if (!pageWithChunks) return

    const docInfo = documentsStore.currentDocumentInfo
    const docTitle = docInfo?.title || docInfo?.filename || 'Untitled'
    const thumbnailUrl = documentsStore.getThumbnailUrl(pageId)

    const item: EvidencePageItem = {
      page: { ...pageWithChunks.page },
      chunks: [...pageWithChunks.chunks],
      documentId: pageWithChunks.page.document_id,
      documentTitle: docTitle,
      thumbnailUrl: thumbnailUrl || undefined,
    }

    evidenceItems.value.set(pageId, item)
    syncCustomGroupsAfterSelectionChange()
  }

  function removeEvidence(pageId: number) {
    // Clean up chunk scores for removed page before deleting
    const item = evidenceItems.value.get(pageId)
    if (item) {
      for (const chunk of item.chunks) {
        chunkScores.value.delete(chunk.id)
      }
    }

    evidenceItems.value.delete(pageId)

    if (groupingMode.value === 'custom') {
      for (const group of customGroups.value) {
        group.delete(pageId)
      }
      cleanupEmptyGroups()
    }
    syncCustomGroupsAfterSelectionChange()
  }

  function toggleEvidence(pageId: number) {
    if (evidenceItems.value.has(pageId)) {
      removeEvidence(pageId)
    } else {
      addEvidence(pageId)
    }
  }

  function clearEvidence() {
    evidenceItems.value.clear()
    chunkScores.value.clear()
    customGroups.value = []
    groupingMode.value = 'and_all'
  }

  // Backward compat alias
  function isSelected(pageId: number): boolean {
    return evidenceItems.value.has(pageId)
  }

  // --- Grouping methods ---

  function setGroupingMode(mode: 'and_all' | 'custom') {
    if (mode === groupingMode.value) return

    if (mode === 'custom') {
      customGroups.value = Array.from(evidenceItems.value.keys()).map(
        (pageId) => new Set([pageId]),
      )
    }

    groupingMode.value = mode
  }

  function mergeIntoGroup(targetGroupIndex: number, pageId: number) {
    if (groupingMode.value !== 'custom') return

    for (const group of customGroups.value) {
      group.delete(pageId)
    }

    // eslint-disable-next-line security/detect-object-injection -- safe array index from internal grouping logic
    const target = customGroups.value[targetGroupIndex]
    if (target) {
      target.add(pageId)
    }

    cleanupEmptyGroups()
  }

  function splitToNewGroup(pageId: number) {
    if (groupingMode.value !== 'custom') return

    for (const group of customGroups.value) {
      group.delete(pageId)
    }

    customGroups.value.push(new Set([pageId]))

    cleanupEmptyGroups()
  }

  function cleanupEmptyGroups() {
    customGroups.value = customGroups.value.filter((grp) => grp.size > 0)
  }

  function syncCustomGroupsAfterSelectionChange() {
    if (groupingMode.value !== 'custom') return

    const currentEvidenceIds = evidenceItems.value

    // Track which pages are already in a group
    const pagesInGroups = new Set<number>()
    for (const group of customGroups.value) {
      for (const pageId of group) {
        pagesInGroups.add(pageId)
      }
    }

    // Add new pages as new solo groups
    for (const pageId of currentEvidenceIds.keys()) {
      if (!pagesInGroups.has(pageId)) {
        customGroups.value.push(new Set([pageId]))
      }
    }

    // Purge removed pages from groups
    for (const group of customGroups.value) {
      for (const pageId of group) {
        if (!currentEvidenceIds.has(pageId)) {
          group.delete(pageId)
        }
      }
    }

    cleanupEmptyGroups()
  }

  // Restore groups from loaded query evidence
  function restoreFromEvidenceGroups(evidenceGroupsData: EvidenceGroupLike[]) {
    evidenceItems.value.clear()
    chunkScores.value.clear()

    const groupPageSets: Set<number>[] = []

    for (const eg of evidenceGroupsData) {
      const pageIdsInGroup = new Set<number>()
      for (const item of eg.items) {
        if (item.page) {
          const pageId = item.page.id

          // Build EvidencePageItem from evidence data
          // Try current doc pages first, then build from evidence data
          const currentPageData = documentsStore.currentPages.find((pw) => pw.page.id === pageId)
          const docInfo = documentsStore.currentDocumentInfo

          if (currentPageData) {
            const docTitle = docInfo?.title || docInfo?.filename || 'Untitled'
            evidenceItems.value.set(pageId, {
              page: { ...currentPageData.page },
              chunks: [...currentPageData.chunks],
              documentId: currentPageData.page.document_id,
              documentTitle: docTitle,
              thumbnailUrl: documentsStore.getThumbnailUrl(pageId) || undefined,
            })
          } else {
            // Build from evidence data (cross-doc case or page not in current doc)
            const chunks: ImageChunkInfo[] = []
            // Collect chunks for this page from all items in the group
            for (const groupItem of eg.items) {
              if (groupItem.page?.id === pageId && groupItem.chunk) {
                chunks.push({
                  id: groupItem.chunk.id,
                  parent_page: pageId,
                  mimetype: 'image/png', // default, actual mimetype comes from chunk data
                })
              }
            }
            evidenceItems.value.set(pageId, {
              page: {
                id: item.page.id,
                page_num: item.page.page_num,
                document_id: item.page.document_id,
                mimetype: item.page.mimetype,
                page_metadata: null,
              },
              chunks,
              documentId: item.page.document_id,
              documentTitle: `Document ${item.page.document_id}`,
              thumbnailUrl: documentsStore.getThumbnailUrl(pageId) || undefined,
            })
          }

          pageIdsInGroup.add(pageId)
        }
        if (item.chunk && item.relation) {
          chunkScores.value.set(item.chunk.id, item.relation.score)
        }
      }
      if (pageIdsInGroup.size > 0) {
        groupPageSets.push(pageIdsInGroup)
      }
    }

    // Determine mode
    const isAndAll =
      groupPageSets.length > 0 &&
      groupPageSets.every((grp) => grp.size === 1) &&
      new Set(groupPageSets.flatMap((grp) => Array.from(grp))).size === groupPageSets.length

    if (isAndAll) {
      groupingMode.value = 'and_all'
      customGroups.value = []
    } else {
      groupingMode.value = 'custom'
      customGroups.value = groupPageSets
    }
  }

  return {
    // Focus state
    focusedPageId,
    focusPage,
    clearFocus,

    // Evidence state
    evidenceItems,
    addEvidence,
    removeEvidence,
    toggleEvidence,
    clearEvidence,
    isInEvidence,

    // Backward-compatible computeds
    selectedPageIds,
    selectedCount,
    hasSelection,
    selectedPages,
    selectedChunkIds,
    selectedChunksWithScores,
    selectedPageIdsList,
    isSelected,

    // Scores
    chunkScores,
    getChunkScore,
    setChunkScore,

    // Grouping
    groupingMode,
    customGroups,
    evidenceGroups,
    groupedPages,
    setGroupingMode,
    mergeIntoGroup,
    splitToNewGroup,
    restoreFromEvidenceGroups,
  }
})

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useSelectionStore, import.meta.hot))
}
