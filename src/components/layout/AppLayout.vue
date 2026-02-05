<script setup lang="ts">
import { onMounted } from 'vue'
import {
  ResizablePanelGroup,
  ResizablePanel,
  ResizableHandle,
} from '@/components/ui/resizable'
import { useConnectionStore, useDocumentsStore, useUiStore } from '@/stores'
import StatusBar from './StatusBar.vue'
import ConnectionDialog from '@/components/connection/ConnectionDialog.vue'
import IngestDialog from '@/components/ingest/IngestDialog.vue'
import DocumentSelector from '@/components/documents/DocumentSelector.vue'
import PageGrid from '@/components/pages/PageGrid.vue'
import PagePreview from '@/components/pages/PagePreview.vue'
import AnnotationPanel from '@/components/annotation/AnnotationPanel.vue'

const connectionStore = useConnectionStore()
const documentsStore = useDocumentsStore()
const uiStore = useUiStore()

onMounted(async () => {
  await connectionStore.checkConnectionStatus()
  if (connectionStore.isConnected) {
    await documentsStore.loadDocuments()
  }
})
</script>

<template>
  <div class="flex h-screen flex-col bg-gray-900">
    <ResizablePanelGroup direction="horizontal" class="flex-1">
      <!-- Left Panel: Document Selector -->
      <ResizablePanel
        :default-size="uiStore.leftPanelSize"
        :min-size="15"
        :max-size="35"
        class="bg-gray-800"
      >
        <DocumentSelector />
      </ResizablePanel>

      <ResizableHandle class="w-1 bg-gray-700 hover:bg-blue-500 transition-colors" />

      <!-- Center Panel: Page Grid -->
      <ResizablePanel :default-size="100 - uiStore.leftPanelSize - uiStore.rightPanelSize" :min-size="30" class="h-full">
        <PageGrid />
      </ResizablePanel>

      <ResizableHandle class="w-1 bg-gray-700 hover:bg-blue-500 transition-colors" />

      <!-- Right Panel: Annotation -->
      <ResizablePanel
        :default-size="uiStore.rightPanelSize"
        :min-size="20"
        :max-size="40"
        class="bg-gray-800"
      >
        <AnnotationPanel />
      </ResizablePanel>
    </ResizablePanelGroup>

    <StatusBar />

    <ConnectionDialog />
    <IngestDialog />
    <PagePreview />
  </div>
</template>
