import { acceptHMRUpdate, defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useDocumentsStore, type PageWithChunks } from './documents'

export const useSelectionStore = defineStore('selection', () => {
  const selectedPageIds = ref<Set<number>>(new Set())
  const lastSelectedId = ref<number | null>(null)

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
  }

  function clearSelection() {
    selectedPageIds.value.clear()
    lastSelectedId.value = null
  }

  function selectPage(pageId: number) {
    selectedPageIds.value.clear()
    selectedPageIds.value.add(pageId)
    lastSelectedId.value = pageId
  }

  function addToSelection(pageId: number) {
    selectedPageIds.value.add(pageId)
    lastSelectedId.value = pageId
  }

  function removeFromSelection(pageId: number) {
    selectedPageIds.value.delete(pageId)
  }

  return {
    selectedPageIds,
    lastSelectedId,
    selectedCount,
    hasSelection,
    selectedPages,
    selectedChunkIds,
    selectedPageIdsList,
    isSelected,
    togglePage,
    selectRange,
    selectAll,
    clearSelection,
    selectPage,
    addToSelection,
    removeFromSelection,
  }
})

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useSelectionStore, import.meta.hot))
}
