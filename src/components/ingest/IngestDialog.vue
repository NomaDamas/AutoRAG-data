<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Progress } from '@/components/ui/progress'
import { useConnectionStore, useDocumentsStore, useIngestStore, useUiStore } from '@/stores'

const connectionStore = useConnectionStore()
const documentsStore = useDocumentsStore()
const ingestStore = useIngestStore()
const uiStore = useUiStore()

const selectedFile = ref<string | null>(null)
const titleOverride = ref('')
const authorOverride = ref('')

const fileName = computed(() => {
  if (!selectedFile.value) return ''
  const parts = selectedFile.value.split('/')
  return parts[parts.length - 1] || ''
})

const canIngest = computed(() => {
  return (
    selectedFile.value &&
    connectionStore.isConnected &&
    !ingestStore.isIngesting
  )
})

// Reset form when dialog closes
watch(
  () => uiStore.isIngestDialogOpen,
  (isOpen) => {
    if (!isOpen) {
      selectedFile.value = null
      titleOverride.value = ''
      authorOverride.value = ''
      ingestStore.reset()
    }
  }
)

async function handleSelectFile() {
  const result = await open({
    multiple: false,
    filters: [
      {
        name: 'PDF Documents',
        extensions: ['pdf'],
      },
    ],
  })

  if (result) {
    selectedFile.value = result as string
  }
}

async function handleIngest() {
  if (!selectedFile.value) return

  const result = await ingestStore.ingestPdf(
    selectedFile.value,
    titleOverride.value || undefined,
    authorOverride.value || undefined
  )

  if (result) {
    console.log('Ingestion successful:', result)
    // Refresh document list
    await documentsStore.loadFiles()
    console.log('Files loaded:', documentsStore.files)
  }
}

function handleClose() {
  uiStore.closeIngestDialog()
}
</script>

<template>
  <Dialog v-model:open="uiStore.isIngestDialogOpen">
    <DialogContent class="bg-gray-800 border-gray-700 text-gray-100 sm:max-w-md">
      <DialogHeader>
        <DialogTitle>Import PDF</DialogTitle>
        <DialogDescription class="text-gray-400">
          Select a PDF file to import into the database. Each page will be rendered as an image.
        </DialogDescription>
      </DialogHeader>

      <div class="space-y-4">
        <!-- Not Connected Warning -->
        <div
          v-if="!connectionStore.isConnected"
          class="rounded-md bg-yellow-900/50 p-3 text-sm text-yellow-300"
        >
          <span class="i-mdi-alert mr-2" />
          Please connect to a database first.
        </div>

        <!-- File Selection -->
        <div class="space-y-2">
          <Label>PDF File</Label>
          <div class="flex gap-2">
            <Input
              :model-value="fileName"
              readonly
              class="flex-1 bg-gray-700 border-gray-600"
              placeholder="No file selected"
            />
            <Button
              type="button"
              variant="outline"
              class="border-gray-600 hover:bg-gray-700"
              :disabled="ingestStore.isIngesting"
              @click="handleSelectFile"
            >
              <span class="i-mdi-folder-open mr-2" />
              Browse
            </Button>
          </div>
        </div>

        <!-- Optional Overrides -->
        <div class="space-y-4">
          <div class="space-y-2">
            <Label for="title-override">Title (optional)</Label>
            <Input
              id="title-override"
              v-model="titleOverride"
              class="bg-gray-700 border-gray-600"
              placeholder="Override PDF title"
              :disabled="ingestStore.isIngesting"
            />
          </div>

          <div class="space-y-2">
            <Label for="author-override">Author (optional)</Label>
            <Input
              id="author-override"
              v-model="authorOverride"
              class="bg-gray-700 border-gray-600"
              placeholder="Override PDF author"
              :disabled="ingestStore.isIngesting"
            />
          </div>
        </div>

        <!-- Progress -->
        <div v-if="ingestStore.progress" class="space-y-2">
          <div class="flex items-center justify-between text-sm">
            <span class="text-gray-400">{{ ingestStore.progress.message }}</span>
            <span class="text-gray-400">{{ ingestStore.progressPercent }}%</span>
          </div>
          <Progress :model-value="ingestStore.progressPercent" class="h-2" />
        </div>

        <!-- Error Message -->
        <div
          v-if="ingestStore.error && !ingestStore.isIngesting"
          class="rounded-md bg-red-900/50 p-3 text-sm text-red-300"
        >
          <span class="i-mdi-alert-circle mr-2" />
          {{ ingestStore.error }}
        </div>

        <!-- Success Message -->
        <div
          v-if="ingestStore.isComplete && ingestStore.lastResult"
          class="rounded-md bg-green-900/50 p-3 text-sm text-green-300"
        >
          <span class="i-mdi-check-circle mr-2" />
          Successfully imported {{ ingestStore.lastResult.page_count }} pages.
        </div>
      </div>

      <DialogFooter class="gap-2">
        <Button
          type="button"
          variant="outline"
          class="border-gray-600 hover:bg-gray-700"
          @click="handleClose"
        >
          {{ ingestStore.isComplete ? 'Close' : 'Cancel' }}
        </Button>
        <Button
          v-if="!ingestStore.isComplete"
          type="button"
          :disabled="!canIngest"
          class="bg-blue-600 hover:bg-blue-700"
          @click="handleIngest"
        >
          <span v-if="ingestStore.isIngesting" class="i-mdi-loading animate-spin mr-2" />
          <span v-else class="i-mdi-file-import mr-2" />
          {{ ingestStore.isIngesting ? 'Importing...' : 'Import' }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
