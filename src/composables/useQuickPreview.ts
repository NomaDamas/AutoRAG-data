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

    // Space key for quick preview of focused page
    if (event.code === 'Space' && !event.repeat) {
      event.preventDefault()

      const focusedId = selectionStore.focusedPageId
      if (focusedId !== null) {
        isPreviewActive.value = true

        // Small delay before showing preview to avoid flicker on quick taps
        previewTimeout = setTimeout(() => {
          uiStore.openPreview(focusedId)
        }, 150)
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
