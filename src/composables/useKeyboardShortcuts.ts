import { onMounted, onUnmounted } from 'vue'
import { useSelectionStore, useUiStore } from '@/stores'
import { usePageSelection } from './usePageSelection'

export function useKeyboardShortcuts() {
  const selectionStore = useSelectionStore()
  const uiStore = useUiStore()
  const {
    selectNextPage,
    selectPreviousPage,
    extendSelectionNext,
    extendSelectionPrevious,
  } = usePageSelection()

  function handleKeyDown(event: KeyboardEvent) {
    // Ignore if we're in an input field
    if (
      event.target instanceof HTMLInputElement ||
      event.target instanceof HTMLTextAreaElement
    ) {
      return
    }

    const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0
    const cmdOrCtrl = isMac ? event.metaKey : event.ctrlKey

    // Arrow navigation
    if (event.key === 'ArrowRight' || event.key === 'ArrowDown') {
      event.preventDefault()
      if (event.shiftKey) {
        extendSelectionNext()
      } else {
        selectNextPage()
      }
      return
    }

    if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') {
      event.preventDefault()
      if (event.shiftKey) {
        extendSelectionPrevious()
      } else {
        selectPreviousPage()
      }
      return
    }

    // Select all (Cmd/Ctrl + A)
    if (cmdOrCtrl && event.key === 'a') {
      event.preventDefault()
      selectionStore.selectAll()
      return
    }

    // Escape to clear selection or close modals
    if (event.key === 'Escape') {
      if (uiStore.isPreviewModalOpen) {
        uiStore.closePreview()
      } else if (uiStore.isConnectionDialogOpen) {
        uiStore.closeConnectionDialog()
      } else if (selectionStore.hasSelection) {
        selectionStore.clearSelection()
      }
      return
    }

    // Enter to open preview of selected page
    if (event.key === 'Enter' && selectionStore.selectedCount === 1) {
      event.preventDefault()
      const selectedId = Array.from(selectionStore.selectedPageIds)[0]
      if (selectedId !== undefined) {
        uiStore.openPreview(selectedId)
      }
      return
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', handleKeyDown)
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeyDown)
  })
}
