<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useDocumentsStore, useUiStore, type DocumentDeletionCheck } from '@/stores'

const documentsStore = useDocumentsStore()
const uiStore = useUiStore()

const isChecking = ref(false)
const checkResult = ref<DocumentDeletionCheck | null>(null)
const checkError = ref<string | null>(null)

watch(
  () => uiStore.isDeleteDocumentDialogOpen,
  async (isOpen) => {
    if (isOpen && uiStore.deleteDocumentTarget) {
      isChecking.value = true
      checkResult.value = null
      checkError.value = null
      try {
        checkResult.value = await documentsStore.checkDocumentDeletable(
          uiStore.deleteDocumentTarget.id,
        )
      } catch (err) {
        checkError.value = err instanceof Error ? err.message : String(err)
      } finally {
        isChecking.value = false
      }
    } else {
      checkResult.value = null
      checkError.value = null
    }
  },
)

async function handleDelete() {
  if (!uiStore.deleteDocumentTarget) return
  const success = await documentsStore.deleteDocument(uiStore.deleteDocumentTarget.id)
  if (success) {
    uiStore.closeDeleteDocumentDialog()
  }
}
</script>

<template>
  <Dialog v-model:open="uiStore.isDeleteDocumentDialogOpen">
    <DialogContent class="bg-gray-800 border-gray-700 text-gray-100 sm:max-w-md">
      <!-- Loading State -->
      <template v-if="isChecking">
        <DialogHeader>
          <DialogTitle>Checking Document</DialogTitle>
          <DialogDescription class="text-gray-400">
            Verifying whether this document can be safely deleted...
          </DialogDescription>
        </DialogHeader>
        <div class="flex items-center justify-center py-8">
          <span class="i-mdi-loading animate-spin text-2xl text-gray-400" />
        </div>
      </template>

      <!-- Error State -->
      <template v-else-if="checkError">
        <DialogHeader>
          <DialogTitle>Error</DialogTitle>
          <DialogDescription class="text-gray-400">
            Failed to check document deletion status.
          </DialogDescription>
        </DialogHeader>
        <div class="rounded-md bg-red-900/50 p-3 text-sm text-red-300">
          <span class="i-mdi-alert-circle mr-2" />
          {{ checkError }}
        </div>
        <DialogFooter>
          <Button
            variant="outline"
            class="border-gray-600 hover:bg-gray-700"
            @click="uiStore.closeDeleteDocumentDialog()"
          >
            Close
          </Button>
        </DialogFooter>
      </template>

      <!-- Blocked State -->
      <template v-else-if="checkResult && !checkResult.deletable">
        <DialogHeader>
          <DialogTitle class="text-red-400">Cannot Delete</DialogTitle>
          <DialogDescription class="text-gray-400">
            "{{ uiStore.deleteDocumentTarget?.title }}" has chunks referenced as retrieval ground
            truth in {{ checkResult.blocking_queries.length }} query/queries. Remove these
            references first.
          </DialogDescription>
        </DialogHeader>
        <ScrollArea class="max-h-60">
          <div class="space-y-1 pr-3">
            <div
              v-for="query in checkResult.blocking_queries"
              :key="query.id"
              class="rounded bg-gray-700/50 px-3 py-2 text-sm"
            >
              <span class="font-mono text-gray-500">Q{{ query.id }}:</span>
              <span class="ml-2 text-gray-300">{{ query.contents }}</span>
            </div>
          </div>
        </ScrollArea>
        <DialogFooter>
          <Button
            variant="outline"
            class="border-gray-600 hover:bg-gray-700"
            @click="uiStore.closeDeleteDocumentDialog()"
          >
            Close
          </Button>
        </DialogFooter>
      </template>

      <!-- Confirm State -->
      <template v-else-if="checkResult && checkResult.deletable">
        <DialogHeader>
          <DialogTitle>Delete Document</DialogTitle>
          <DialogDescription class="text-gray-400">
            Are you sure you want to delete "{{ uiStore.deleteDocumentTarget?.title }}"? This will
            permanently remove the document and all its pages and chunks.
          </DialogDescription>
        </DialogHeader>
        <DialogFooter class="gap-2">
          <Button
            variant="outline"
            class="border-gray-600 hover:bg-gray-700"
            :disabled="documentsStore.isDeleting"
            @click="uiStore.closeDeleteDocumentDialog()"
          >
            Cancel
          </Button>
          <Button
            variant="destructive"
            :disabled="documentsStore.isDeleting"
            @click="handleDelete"
          >
            <span v-if="documentsStore.isDeleting" class="i-mdi-loading animate-spin mr-2" />
            <span v-else class="i-mdi-delete mr-2" />
            {{ documentsStore.isDeleting ? 'Deleting...' : 'Delete' }}
          </Button>
        </DialogFooter>
      </template>
    </DialogContent>
  </Dialog>
</template>
