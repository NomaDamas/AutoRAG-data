import { acceptHMRUpdate, defineStore } from 'pinia'
import { ref } from 'vue'

export const useUiStore = defineStore('ui', () => {
  // Panel sizes (percentages)
  const leftPanelSize = ref(20)
  const rightPanelSize = ref(25)

  // Modal states
  const isConnectionDialogOpen = ref(false)
  const isPreviewModalOpen = ref(false)
  const isIngestDialogOpen = ref(false)
  const previewPageId = ref<number | null>(null)

  // View settings
  const thumbnailSize = ref<'small' | 'medium' | 'large'>('medium')
  const showPageNumbers = ref(true)

  function openConnectionDialog() {
    isConnectionDialogOpen.value = true
  }

  function closeConnectionDialog() {
    isConnectionDialogOpen.value = false
  }

  function openIngestDialog() {
    isIngestDialogOpen.value = true
  }

  function closeIngestDialog() {
    isIngestDialogOpen.value = false
  }

  function openPreview(pageId: number) {
    previewPageId.value = pageId
    isPreviewModalOpen.value = true
  }

  function closePreview() {
    isPreviewModalOpen.value = false
    previewPageId.value = null
  }

  function setThumbnailSize(size: 'small' | 'medium' | 'large') {
    thumbnailSize.value = size
  }

  function togglePageNumbers() {
    showPageNumbers.value = !showPageNumbers.value
  }

  function setLeftPanelSize(size: number) {
    leftPanelSize.value = Math.max(10, Math.min(40, size))
  }

  function setRightPanelSize(size: number) {
    rightPanelSize.value = Math.max(15, Math.min(40, size))
  }

  return {
    leftPanelSize,
    rightPanelSize,
    isConnectionDialogOpen,
    isPreviewModalOpen,
    isIngestDialogOpen,
    previewPageId,
    thumbnailSize,
    showPageNumbers,
    openConnectionDialog,
    closeConnectionDialog,
    openIngestDialog,
    closeIngestDialog,
    openPreview,
    closePreview,
    setThumbnailSize,
    togglePageNumbers,
    setLeftPanelSize,
    setRightPanelSize,
  }
})

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useUiStore, import.meta.hot))
}
