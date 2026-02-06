import { useSelectionStore, useDocumentsStore } from '@/stores'

export function usePageSelection() {
  const selectionStore = useSelectionStore()
  const documentsStore = useDocumentsStore()

  function handlePageClick(
    pageId: number,
    event: { metaKey?: boolean; ctrlKey?: boolean }
  ) {
    const multiSelectKey = event.metaKey || event.ctrlKey

    if (multiSelectKey) {
      selectionStore.toggleEvidence(pageId)
    } else {
      selectionStore.focusPage(pageId)
    }
  }

  function focusNextPage() {
    const pages = documentsStore.currentPages
    if (pages.length === 0) return

    if (selectionStore.focusedPageId === null) {
      const firstPage = pages[0]
      if (firstPage) {
        selectionStore.focusPage(firstPage.page.id)
      }
      return
    }

    const currentIndex = pages.findIndex((pw) => pw.page.id === selectionStore.focusedPageId)
    if (currentIndex === -1 || currentIndex >= pages.length - 1) return

    const nextPage = pages[currentIndex + 1]
    if (nextPage) {
      selectionStore.focusPage(nextPage.page.id)
    }
  }

  function focusPreviousPage() {
    const pages = documentsStore.currentPages
    if (pages.length === 0) return

    if (selectionStore.focusedPageId === null) {
      const lastPage = pages[pages.length - 1]
      if (lastPage) {
        selectionStore.focusPage(lastPage.page.id)
      }
      return
    }

    const currentIndex = pages.findIndex((pw) => pw.page.id === selectionStore.focusedPageId)
    if (currentIndex === -1 || currentIndex <= 0) return

    const prevPage = pages[currentIndex - 1]
    if (prevPage) {
      selectionStore.focusPage(prevPage.page.id)
    }
  }

  return {
    handlePageClick,
    focusNextPage,
    focusPreviousPage,
  }
}
