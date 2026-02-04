import { ref, onMounted, onUnmounted } from 'vue'
import { useSelectionStore, useUiStore } from '@/stores'

export function useQuickPreview() {
  const selectionStore = useSelectionStore()
  const uiStore = useUiStore()

  const isPreviewActive = ref(false)
  let previewTimeout: ReturnType<typeof setTimeout> | null = null

  function handleKeyDown(event: KeyboardEvent) {
    // Ignore if we're in an input field
    if (
      event.target instanceof HTMLInputElement ||
      event.target instanceof HTMLTextAreaElement
    ) {
      return
    }

    // Space key for quick preview
    if (event.code === 'Space' && !event.repeat) {
      event.preventDefault()

      // Only preview if exactly one page is selected
      if (selectionStore.selectedCount === 1) {
        const selectedId = Array.from(selectionStore.selectedPageIds)[0]
        if (selectedId !== undefined) {
          isPreviewActive.value = true

          // Small delay before showing preview to avoid flicker on quick taps
          previewTimeout = setTimeout(() => {
            uiStore.openPreview(selectedId)
          }, 150)
        }
      }
    }
  }

  function handleKeyUp(event: KeyboardEvent) {
    if (event.code === 'Space') {
      isPreviewActive.value = false

      // Cancel pending preview if released quickly
      if (previewTimeout) {
        clearTimeout(previewTimeout)
        previewTimeout = null
      }

      // Close preview
      if (uiStore.isPreviewModalOpen) {
        uiStore.closePreview()
      }
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', handleKeyDown)
    window.addEventListener('keyup', handleKeyUp)
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeyDown)
    window.removeEventListener('keyup', handleKeyUp)
    if (previewTimeout) {
      clearTimeout(previewTimeout)
    }
  })

  return {
    isPreviewActive,
  }
}
