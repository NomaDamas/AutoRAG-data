import { useSelectionStore, useDocumentsStore } from '@/stores'

export function usePageSelection() {
  const selectionStore = useSelectionStore()
  const documentsStore = useDocumentsStore()

  function handlePageClick(
    pageId: number,
    event: { shiftKey?: boolean; metaKey?: boolean; ctrlKey?: boolean }
  ) {
    // Use ctrlKey as fallback for non-Mac systems
    const multiSelectKey = event.metaKey || event.ctrlKey

    selectionStore.togglePage(pageId, {
      shiftKey: event.shiftKey,
      metaKey: multiSelectKey,
    })
  }

  function selectNextPage() {
    const pages = documentsStore.currentPages
    if (pages.length === 0) return

    if (selectionStore.selectedCount === 0) {
      // Select first page
      const firstPage = pages[0]
      if (firstPage) {
        selectionStore.selectPage(firstPage.page.id)
      }
      return
    }

    // Find the last selected page and select the next one
    const lastSelectedId = selectionStore.lastSelectedId
    if (lastSelectedId === null) return

    const currentIndex = pages.findIndex((pw) => pw.page.id === lastSelectedId)
    if (currentIndex === -1 || currentIndex >= pages.length - 1) return

    const nextPage = pages[currentIndex + 1]
    if (nextPage) {
      selectionStore.selectPage(nextPage.page.id)
    }
  }

  function selectPreviousPage() {
    const pages = documentsStore.currentPages
    if (pages.length === 0) return

    if (selectionStore.selectedCount === 0) {
      // Select last page
      const lastPage = pages[pages.length - 1]
      if (lastPage) {
        selectionStore.selectPage(lastPage.page.id)
      }
      return
    }

    // Find the last selected page and select the previous one
    const lastSelectedId = selectionStore.lastSelectedId
    if (lastSelectedId === null) return

    const currentIndex = pages.findIndex((pw) => pw.page.id === lastSelectedId)
    if (currentIndex === -1 || currentIndex <= 0) return

    const prevPage = pages[currentIndex - 1]
    if (prevPage) {
      selectionStore.selectPage(prevPage.page.id)
    }
  }

  function extendSelectionNext() {
    const pages = documentsStore.currentPages
    if (pages.length === 0) return

    const lastSelectedId = selectionStore.lastSelectedId
    if (lastSelectedId === null) {
      const firstPage = pages[0]
      if (firstPage) {
        selectionStore.selectPage(firstPage.page.id)
      }
      return
    }

    const currentIndex = pages.findIndex((pw) => pw.page.id === lastSelectedId)
    if (currentIndex === -1 || currentIndex >= pages.length - 1) return

    const nextPage = pages[currentIndex + 1]
    if (nextPage) {
      selectionStore.addToSelection(nextPage.page.id)
    }
  }

  function extendSelectionPrevious() {
    const pages = documentsStore.currentPages
    if (pages.length === 0) return

    const lastSelectedId = selectionStore.lastSelectedId
    if (lastSelectedId === null) {
      const lastPage = pages[pages.length - 1]
      if (lastPage) {
        selectionStore.selectPage(lastPage.page.id)
      }
      return
    }

    const currentIndex = pages.findIndex((pw) => pw.page.id === lastSelectedId)
    if (currentIndex === -1 || currentIndex <= 0) return

    const prevPage = pages[currentIndex - 1]
    if (prevPage) {
      selectionStore.addToSelection(prevPage.page.id)
    }
  }

  return {
    handlePageClick,
    selectNextPage,
    selectPreviousPage,
    extendSelectionNext,
    extendSelectionPrevious,
  }
}
