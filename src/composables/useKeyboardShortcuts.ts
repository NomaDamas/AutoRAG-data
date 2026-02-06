import { onMounted, onUnmounted } from 'vue'
import { useSelectionStore, useUiStore } from '@/stores'
import { usePageSelection } from './usePageSelection'

export function useKeyboardShortcuts() {
  const selectionStore = useSelectionStore()
  const uiStore = useUiStore()
  const { focusNextPage, focusPreviousPage } = usePageSelection()

  function handleKeyDown(event: KeyboardEvent) {
    // Ignore if we're in an input field
    if (
      event.target instanceof HTMLInputElement ||
      event.target instanceof HTMLTextAreaElement
    ) {
      return
    }

    // Arrow navigation (focus only, does not affect evidence)
    if (event.key === 'ArrowRight' || event.key === 'ArrowDown') {
      event.preventDefault()
      focusNextPage()
      return
    }

    if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') {
      event.preventDefault()
      focusPreviousPage()
      return
    }

    // Escape: close modals → clear focus (NOT evidence)
    if (event.key === 'Escape') {
      if (uiStore.isPreviewModalOpen) {
        uiStore.closePreview()
      } else if (uiStore.isConnectionDialogOpen) {
        uiStore.closeConnectionDialog()
      } else {
        selectionStore.clearFocus()
      }
      return
    }

    // Enter to open preview of focused page
    if (event.key === 'Enter' && selectionStore.focusedPageId !== null) {
      event.preventDefault()
      uiStore.openPreview(selectionStore.focusedPageId)
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
