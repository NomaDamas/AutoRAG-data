import { acceptHMRUpdate, defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useDocumentsStore, type PageWithChunks } from './documents'

export interface EvidenceWithScore {
  chunk_id: number
  score: number
}

// Minimal shape for restoring evidence groups (avoids circular import with annotation store)
interface EvidenceGroupLike {
  group_index: number
  items: {
    relation?: { score: number } | null
    chunk?: { id: number } | null
    page?: { id: number } | null
  }[]
}

export const useSelectionStore = defineStore('selection', () => {
  const selectedPageIds = ref<Set<number>>(new Set())
  const lastSelectedId = ref<number | null>(null)
  // Map from chunk_id to score (default is 1 = somewhat relevant)
  const chunkScores = ref<Map<number, number>>(new Map())

  // Grouping state
  const groupingMode = ref<'and_all' | 'custom'>('and_all')
  const customGroups = ref<Set<number>[]>([]) // array of Sets of pageIds

  const documentsStore = useDocumentsStore()

  const selectedCount = computed(() => selectedPageIds.value.size)
  const hasSelection = computed(() => selectedPageIds.value.size > 0)

  const selectedPages = computed((): PageWithChunks[] => {
    return documentsStore.currentPages.filter((pw) => selectedPageIds.value.has(pw.page.id))
  })

  // Get all chunk IDs from selected pages
  const selectedChunkIds = computed((): number[] => {
    const chunkIds: number[] = []
    for (const pageWithChunks of selectedPages.value) {
      for (const chunk of pageWithChunks.chunks) {
        chunkIds.push(chunk.id)
      }
    }
    return chunkIds
  })

  // Get all chunks with their scores for creating queries
  const selectedChunksWithScores = computed((): EvidenceWithScore[] => {
    const result: EvidenceWithScore[] = []
    for (const pageWithChunks of selectedPages.value) {
      for (const chunk of pageWithChunks.chunks) {
        result.push({
          chunk_id: chunk.id,
          score: chunkScores.value.get(chunk.id) ?? 1, // Default to 1 (somewhat relevant)
        })
      }
    }
    return result
  })

  // Evidence groups: each group is an array of EvidenceWithScore
  // and_all: each page's chunks are their own group (AND semantics)
  // custom: groups defined by customGroups
  const evidenceGroups = computed((): EvidenceWithScore[][] => {
    if (groupingMode.value === 'and_all') {
      return selectedPages.value.map((pw) =>
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
          const pw = selectedPages.value.find((pg) => pg.page.id === pageId)
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
    if (groupingMode.value === 'and_all') {
      return selectedPages.value.map((pw, i) => ({
        groupIndex: i,
        pages: [pw],
      }))
    }

    // custom mode
    return customGroups.value
      .map((group, idx) => {
        const pages = Array.from(group)
          .map((pageId) => selectedPages.value.find((pg) => pg.page.id === pageId))
          .filter((pg): pg is PageWithChunks => pg !== undefined)
        return { groupIndex: idx, pages }
      })
      .filter((grp) => grp.pages.length > 0)
  })

  function getChunkScore(chunkId: number): number {
    return chunkScores.value.get(chunkId) ?? 1 // Default to 1 (somewhat relevant)
  }

  function setChunkScore(chunkId: number, score: number) {
    chunkScores.value.set(chunkId, score)
  }

  // Get all page IDs from selected pages (for when chunks aren't available)
  const selectedPageIdsList = computed((): number[] => {
    return Array.from(selectedPageIds.value)
  })

  function isSelected(pageId: number): boolean {
    return selectedPageIds.value.has(pageId)
  }

  function togglePage(pageId: number, event?: { shiftKey?: boolean; metaKey?: boolean }) {
    if (event?.shiftKey && lastSelectedId.value !== null) {
      // Shift+click: select range
      selectRange(lastSelectedId.value, pageId)
    } else if (event?.metaKey) {
      // Cmd+click: toggle single
      if (selectedPageIds.value.has(pageId)) {
        selectedPageIds.value.delete(pageId)
      } else {
        selectedPageIds.value.add(pageId)
        lastSelectedId.value = pageId
      }
    } else {
      // Regular click: select only this one
      selectedPageIds.value.clear()
      selectedPageIds.value.add(pageId)
      lastSelectedId.value = pageId
    }
    syncCustomGroupsAfterSelectionChange()
  }

  function selectRange(fromId: number, toId: number) {
    const allPages = documentsStore.currentPages
    const fromIndex = allPages.findIndex((pw) => pw.page.id === fromId)
    const toIndex = allPages.findIndex((pw) => pw.page.id === toId)

    if (fromIndex === -1 || toIndex === -1) return

    const start = Math.min(fromIndex, toIndex)
    const end = Math.max(fromIndex, toIndex)

    for (let idx = start; idx <= end; idx++) {
      // eslint-disable-next-line security/detect-object-injection -- safe array index access
      const pageAtIndex = allPages[idx]
      if (pageAtIndex) {
        selectedPageIds.value.add(pageAtIndex.page.id)
      }
    }

    lastSelectedId.value = toId
    syncCustomGroupsAfterSelectionChange()
  }

  function selectAll() {
    const pages = documentsStore.currentPages
    for (const page of pages) {
      selectedPageIds.value.add(page.page.id)
    }
    const lastPage = pages[pages.length - 1]
    if (lastPage) {
      lastSelectedId.value = lastPage.page.id
    }
    syncCustomGroupsAfterSelectionChange()
  }

  function clearSelection() {
    selectedPageIds.value.clear()
    lastSelectedId.value = null
    chunkScores.value.clear()
    customGroups.value = []
    groupingMode.value = 'and_all'
  }

  function selectPage(pageId: number) {
    selectedPageIds.value.clear()
    selectedPageIds.value.add(pageId)
    lastSelectedId.value = pageId
    syncCustomGroupsAfterSelectionChange()
  }

  function addToSelection(pageId: number) {
    selectedPageIds.value.add(pageId)
    lastSelectedId.value = pageId
    syncCustomGroupsAfterSelectionChange()
  }

  function removeFromSelection(pageId: number) {
    selectedPageIds.value.delete(pageId)
    if (groupingMode.value === 'custom') {
      for (const group of customGroups.value) {
        group.delete(pageId)
      }
      cleanupEmptyGroups()
    }
  }

  // --- Grouping methods ---

  function setGroupingMode(mode: 'and_all' | 'custom') {
    if (mode === groupingMode.value) return

    if (mode === 'custom') {
      // Initialize: each selected page as its own group
      customGroups.value = Array.from(selectedPageIds.value).map(
        (pageId) => new Set([pageId]),
      )
    }
    // Switching to and_all: custom groups are discarded (computed handles it)

    groupingMode.value = mode
  }

  function mergeIntoGroup(targetGroupIndex: number, pageId: number) {
    if (groupingMode.value !== 'custom') return

    // Remove from current group
    for (const group of customGroups.value) {
      group.delete(pageId)
    }

    // Add to target group
    // eslint-disable-next-line security/detect-object-injection -- safe array index from internal grouping logic
    const target = customGroups.value[targetGroupIndex]
    if (target) {
      target.add(pageId)
    }

    cleanupEmptyGroups()
  }

  function splitToNewGroup(pageId: number) {
    if (groupingMode.value !== 'custom') return

    // Remove from current group
    for (const group of customGroups.value) {
      group.delete(pageId)
    }

    // Create new group
    customGroups.value.push(new Set([pageId]))

    cleanupEmptyGroups()
  }

  function cleanupEmptyGroups() {
    customGroups.value = customGroups.value.filter((grp) => grp.size > 0)
  }

  function syncCustomGroupsAfterSelectionChange() {
    if (groupingMode.value !== 'custom') return

    // Track which pages are already in a group
    const pagesInGroups = new Set<number>()
    for (const group of customGroups.value) {
      for (const pageId of group) {
        pagesInGroups.add(pageId)
      }
    }

    // Add new pages as new solo groups
    for (const pageId of selectedPageIds.value) {
      if (!pagesInGroups.has(pageId)) {
        customGroups.value.push(new Set([pageId]))
      }
    }

    // Purge removed pages from groups
    for (const group of customGroups.value) {
      for (const pageId of group) {
        if (!selectedPageIds.value.has(pageId)) {
          group.delete(pageId)
        }
      }
    }

    cleanupEmptyGroups()
  }

  // Restore groups from loaded query evidence
  function restoreFromEvidenceGroups(evidenceGroupsData: EvidenceGroupLike[]) {
    // Clear existing state
    selectedPageIds.value.clear()
    chunkScores.value.clear()

    // Build page-to-group mapping from evidence
    const groupPageSets: Set<number>[] = []

    for (const eg of evidenceGroupsData) {
      const pageIdsInGroup = new Set<number>()
      for (const item of eg.items) {
        if (item.page) {
          const pageId = item.page.id
          selectedPageIds.value.add(pageId)
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

    // Determine mode: if every group has exactly one unique page, it's and_all
    const isAndAll =
      groupPageSets.length > 0 &&
      groupPageSets.every((grp) => grp.size === 1) &&
      // Also check that no two groups share the same page
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
    selectedPageIds,
    lastSelectedId,
    chunkScores,
    selectedCount,
    hasSelection,
    selectedPages,
    selectedChunkIds,
    selectedChunksWithScores,
    selectedPageIdsList,
    isSelected,
    togglePage,
    selectRange,
    selectAll,
    clearSelection,
    selectPage,
    addToSelection,
    removeFromSelection,
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
