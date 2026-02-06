<script setup lang="ts">
import { ScrollArea } from '@/components/ui/scroll-area'
import { Skeleton } from '@/components/ui/skeleton'
import { Button } from '@/components/ui/button'
import { useConnectionStore, useDocumentsStore, useSelectionStore, useUiStore } from '@/stores'

const connectionStore = useConnectionStore()
const documentsStore = useDocumentsStore()
const selectionStore = useSelectionStore()
const uiStore = useUiStore()

async function selectDocument(documentId: number) {
  selectionStore.clearFocus()
  await documentsStore.selectDocument(documentId)
}

function isDocumentSelected(documentId: number): boolean {
  return documentsStore.currentDocumentInfo?.id === documentId
}

async function handleRefresh() {
  await documentsStore.loadDocuments()
}
</script>

<template>
  <div class="flex h-full flex-col overflow-hidden">
    <div class="flex items-center justify-between border-b border-gray-700 px-4 py-3">
      <h2 class="text-sm font-semibold text-gray-200">Documents</h2>
      <div class="flex items-center gap-2">
        <Button
          variant="ghost"
          size="sm"
          class="h-7 w-7 p-0"
          :disabled="!connectionStore.isConnected"
          title="Import PDF"
          @click="uiStore.openIngestDialog()"
        >
          <span class="i-mdi-file-import text-lg" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          class="h-7 w-7 p-0"
          :disabled="!connectionStore.isConnected"
          title="Refresh"
          @click="handleRefresh"
        >
          <span class="i-mdi-refresh text-lg" :class="documentsStore.isLoading && 'animate-spin'" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          class="h-7 w-7 p-0"
          title="Database Connection"
          @click="uiStore.openConnectionDialog()"
        >
          <span class="i-mdi-database text-lg" />
        </Button>
      </div>
    </div>

    <ScrollArea class="flex-1">
      <div class="p-2">
        <!-- Not Connected State -->
        <div
          v-if="!connectionStore.isConnected"
          class="flex flex-col items-center justify-center p-8 text-center text-gray-500"
        >
          <span class="i-mdi-database-off text-4xl mb-4" />
          <p class="text-sm">Not connected to database</p>
          <Button
            variant="outline"
            size="sm"
            class="mt-4 border-gray-600"
            @click="uiStore.openConnectionDialog()"
          >
            Connect
          </Button>
        </div>

        <!-- Loading State -->
        <div v-else-if="documentsStore.isLoading && documentsStore.documents.length === 0" class="space-y-2 p-2">
          <Skeleton v-for="i in 5" :key="i" class="h-8 w-full bg-gray-700" />
        </div>

        <!-- Empty State -->
        <div
          v-else-if="documentsStore.documents.length === 0"
          class="flex flex-col items-center justify-center p-8 text-center text-gray-500"
        >
          <span class="i-mdi-file-document-outline text-4xl mb-4" />
          <p class="text-sm">No documents found</p>
        </div>

        <!-- Document List -->
        <div v-else class="space-y-0.5">
          <button
            v-for="doc in documentsStore.documents"
            :key="doc.id"
            class="flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm transition-colors"
            :class="isDocumentSelected(doc.id) ? 'bg-blue-600 text-white' : 'hover:bg-gray-700 text-gray-300'"
            @click="selectDocument(doc.id)"
          >
            <span class="i-mdi-file-document-outline" />
            <span class="flex-1 truncate">{{ doc.title || doc.filename || 'Untitled' }}</span>
          </button>
        </div>
      </div>
    </ScrollArea>
  </div>
</template>
